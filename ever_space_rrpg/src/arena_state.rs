use crate::prelude::*;
use crate::State;

/// Battle Arena's own orchestration methods on `State` - split out of
/// `main.rs` (2026-09-13) to match the convention every dungeon/menu
/// screen already follows (`screens/pause.rs`, `screens/battle.rs`,
/// `screens/item_menu.rs`, ... each add their own methods to `State`
/// from their own file; Rust privacy lets a descendant module see an
/// ancestor's private fields, so this needs no `pub` on anything `State`
/// already wasn't exposing). `main.rs` keeps general `State` bootstrap/
/// dispatch plus Dungeon Crawl's own two transition methods
/// (`advance_level`, `dungeon_shop_transition`) - shop-building itself
/// (`build_shop_room`/`spawn_arena_shop_items`/`reveal_and_freeze_fov`)
/// stays in `main.rs` too, since Dungeon Crawl's shop transition calls
/// it just as much as Arena's does.
impl State {
    /// Builds a fresh Battle Arena run for `class` - the arena-mode
    /// counterpart to start_game, called from class_select when
    /// adventure_mode is BattleArena instead of DungeonCrawl. Rather than
    /// a normal dungeon floor, this drops the player straight into the
    /// starting shop (level 0 gear) - see MapBuilder::new_arena_shop.
    /// Deliberately does NOT call grant_starting_items: in Arena mode the
    /// shop itself is the player's starting kit, so granting the normal
    /// dungeon-crawl kit on top would be a double allocation.
    pub(crate) fn start_arena(&mut self, class: &str) {
        self.ecs = World::default();
        self.resources = Resources::default();
        let mut rng = RandomNumberGenerator::new();
        let arena_run = ArenaRun::new(1);
        let items = roll_arena_shop_items(&mut rng, class, arena_run.template_level());
        // No starting kit to grant here (see this fn's doc comment).
        // Spawned at a throwaway position - build_shop_room moves it to
        // the real player_start once the shop map actually exists.
        let player = spawn_player(&mut self.ecs, Point::zero(), class);

        self.build_shop_room(player, &mut rng, &items);

        // Only a Battle Arena player ever gets a Gold component at all -
        // see Gold's own doc comment for why every gold codepath treats
        // that presence, not a separate mode check, as the source of
        // truth.
        let mut commands = legion::systems::CommandBuffer::new(&self.ecs);
        commands.add_component(player, Gold(ARENA_STARTING_GOLD));
        commands.flush(&mut self.ecs);

        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(None::<Battle>);
        self.resources.insert(None::<BattleVictory>);
        self.resources.insert(Keymap::load());
        self.resources.insert(BattleSpeed::load());
        self.resources.insert(AtbMode::load());
        self.resources.insert(MenuMemory::load());
        self.resources.insert(LastBattleAction::load());
        self.resources.insert(Some(arena_run));
        self.resources.insert(Some(ShoppingActive));
        self.resources.insert(None::<ShopMessage>);
        // Battle Arena never spawns a Chest, but movement_system's
        // #[resource] fetch for this is unconditional (shared by both
        // modes) - missing it here would panic the moment any Arena
        // enemy movement runs, not just a real chest interaction.
        self.resources.insert(None::<ChestLoot>);

        let mut stats = Stats::load();
        stats.record_game_started(class, AdventureMode::BattleArena);
        self.resources.insert(stats);
    }

    /// Reached by stepping on a shop's stairs tile while ArenaRun is
    /// active (see TurnState::ArenaTransition / systems/end_turn.rs) -
    /// always means "begin wave 1 of whatever level this shop was
    /// preparing you for", whether that shop was the very first one or
    /// one reached after a boss kill. One-shot, like advance_level: runs
    /// once, changes TurnState away from ArenaTransition so it doesn't
    /// repeat next frame.
    pub(crate) fn arena_transition_tick(&mut self, _ctx: &mut BTerm) {
        let run = self
            .resources
            .get::<Option<ArenaRun>>()
            .unwrap()
            .clone()
            .expect("ArenaTransition reached without an active ArenaRun");
        self.arena_begin_wave(run.level, 1);
    }

