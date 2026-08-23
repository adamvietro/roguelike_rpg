use crate::prelude::*;
use ron::de::from_reader;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs::File;

#[derive(Clone, Deserialize, Debug, PartialEq)]
pub enum EntityType {
    Enemy,
    Item,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Template {
    pub entity_type: EntityType,
    pub levels: HashSet<usize>,
    pub frequency: i32,
    pub name: String,
    pub glyph: char,
    pub provides: Option<Vec<(String, i32)>>,
    pub hp: Option<i32>,
    pub base_damage: Option<i32>,
    pub speed: Option<i32>,
    /// A one-time battle technique's mechanical effect, e.g.
    /// `DamageMultiplier(2)` or `Counter(chance_percent: 65, multiplier: 3)`.
    /// See components::TechniqueEffect. Replaces the old approach of
    /// matching hardcoded strings ("Deathblow", "QuickAttack", ...) in
    /// `provides` and adding a distinct marker component per match -
    /// `provides` is still used for non-technique effects (Healing,
    /// MagicMap).
    pub technique: Option<TechniqueEffect>,
    /// Which class can use/be granted this item, e.g. "Barbarian". None
    /// means unrestricted - usable by anyone (weapons, potions, etc. stay
    /// unrestricted unless you want to gate those too later).
    pub class: Option<String>,
    /// Flavor/mechanical text shown when hovering this item's HUD listing.
    pub description: Option<String>,
    /// If true, this template never appears in the general ambient spawn
    /// pool (spawn_entities) - it's only ever placed directly at a
    /// prefab's dedicated marker point (see
    /// map_builder::prefab and spawn_prefab_weapon/spawn_prefab_enemies).
    /// Missing from template.ron defaults to false via serde.
    #[serde(default)]
    pub prefab_only: bool,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Templates {
    pub entities: Vec<Template>,
}

/// Percent chance a battle victory grants a random loot item at all.
const BATTLE_LOOT_DROP_CHANCE_PERCENT: i32 = 40;

impl Templates {
    pub fn load() -> Self {
        let file = File::open("resources/template.ron").expect("Failed opening file");
        from_reader(file).expect("Unable to load templates")
    }

    pub fn spawn_entities(
        &self,
        ecs: &mut World,
        rng: &mut RandomNumberGenerator,
        level: usize,
        spawn_points: &[Point],
    ) {
        let mut available_entities = Vec::new();
        self.entities
            .iter()
            .filter(|e| e.levels.contains(&level) && !e.prefab_only)
            .for_each(|t| {
                for _ in 0..t.frequency {
                    available_entities.push(t);
                }
            });

        let mut commands = legion::systems::CommandBuffer::new(ecs);
        spawn_points.iter().for_each(|pt| {
            if let Some(entity) = rng.random_slice_entry(&available_entities) {
                self.spawn_entity(pt, entity, &mut commands);
            }
        });
        commands.flush(ecs);
    }

    /// Spawns a guaranteed enemy (never an item) at each of `spawn_points`,
    /// weighted by frequency among Enemy-type templates for this level.
    /// Used for a prefab's guard positions (see
    /// map_builder::prefab / MapBuilder::prefab_enemy_spawns) - unlike
    /// spawn_entities' general pool, which mixes enemies and items
    /// together with no guarantee either way, this only ever picks an
    /// actual monster.
    pub fn spawn_prefab_enemies(
        &self,
        ecs: &mut World,
        rng: &mut RandomNumberGenerator,
        level: usize,
        spawn_points: &[Point],
    ) {
        let mut available_enemies = Vec::new();
        self.entities
            .iter()
            .filter(|t| t.entity_type == EntityType::Enemy && t.levels.contains(&level))
            .for_each(|t| {
                for _ in 0..t.frequency {
                    available_enemies.push(t);
                }
            });

        let mut commands = legion::systems::CommandBuffer::new(ecs);
        spawn_points.iter().for_each(|pt| {
            if let Some(entity) = rng.random_slice_entry(&available_enemies) {
                self.spawn_entity(pt, entity, &mut commands);
            }
        });
        commands.flush(ecs);
    }

    /// Spawns a guaranteed weapon at `spawn_point`, if a prefab placed
    /// successfully this level (see map_builder::prefab /
    /// MapBuilder::prefab_weapon_spawn - placement can fail, so this may
    /// be None). Picks randomly, weighted by frequency, among ALL
    /// `prefab_only` templates whose `levels` includes this dungeon level
    /// - Swords and Staffs share this same single guaranteed slot (a level
    /// gets one or the other, not both), the same way `levels` already
    /// gates the general ambient pool. Neither weapon family appears in
    /// that general pool at all - see Template::prefab_only.
    pub fn spawn_prefab_weapon(
        &self,
        ecs: &mut World,
        rng: &mut RandomNumberGenerator,
        level: usize,
        spawn_point: Option<Point>,
    ) {
        let pt = match spawn_point {
            Some(pt) => pt,
            None => return,
        };

        let mut available_weapons = Vec::new();
        self.entities
            .iter()
            .filter(|t| t.prefab_only && t.levels.contains(&level))
            .for_each(|t| {
                for _ in 0..t.frequency {
                    available_weapons.push(t);
                }
            });

        if let Some(template) = rng.random_slice_entry(&available_weapons) {
            let mut commands = legion::systems::CommandBuffer::new(ecs);
            self.spawn_entity(&pt, template, &mut commands);
            commands.flush(ecs);
        }
    }

    /// Rolls a chance to grant the player a random one-time battle item
    /// after a battle victory, picked only from templates whose `class`
    /// matches `player_class` - a Barbarian only ever gets Barbarian
    /// techniques, etc. Call this from battle_tick when an enemy dies.
    /// Returns the granted item's display name, if any.
    pub fn grant_random_battle_loot(
        &self,
        ecs: &mut World,
        rng: &mut RandomNumberGenerator,
        player: Entity,
        player_class: &str,
    ) -> Option<String> {
        if rng.range(0, 100) >= BATTLE_LOOT_DROP_CHANCE_PERCENT {
            return None;
        }

        let candidates: Vec<&Template> = self
            .entities
            .iter()
            .filter(|t| t.class.as_deref() == Some(player_class))
            .collect();
        let template = rng.random_slice_entry(&candidates)?;

        let mut commands = legion::systems::CommandBuffer::new(ecs);
        let entity = commands.push((
            Render {
                color: ColorPair::new(WHITE, BLACK),
                glyph: to_cp437(template.glyph),
            },
            Name(template.name.clone()),
            Item {},
            Carried(player),
        ));
        Self::apply_provides(template, entity, &mut commands);
        Self::apply_technique(template, entity, &mut commands);
        Self::apply_class(template, entity, &mut commands);
        Self::apply_description(template, entity, &mut commands);
        commands.flush(ecs);

        Some(template.name.clone())
    }

    /// Spawns a guaranteed named item directly into `player`'s inventory -
    /// no RNG, no drop chance, no pool filtering. Used for class starting
    /// kits (see spawner::grant_starting_items /
    /// resources/starting_kits.ron), which can name ANY template
    /// including `prefab_only` weapons or `levels: []` battle-loot
    /// techniques - this bypasses those restrictions entirely and grants
    /// exactly what's asked for. Unlike grant_random_battle_loot, this
    /// also handles base_damage/Weapon, so a starting kit can hand out a
    /// real weapon (e.g. a starting Staff), not just techniques/potions.
    /// Silently warns and does nothing if `name` doesn't match any
    /// template, so a typo in starting_kits.ron can't crash the game.
    pub fn spawn_named_item(&self, ecs: &mut World, player: Entity, name: &str) {
        let template = match self.entities.iter().find(|t| t.name == name) {
            Some(t) => t,
            None => {
                println!("Warning: starting kit references unknown item '{}'", name);
                return;
            }
        };

        let mut commands = legion::systems::CommandBuffer::new(ecs);
        let entity = commands.push((
            Render {
                color: ColorPair::new(WHITE, BLACK),
                glyph: to_cp437(template.glyph),
            },
            Name(template.name.clone()),
            Item {},
            Carried(player),
        ));
        Self::apply_provides(template, entity, &mut commands);
        Self::apply_technique(template, entity, &mut commands);
        Self::apply_class(template, entity, &mut commands);
        Self::apply_description(template, entity, &mut commands);
        if let Some(damage) = &template.base_damage {
            commands.add_component(entity, Damage(*damage));
            if template.entity_type == EntityType::Item {
                commands.add_component(entity, Weapon {});
            }
        }
        commands.flush(ecs);
    }

    fn spawn_entity(
        &self,
        pt: &Point,
        template: &Template,
        commands: &mut legion::systems::CommandBuffer,
    ) {
        let entity = commands.push((
            pt.clone(),
            Render {
                color: ColorPair::new(WHITE, BLACK),
                glyph: to_cp437(template.glyph),
            },
            Name(template.name.clone()),
        ));
        match template.entity_type {
            EntityType::Item => commands.add_component(entity, Item {}),
            EntityType::Enemy => {
                commands.add_component(entity, Enemy {});
                commands.add_component(entity, FieldOfView::new(6));
                commands.add_component(entity, ChasingPlayer {});
                commands.add_component(entity, CanAttack);
                commands.add_component(entity, Speed(template.speed.unwrap_or(5)));
                commands.add_component(
                    entity,
                    Health {
                        current: template.hp.unwrap(),
                        max: template.hp.unwrap(),
                    },
                );
            }
        }
        Self::apply_provides(template, entity, commands);
        Self::apply_technique(template, entity, commands);
        Self::apply_class(template, entity, commands);
        Self::apply_description(template, entity, commands);
        if let Some(damage) = &template.base_damage {
            commands.add_component(entity, Damage(*damage));
            if template.entity_type == EntityType::Item {
                commands.add_component(entity, Weapon {});
            }
        }
    }

    /// Adds whichever ProvidesXxx component a template's `provides` list
    /// calls for. Only non-technique effects live here now (Healing,
    /// MagicMap, Invisibility) - one-time battle techniques are handled by
    /// `apply_technique` instead, driven directly by `Template.technique`
    /// rather than a string tag.
    fn apply_provides(
        template: &Template,
        entity: Entity,
        commands: &mut legion::systems::CommandBuffer,
    ) {
        if let Some(effects) = &template.provides {
            effects
                .iter()
                .for_each(|(provides, n)| match provides.as_str() {
                    "Healing" => commands.add_component(entity, ProvidesHealing { amount: *n }),
                    "MagicMap" => commands.add_component(entity, ProvidesDungeonMap {}),
                    "Invisibility" => {
                        commands.add_component(entity, ProvidesInvisibility { moves: *n })
                    }
                    _ => {
                        println!("Warning: we don't know how to provide {}", provides);
                    }
                });
        }
    }

    /// Tags an entity with its template's Technique effect (and BattleItem,
    /// so it shows up in the HUD's separate "Battle Attacks" panel), if it
    /// has one. Shared by spawn_entity and grant_random_battle_loot.
    fn apply_technique(
        template: &Template,
        entity: Entity,
        commands: &mut legion::systems::CommandBuffer,
    ) {
        if let Some(effect) = template.technique {
            commands.add_component(entity, Technique(effect));
            commands.add_component(entity, BattleItem);
        }
    }

    /// Tags an entity with its template's Class, if it has one. Shared by
    /// spawn_entity and grant_random_battle_loot so a class-gated item
    /// carries its restriction wherever it's created.
    fn apply_class(
        template: &Template,
        entity: Entity,
        commands: &mut legion::systems::CommandBuffer,
    ) {
        if let Some(class) = &template.class {
            commands.add_component(entity, Class(class.clone()));
        }
    }

    /// Tags an entity with its template's Description, if it has one.
    fn apply_description(
        template: &Template,
        entity: Entity,
        commands: &mut legion::systems::CommandBuffer,
    ) {
        if let Some(description) = &template.description {
            commands.add_component(entity, Description(description.clone()));
        }
    }
}
