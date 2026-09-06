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
    /// An out-of-combat item's effect, e.g. `Healing(6)` or
    /// `IceArmor(defense_bonus: 2, attacks: 10)`. See
    /// components::ProvidesEffect. Applied when used from the dungeon-view
    /// item list (systems/use_items.rs) - separate from `technique`, which
    /// is for battle-menu-only items.
    pub effect: Option<ProvidesEffect>,
    pub hp: Option<i32>,
    pub base_damage: Option<i32>,
    pub speed: Option<i32>,
    /// A one-time battle technique's mechanical effect, e.g.
    /// `DamageMultiplier(2)` or `Counter(chance_percent: 65, multiplier: 3)`.
    /// See components::TechniqueEffect.
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
    /// If true, this Enemy template is a level boss - never a normal
    /// ambient spawn, only ever placed via spawn_boss at
    /// MapBuilder::amulet_start (the same "furthest reachable point"
    /// already used as the Exit tile on levels 0/1 and the Amulet's
    /// position on level 2). Missing from template.ron defaults to false.
    #[serde(default)]
    pub boss_only: bool,
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
            .filter(|e| e.levels.contains(&level) && !e.prefab_only && !e.boss_only)
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
    /// actual monster. Excludes boss_only templates - a boss is placed
    /// exclusively via spawn_boss, never as a regular prefab guard.
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
            .filter(|t| {
                t.entity_type == EntityType::Enemy && t.levels.contains(&level) && !t.boss_only
            })
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

    /// Spawns a guaranteed boss (never an item) at `spawn_point` - the
    /// level's MapBuilder::amulet_start, the same "furthest reachable
    /// point" already used as the Exit tile on levels 0/1 and the
    /// Amulet's own position on level 2, so a boss placed there is
    /// standing directly on (or as close as the map allows to) the thing
    /// it's guarding. Since the player can never move onto a tile another
    /// entity occupies (see systems/player_input.rs - stepping onto an
    /// enemy's tile starts a battle instead of moving), this guarantees
    /// the boss must be defeated before the stairs/Amulet can actually be
    /// reached.
    ///
    /// Picks randomly, weighted by frequency, among Enemy templates whose
    /// `boss_only` is true and whose `levels` includes this dungeon
    /// level. If no boss is defined for this level yet, this silently
    /// does nothing - same "no candidates, no spawn" behavior as
    /// spawn_prefab_weapon when a class has no matching weapon tier.
    /// Tags the spawned entity with components::Boss in addition to
    /// everything spawn_entity already sets up for a normal Enemy.
    pub fn spawn_boss(
        &self,
        ecs: &mut World,
        rng: &mut RandomNumberGenerator,
        level: usize,
        spawn_point: Point,
    ) {
        let mut available_bosses = Vec::new();
        self.entities
            .iter()
            .filter(|t| {
                t.entity_type == EntityType::Enemy && t.boss_only && t.levels.contains(&level)
            })
            .for_each(|t| {
                for _ in 0..t.frequency {
                    available_bosses.push(t);
                }
            });

        let mut commands = legion::systems::CommandBuffer::new(ecs);
        if let Some(template) = rng.random_slice_entry(&available_bosses) {
            let entity = self.spawn_entity(&spawn_point, template, &mut commands);
            commands.add_component(entity, Boss);
        }
        commands.flush(ecs);
    }

    /// Spawns a guaranteed weapon at `spawn_point`, if a prefab placed
    /// successfully this level (see map_builder::prefab /
    /// MapBuilder::prefab_weapon_spawn - placement can fail, so this may
    /// be None). Picks randomly, weighted by frequency, among `prefab_only`
    /// templates whose `levels` includes this dungeon level AND whose
    /// `class` either matches `player_class` or is unset (unrestricted) -
    /// e.g. a Mage only ever finds Staffs here, never a Sword. If a class
    /// has no matching weapon tier defined yet, the pool comes up empty
    /// and nothing spawns this level - silent, not an error, same as
    /// when spawn_point is None.
    pub fn spawn_prefab_weapon(
        &self,
        ecs: &mut World,
        rng: &mut RandomNumberGenerator,
        level: usize,
        spawn_point: Option<Point>,
        player_class: &str,
    ) {
        let pt = match spawn_point {
            Some(pt) => pt,
            None => return,
        };

        let mut available_weapons = Vec::new();
        self.entities
            .iter()
            .filter(|t| {
                t.prefab_only
                    && t.levels.contains(&level)
                    && (t.class.is_none() || t.class.as_deref() == Some(player_class))
            })
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
    /// techniques, etc. Excludes `prefab_only` templates (Swords, Staffs)
    /// even when their class matches - those are guaranteed fortress
    /// treasure only (see spawn_prefab_weapon), never random loot; without
    /// this exclusion, tagging Staffs `class: Some("Mage")` to fix them
    /// spawning for the wrong class also made them eligible here by
    /// accident. Call this from battle_tick when an enemy dies. Returns
    /// the granted item's display name, if any.
    /// Rolls a chance to grant the player a random one-time battle item
    /// after a battle victory (see BATTLE_LOOT_DROP_CHANCE_PERCENT above),
    /// filtered to items matching `player_class`. `enemy` is the entity
    /// that was just defeated - if it's tagged Boss (see
    /// spawner::spawn_boss), the drop is guaranteed instead of rolled,
    /// so a boss always leaves something behind. Called before the enemy
    /// entity is actually removed (see screens/battle.rs), so `enemy` is
    /// still valid to look up here.
    pub fn grant_random_battle_loot(
        &self,
        ecs: &mut World,
        rng: &mut RandomNumberGenerator,
        player: Entity,
        enemy: Entity,
        player_class: &str,
    ) -> Option<String> {
        let is_boss = ecs
            .entry_ref(enemy)
            .map(|e| e.get_component::<Boss>().is_ok())
            .unwrap_or(false);
        if !is_boss && rng.range(0, 100) >= BATTLE_LOOT_DROP_CHANCE_PERCENT {
            return None;
        }

        let candidates: Vec<&Template> = self
            .entities
            .iter()
            .filter(|t| !t.prefab_only && t.class.as_deref() == Some(player_class))
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
        Self::apply_effect(template, entity, &mut commands);
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
        Self::apply_effect(template, entity, &mut commands);
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

    /// Whether `name`'s template is a weapon (has base_damage) - used by
    /// the arena shop's buy handler to decide whether buying this item
    /// should replace a previously-carried weapon, without that handler
    /// needing to know anything about Template's internal fields.
    pub fn item_is_weapon(&self, name: &str) -> bool {
        self.entities
            .iter()
            .find(|t| t.name == name)
            .map(|t| t.base_damage.is_some())
            .unwrap_or(false)
    }

    /// Same as spawn_named_item, but pushes through a CommandBuffer
    /// instead of taking `&mut World` directly - every step it shares
    /// with spawn_named_item (apply_effect/apply_technique/apply_class/
    /// apply_description/weapon damage) already takes a CommandBuffer
    /// internally, so this is just spawn_named_item's same body with the
    /// top-level push/flush swapped for commands.push - which makes this
    /// version usable from INSIDE a system (like player_input's
    /// buy_nearby_item), where only a SubWorld + CommandBuffer are
    /// available, not a real `&mut World`.
    pub fn spawn_named_item_via_commands(
        &self,
        name: &str,
        player: Entity,
        commands: &mut CommandBuffer,
    ) {
        let template = match self.entities.iter().find(|t| t.name == name) {
            Some(t) => t,
            None => {
                println!("Warning: arena shop references unknown item '{}'", name);
                return;
            }
        };

        let entity = commands.push((
            Render {
                color: ColorPair::new(WHITE, BLACK),
                glyph: to_cp437(template.glyph),
            },
            Name(template.name.clone()),
            Item {},
            Carried(player),
        ));
        Self::apply_effect(template, entity, commands);
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

    /// Places a lightweight shop-counter marker at `pt`: just enough to
    /// display and track it (Point + Render + Name + ShopStock + Price),
    /// NOT a real usable Item - it has no Effect/Technique/Weapon/Carried
    /// components at all, so it can't be picked up, used, or equipped
    /// directly. Buying it (see player_input.rs's buy_nearby_item) grants
    /// a real copy via spawn_named_item_via_commands and decrements this
    /// marker's ShopStock, removing the marker entirely once it reaches
    /// 0 - "the shop ran out."
    pub fn spawn_shop_stock_at(
        &self,
        ecs: &mut World,
        name: &str,
        pt: Point,
        quantity: i32,
        price: i32,
    ) {
        let template = match self.entities.iter().find(|t| t.name == name) {
            Some(t) => t,
            None => {
                println!("Warning: arena shop references unknown item '{}'", name);
                return;
            }
        };
        ecs.push((
            pt,
            Render {
                color: ColorPair::new(WHITE, BLACK),
                glyph: to_cp437(template.glyph),
            },
            Name(template.name.clone()),
            ShopStock(quantity),
            Price(price),
        ));
    }

    /// Picks which weapon template the arena shop should offer `class` at
    /// a given (0-indexed) template level - the first entry in file order
    /// whose `class` matches and whose `levels` set contains `level` and
    /// which actually has `base_damage` (i.e. is a real weapon, not a
    /// potion/technique). Where a class has more than one weapon tagged
    /// for the same level (e.g. Barbarian's Shiny Sword and Huge Sword
    /// both include level 1), this deliberately just takes the first
    /// match in file order rather than picking between them - fine for
    /// this first pass, revisit if the shop ever needs to offer a choice
    /// of weapons at the same tier.
    pub fn weapon_name_for_class_level(&self, class: &str, level: usize) -> Option<String> {
        self.entities
            .iter()
            .find(|t| {
                t.class.as_deref() == Some(class)
                    && t.levels.contains(&level)
                    && t.base_damage.is_some()
            })
            .map(|t| t.name.clone())
    }

    /// Every distinct technique name defined for `class`, regardless of
    /// whether the player currently owns any copies - used by the battle
    /// menu to show a class's full technique roster with unowned ones
    /// greyed out, instead of only ever showing what's currently carried.
    /// Preserves template.ron's file order. Technique-only, deliberately -
    /// the battle menu can only ever act on a Technique (an out-of-combat
    /// Effect item like Stealth or Throw Spear isn't usable mid-battle at
    /// all), so widening this to include Effects would list abilities
    /// there that the menu could never actually let you use. See
    /// class_ability_names_for_class below for the shop's broader version.
    pub fn technique_names_for_class(&self, class: &str) -> Vec<String> {
        self.entities
            .iter()
            .filter(|t| t.technique.is_some() && t.class.as_deref() == Some(class))
            .map(|t| t.name.clone())
            .collect()
    }

    /// Every distinct out-of-combat ABILITY name defined for `class` -
    /// Trap, Throw Spear, Invisible Cloak, and so on - in template.ron's
    /// own file order, which is what fixes the Ability Bar's slot order
    /// (see components::ability_bar_slots/systems/hud.rs). Deliberately
    /// mirrors technique_names_for_class's shape but filters on `effect`
    /// instead of `technique`, so a class's out-of-combat abilities and
    /// its in-battle techniques stay two clearly separate rosters -
    /// exactly the split the Item Menu (universal consumables) and
    /// Ability Bar (class abilities) now rely on.
    pub fn effect_names_for_class(&self, class: &str) -> Vec<String> {
        self.entities
            .iter()
            .filter(|t| t.effect.is_some() && t.class.as_deref() == Some(class))
            .map(|t| t.name.clone())
            .collect()
    }

    /// Every distinct universal (no `class:` tag) usable item name defined
    /// in template.ron - Healing Potion, Dungeon Map, and any future item
    /// every class can carry - in template.ron's own file order, which
    /// fixes the Item Bar's slot order (see components::item_bar_slots/
    /// systems/hud.rs). Mirrors effect_names_for_class's shape exactly,
    /// but for items with NO class restriction instead of one specific
    /// class - the template-level equivalent of components::
    /// usable_menu_items' `item_class(..).is_none()` filter.
    pub fn universal_item_names(&self) -> Vec<String> {
        self.entities
            .iter()
            .filter(|t| t.effect.is_some() && t.class.is_none())
            .map(|t| t.name.clone())
            .collect()
    }

    /// The glyph a template with this exact `name` renders as - used by
    /// the Ability Bar (systems/hud.rs) to show an icon for a roster slot
    /// the player doesn't currently own any copies of, where there's no
    /// carried entity to read a Render component from at all. None if no
    /// template has that name (shouldn't happen for anything actually in
    /// a class's roster, but fails quietly rather than panicking).
    pub fn glyph_for_name(&self, name: &str) -> Option<char> {
        self.entities
            .iter()
            .find(|t| t.name == name)
            .map(|t| t.glyph)
    }

    /// The Description text a template with this exact `name` carries, if
    /// any - same reasoning as glyph_for_name (the Ability Bar needs this
    /// for a roster slot with no carried entity to read a Description
    /// component from).
    pub fn description_for_name(&self, name: &str) -> Option<String> {
        self.entities
            .iter()
            .find(|t| t.name == name)
            .and_then(|t| t.description.clone())
    }

    /// Every distinct ABILITY name defined for `class` - both in-battle
    /// Techniques (Deathblow, Fireball, ...) AND out-of-combat Effects
    /// (Stealth, Throw Spear, Invisible Cloak, ...), as long as the
    /// template has a `class:` tag. Used by the Battle Arena shop's ability
    /// roll (see arena::roll_arena_shop_items) - unlike
    /// technique_names_for_class above, the shop sells BOTH kinds of
    /// ability, so restricting it to Techniques silently made every
    /// Effect-based ability (an entire class's worth, for some classes -
    /// Rogue's Stealth/Amazon's Throw Spear+Poison Spear/Hunter's Shoot+
    /// Freeze Trap/Mage's Invisible Cloak+Ice Armor) impossible to ever
    /// roll into a shop's stock, no matter how many times the RNG ran.
    /// Weapons never match this filter even though they also carry a
    /// `class:` tag - a weapon template has neither `technique` nor
    /// `effect` set (only `base_damage`), so it's excluded automatically
    /// rather than needing its own explicit exclusion check.
    pub fn class_ability_names_for_class(&self, class: &str) -> Vec<String> {
        self.entities
            .iter()
            .filter(|t| {
                t.class.as_deref() == Some(class) && (t.technique.is_some() || t.effect.is_some())
            })
            .map(|t| t.name.clone())
            .collect()
    }

    /// Returns the newly-spawned Entity - spawn_boss needs it to attach
    /// the components::Boss tag afterward; every other caller ignores the
    /// return value, which Rust allows without a warning.
    fn spawn_entity(
        &self,
        pt: &Point,
        template: &Template,
        commands: &mut legion::systems::CommandBuffer,
    ) -> Entity {
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
                // Pre-seed an already-expired MovingAnimation right at
                // spawn, rather than waiting for this enemy's first real
                // move to attach one for the first time. Every move after
                // the first is a cheap overwrite of an existing component
                // (see systems/animation.rs's tick_animations, which never
                // removes this once attached) - but an entity's very
                // first-ever move is still a genuinely new component being
                // added, which is a structural change other systems can
                // read one frame late (the same root cause behind the
                // player's old opposite-direction jump). Spawning with one
                // already in place, already expired (elapsed_ms at the
                // duration, start/end both the spawn tile), closes that
                // last gap: gliding_position reads it as "not animating"
                // immediately, and this enemy's actual first step is just
                // another overwrite like every move after it.
                commands.add_component(
                    entity,
                    MovingAnimation {
                        start: *pt,
                        end: *pt,
                        elapsed_ms: MOVE_ANIM_DURATION_MS,
                    },
                );
                // See IdleAnimation's own doc comment - same placeholder
                // (all frames = the base glyph) as the player gets in
                // spawner/mod.rs's spawn_player.
                commands.add_component(entity, idle_frames_for(to_cp437(template.glyph)));
            }
        }
        Self::apply_effect(template, entity, commands);
        Self::apply_technique(template, entity, commands);
        Self::apply_class(template, entity, commands);
        Self::apply_description(template, entity, commands);
        if let Some(damage) = &template.base_damage {
            commands.add_component(entity, Damage(*damage));
            if template.entity_type == EntityType::Item {
                commands.add_component(entity, Weapon {});
            }
        }
        entity
    }

    /// Tags an entity with its template's out-of-combat Effect, if it has
    /// one. Only non-technique effects live here - one-time battle
    /// techniques are handled by `apply_technique` instead.
    fn apply_effect(
        template: &Template,
        entity: Entity,
        commands: &mut legion::systems::CommandBuffer,
    ) {
        if let Some(effect) = template.effect {
            commands.add_component(entity, Effect(effect));
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