    /// Reached when systems/end_turn.rs detects the last Enemy died via a
    /// non-battle mechanism (a Throw Spear/Shoot ranged strike, or a
    /// placed Trap) while an Arena wave or boss encounter was active -
    /// see TurnState::ArenaWaveCleared's own doc comment for the full
    /// reasoning. Just reuses handle_arena_kill, the exact same
    /// orchestration the normal battle-victory dismissal path already
    /// runs - this is only a different way of REACHING that call, not a
    /// second copy of its logic.
    pub(crate) fn arena_wave_cleared_tick(&mut self) {
        let run = self
            .resources
            .get::<Option<ArenaRun>>()
            .unwrap()
            .clone()
            .expect("ArenaWaveCleared reached without an active ArenaRun");
        self.handle_arena_kill(run);
    }

    /// Removes every entity except the player and whatever they're
    /// carrying - the same entity-preservation pattern advance_level
    /// already uses to move a dungeon-crawl player to a fresh floor
    /// without losing their inventory. Shared by every Arena transition
    /// that needs a clean map but the SAME player/inventory: starting a
    /// new wave, and moving on to the next level's shop.
    pub(crate) fn arena_rebuild_keep_player(&mut self) -> Entity {
        let (player_entity, _) = find_player(&self.ecs).unwrap();

        let mut entities_to_keep = std::collections::HashSet::new();
        entities_to_keep.insert(player_entity);
        <(Entity, &Carried)>::query()
            .iter(&self.ecs)
            .filter(|(_e, carry)| carry.0 == player_entity)
            .map(|(e, _carry)| *e)
            .for_each(|e| {
                entities_to_keep.insert(e);
            });
        let mut cb = CommandBuffer::new(&mut self.ecs);
        for e in Entity::query().iter(&self.ecs) {
            if !entities_to_keep.contains(e) {
                cb.remove(*e);
            }
        }
        cb.flush(&mut self.ecs);

        player_entity
    }

    /// Builds a fresh wave arena map for `level`/`wave`, keeping the
    /// same player entity and inventory (see arena_rebuild_keep_player),
    /// spawns that wave's enemies, and forces full visibility across the
    /// map's reveal rectangle - the same "no fog of war" trick
    /// start_arena uses for the shop, applied here because the design
    /// calls for the whole arena being visible at once rather than
    /// explored tile by tile. Updates the ArenaRun resource to reflect
    /// the new wave and stores this map's boss_spawn point for later
    /// (see ArenaRun::boss_spawn's doc comment).
    pub(crate) fn arena_begin_wave(&mut self, level: u8, wave: u8) {
        let player_entity = self.arena_rebuild_keep_player();
        let mut rng = RandomNumberGenerator::new();
        let template_level = (level - 1) as usize;
        let enemy_count = ARENA_WAVE_ENEMY_COUNTS[(wave - 1) as usize] as usize;
        let (mut map_builder, enemy_spawns, boss_spawn, reveal_x, reveal_y, reveal_w, reveal_h) =
            MapBuilder::new_arena_wave(&mut rng, enemy_count);

        let mut cb = CommandBuffer::new(&mut self.ecs);
        cb.add_component(player_entity, map_builder.player_start);
        cb.flush(&mut self.ecs);

        self.reveal_and_freeze_fov(
            player_entity,
            &mut map_builder.map,
            reveal_x,
            reveal_y,
            reveal_w,
            reveal_h,
        );

        spawn_prefab_enemies(&mut self.ecs, &mut rng, template_level, &enemy_spawns);
        self.boost_arena_enemy_fov();

        self.resources.insert(map_builder.map);
        // new_bounded, not plain new - this map only reveals a smaller
        // rectangle inside the full 80x50 grid (see Camera's own bounds_*
        // field doc comment), and the camera needs to stay clamped to
        // that same rectangle or it can pan past its real edge into
        // unrevealed (black) space near the clearing's own boundary.
        self.resources.insert(Camera::new_bounded(
            map_builder.player_start,
            reveal_x,
            reveal_y,
            reveal_w,
            reveal_h,
        ));
        self.resources.insert(map_builder.theme);
        self.resources.insert(None::<Battle>);
        self.resources.insert(None::<BattleVictory>);
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(None::<ShoppingActive>);
        self.resources.insert(None::<ShopMessage>);
        self.resources.insert(Some(ArenaRun {
            level,
            wave,
            boss_active: false,
            boss_spawn,
        }));
    }

    /// Spawns this level's boss onto the SAME map wave 3 was just fought
    /// on (at the boss_spawn point that map was built with), rather than
    /// building a fresh one - the design calls for the boss appearing
    /// once the last wave clears, not a separate arena of its own.
    fn arena_spawn_boss_on_current_map(&mut self, run: ArenaRun) {
        let mut rng = RandomNumberGenerator::new();
        spawn_boss(
            &mut self.ecs,
            &mut rng,
            run.template_level(),
            run.boss_spawn,
        );
        self.boost_arena_enemy_fov();
        self.resources.insert(Some(ArenaRun {
            boss_active: true,
            ..run
        }));
        self.resources.insert(TurnState::AwaitingInput);
    }

    /// Overrides every current Enemy entity's FieldOfView with
    /// ARENA_ENEMY_FOV_RADIUS - called right after spawning a wave's
    /// enemies or a level's boss, both of which start with the small
    /// dungeon-tuned radius spawn_entity gives every enemy by default.
    /// Safe to apply indiscriminately to every Enemy in the world at
    /// that point: arena_begin_wave already cleared every non-player
    /// entity before spawning this wave's enemies, and this is called
    /// immediately after spawning, so the only Enemy entities that can
    /// possibly exist yet are the ones just spawned this call.
    fn boost_arena_enemy_fov(&mut self) {
        let mut cb = CommandBuffer::new(&mut self.ecs);
        <(Entity, &Enemy)>::query()
            .iter(&self.ecs)
            .for_each(|(e, _)| {
                cb.add_component(*e, FieldOfView::new(ARENA_ENEMY_FOV_RADIUS));
            });
        cb.flush(&mut self.ecs);
    }

    /// Moves the player on to `next_level`'s shop after clearing the
    /// previous level's boss - same player/inventory (see
    /// arena_rebuild_keep_player), a freshly rolled and stocked shop (see
    /// build_shop_room, shared with start_arena's very first shop and
    /// Dungeon Crawl's own dungeon_shop_transition).
    fn arena_advance_to_next_shop(&mut self, next_level: u8) {
        let player_entity = self.arena_rebuild_keep_player();
        let class = entity_class(&self.ecs, player_entity).unwrap_or_default();
        let mut rng = RandomNumberGenerator::new();
        let template_level = (next_level - 1) as usize;
        let items = roll_arena_shop_items(&mut rng, &class, template_level);

        self.build_shop_room(player_entity, &mut rng, &items);

        self.resources.insert(None::<Battle>);
        self.resources.insert(None::<BattleVictory>);
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(Some(ArenaRun::new(next_level)));
        self.resources.insert(Some(ShoppingActive));
        self.resources.insert(None::<ShopMessage>);
    }

    /// The single decision point for what happens after an Arena kill -
    /// called from battle.rs's battle_victory_tick when ArenaRun is
    /// active, once the player dismisses the "You defeated X!" screen.
    /// ALSO called from arena_wave_cleared_tick above, for a kill that
    /// happened outside of battle entirely - both are just different
    /// TRIGGERS for this same orchestration. Counts surviving Enemy
    /// entities directly (rather than a separate hand-maintained counter)
    /// so this can never drift out of sync with what's actually still
    /// alive on the map.
    pub(crate) fn handle_arena_kill(&mut self, run: ArenaRun) {
        let enemies_left = <&Enemy>::query().iter(&self.ecs).count();
        if enemies_left > 0 {
            // Still more to fight in this wave/boss encounter - nothing
            // to advance yet.
            self.resources.insert(TurnState::AwaitingInput);
            return;
        }

        if !run.boss_active {
            let next_wave = run.wave + 1;
            if (next_wave as usize) <= ARENA_WAVE_ENEMY_COUNTS.len() {
                self.arena_begin_wave(run.level, next_wave);
            } else {
                // Wave 3 just cleared - the boss appears now, on this
                // same map.
                self.arena_spawn_boss_on_current_map(run);
            }
        } else if run.level < 3 {
            self.arena_advance_to_next_shop(run.level + 1);
        } else {
            // Level 3's boss just fell - the whole run is won.
            self.resources.insert(TurnState::Victory);
        }
    }
}
