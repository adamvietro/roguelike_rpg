use crate::prelude::*;
use crate::State;

/// Outcome of State::dismiss_action_result - whether battle_tick should
/// keep going this frame (back to Filling for a fresh race) or stop
/// immediately because something already fully resolved the resources
/// (fled/GameOver/BattleVictory all replace `Option<Battle>` themselves).
enum ResultOutcome {
    Continue,
    EndBattleTick,
}

/// One enemy's resolved portrait info for draw_battle_arena - its Render
/// (looked up live from the ECS) and its own flash state (see
/// EnemyCombatant::flash). Bundled here rather than passed as two
/// parallel slices, which would need index-matching to stay correct.
struct EnemyPortrait {
    render: Render,
    flash: Option<(FlashKind, f32)>,
    /// This enemy's own name (for enemy_battle_glyph's row lookup) and
    /// current battle-idle frame (EnemyCombatant::battle_idle_frame) -
    /// the enemy equivalent of draw_battle_arena's player_class/
    /// player_idle_frame parameters, bundled per-portrait here since
    /// there can be more than one enemy at once.
    name: String,
    battle_idle_frame: usize,
    /// This enemy's current one-shot attack-animation glyph, if it's
    /// mid-attack this turn (EnemyCombatant::attack_animation) - checked
    /// before falling back to the tiered enemy_battle_glyph lookup below.
    attack_glyph: Option<FontCharType>,
}

/// Fractional (col, row) position, in the coarse BATTLE_PORTRAIT_COLS x
/// BATTLE_PORTRAIT_ROWS grid, for enemy #`index` of `count` total.
/// Whole-number draw_portrait can't express these - see
/// render_helpers.rs's draw_portrait_fancy/draw_wiggling_portrait, both
/// fractional for exactly this reason.
///
/// A shallow zigzag between `rows.0` (back) and `rows.1` (front),
/// alternating by index parity - a 3-enemy fight reads as a wedge
/// (back-front-back), a 2 or 4-enemy fight as a full zigzag. Index 0
/// (leftmost, closest to the player's own bottom-left spot) always
/// lands on the back row, since a front-row placement there risks
/// visually overlapping the player's portrait. `rows` comes from the
/// live theme's own `MapTheme::enemy_formation_rows` rather than one
/// shared constant - each theme's painted background has different
/// headroom before the formation clips its perimeter art (a wall, a
/// fence, a pipe band), tuned against real screenshots per theme; see
/// `MapTheme::enemy_formation_rows`'s own doc comment for the default
/// Forest uses and why it stays the conservative fallback.
///
/// Single-enemy fights are untouched (3.0, 1.0). 5+ enemies (not
/// currently a normal battle size) reuse the 4-enemy layout's rightmost
/// slot rather than a 5th position.
fn enemy_portrait_position(count: usize, index: usize, rows: (f32, f32)) -> (f32, f32) {
    if count <= 1 {
        return (3.0, 1.0);
    }
    let (row_back, row_front) = rows;
    const LEFT: f32 = 1.9;
    const RIGHT: f32 = 3.9;
    let effective_count = count.min(4);
    let effective_index = index.min(effective_count - 1);
    let step = (RIGHT - LEFT) / (effective_count - 1) as f32;
    let col = LEFT + step * effective_index as f32;
    let row = if effective_index % 2 == 0 { row_back } else { row_front };
    (col, row)
}

/// HUD_CONSOLE (col, row) for enemy #`index` (of `count`)'s own name/HP-
/// bar/ATB-bar/status text block, anchored directly BELOW that enemy's
/// own portrait (see enemy_portrait_position) instead of in one shared
/// column off to the side. A single shared column stopped making sense
/// once portraits spread across columns instead of stacking in one - it
/// used to land squarely on top of the Actions box, which sits in that
/// same horizontal territory (see the screenshot that prompted this
/// whole rework). Single-enemy keeps the exact original spot (col 96,
/// row 41) - completely unaffected by any of this.
///
/// NOTE: despite living in a variable named after HUD_CONSOLE
/// conventions elsewhere in this file, this text actually renders on
/// FINE_TEXT_CONSOLE (see battle_tick's own
/// `ctx.set_active_console(FINE_TEXT_CONSOLE)` right before this block
/// runs), NOT HUD_CONSOLE - the ratios below (32 columns and 20 rows per
/// one coarse portrait-grid unit) are FINE_TEXT_CONSOLE's own. Confirmed
/// against the original single-enemy constants
/// themselves: portrait position (3, 1) gives col 3*32=96 and row
/// (1+1)*20+1=41, exactly matching the values already proven correct -
/// console 2 is a 160x100 grid over the same 1280x800 window, so each
/// portrait-grid unit (256x160px) is exactly 32 console-2 columns and 20
/// console-2 rows, no rounding even needed. An earlier version of this
/// function used HUD_CONSOLE's own ~21.4/~13.4 ratios by mistake, which
/// put every multi-enemy text block in the wrong place entirely. Row is
/// measured from the portrait's own BOTTOM edge (position + 1.0, since
/// every portrait is exactly one grid unit tall regardless of its
/// fractional top-left anchor), plus a 1-row margin so text starts just
/// past the sprite rather than flush against it.
fn enemy_text_position(count: usize, index: usize, rows: (f32, f32)) -> (i32, i32) {
    if count <= 1 {
        return (96, 41);
    }
    let (col, row) = enemy_portrait_position(count, index, rows);
    let console2_col = (col * 32.0).round() as i32;
    let console2_row = ((row + 1.0) * 20.0).round() as i32 + 1;
    (console2_col, console2_row)
}

/// BIG_TEXT_CONSOLE (col, row) to center a floating damage number over
/// enemy #`index` (of `count`) - see the floating-damage-number block in
/// battle_tick. Derived the same way the original single-enemy constants
/// (28, 7 - for portrait position (3, 1)) were: BIG_TEXT_CONSOLE and the
/// portrait console cover the same physical window, at ratios of 8
/// BIG_TEXT columns and 5 BIG_TEXT rows per one portrait-grid unit,
/// centered half a unit into whichever cell the portrait occupies.
fn enemy_damage_popup_position(count: usize, index: usize, rows: (f32, f32)) -> (i32, i32) {
    let (col, row) = enemy_portrait_position(count, index, rows);
    let big_col = (col * 8.0 + 4.0).round() as i32;
    let big_row = (row * 5.0 + 2.0).round() as i32;
    (big_col, big_row)
}

/// A natural-language join of names for the victory message - "the Goblin!",
/// "the Goblin and the Orc!", "the Goblin, the Orc, and the Rat!". Every
/// name already includes its own "the " (see how `enemy_names` is built
/// in record_enemy_kill) - this just handles the comma/and joinery.
fn join_enemy_names(names: &[String]) -> String {
    match names.len() {
        0 => String::new(),
        1 => names[0].clone(),
        2 => format!("{} and {}", names[0], names[1]),
        _ => {
            let (last, rest) = names.split_last().unwrap();
            format!("{}, and {}", rest.join(", "), last)
        }
    }
}

impl State {
    /// Draws the battle arena background (console 0: the current theme's
    /// floor/walls/scenery) and every combatant portrait (console 3) -
    /// the player plus up to MAX_BATTLE_ENEMIES enemies. Callers look up
    /// each Render live during an active battle, or pass captured values
    /// (battle_victory_tick, where every enemy has already been removed
    /// from the ECS - it always passes an empty `enemies` slice).
    /// `player_class` is the player's class name whenever it's known
    /// (both live battle and the Victory screen have a real player to
    /// ask - only truly missing for something with no Class component at
    /// all). `player_idle_frame` is additionally `Some(frame_index)`
    /// only for a LIVE battle-idle animation (called from battle_tick,
    /// with a real Battle in progress ticking Battle::player_idle_frame)
    /// - `None` for the Victory screen's own call
    /// (screens/battle.rs's battle_victory_tick), which has no ongoing
    /// Battle to read a frame counter off.
    ///
    /// Three-tier fallback for the player's own portrait, checked in
    /// order: character_battle.png's animated Fight_Stance loop (only
    /// when player_idle_frame is Some AND the class has a row there) ->
    /// character_battle.png's still portrait (same class check, no frame
    /// needed) -> the old plain dungeonfont Render glyph. A class with
    /// neither per-class sheet row still gets something reasonable
    /// (its old dungeonfont glyph) rather than nothing.
    ///
    /// `technique_glyph`/`victory_glyph` each override that whole 3-tier
    /// fallback when Some - a live battle passes
    /// `battle.player_action_animation`'s current glyph (see
    /// Battle::player_action_animation's own doc comment) for the
    /// duration of an Attack/Defend/Technique's own ActionResult, and
    /// battle_victory_tick passes BattleVictory::portrait_animation's
    /// once the fight is won. Never both Some for the same call - an
    /// action animation only ever plays during a live battle, a victory
    /// animation only once the battle is already over. `action_kind`
    /// (see Battle::player_action_kind) says which of the three separate
    /// sheets/consoles `technique_glyph`'s index resolves against - only
    /// meaningful when `technique_glyph` is Some, ignored otherwise (the
    /// Victory screen's own call always passes `None` for both).
    fn draw_battle_arena(
        &mut self,
        enemies: &[EnemyPortrait],
        dying_effects: &[DyingEnemyEffect],
        player_render: Option<Render>,
        player_flash: Option<(FlashKind, f32)>,
        player_class: Option<String>,
        player_idle_frame: Option<usize>,
        technique_glyph: Option<FontCharType>,
        action_kind: Option<PlayerActionKind>,
        victory_glyph: Option<FontCharType>,
    ) {
        // --- Arena background. Two paths, chosen per the live dungeon
        // theme's MapTheme::battle_background_row:
        //
        // Real art (Some(row)): the theme's one full painted scene from
        // resources/battle_backgrounds.png, drawn as a single glyph on
        // BATTLE_BACKDROP_CONSOLE (a 1x1-cell console stretched to fill
        // the entire window - see its own doc comment in main.rs).
        // Fully opaque, so it needs nothing else drawn under it.
        //
        // No real art yet (None): the original procedural fallback -
        // the theme's floor/wall tiles, tinted with its palette and
        // framed with a border, plus a soft vignette that brightens
        // toward the center (a "clearing") and darkens toward the edges,
        // drawn cell-by-cell onto console 0 (otherwise blank during
        // battle, so free real estate). Cell backgrounds (not just the
        // thin foreground glyph) carry the tint, since a small character
        // like '.' or ';' only covers a fraction of a cell's pixels -
        // foreground-only color reads as scattered specks on black
        // rather than an actual colored floor.
        {
            let theme = self.resources.get::<Box<dyn MapTheme>>().unwrap();
            let background_row = theme.battle_background_row();
            let floor_glyph = theme.tile_to_render(TileType::Floor);
            let wall_glyph = theme.tile_to_render(TileType::Wall);
            let floor_base = theme.floor_color();
            let wall_base = theme.wall_color();
            let scenery = theme.battle_scenery();
            drop(theme);

            if let Some(row) = background_row {
                // fg WHITE so the real art shows through untinted;
                // bg BLACK (not WHITE) as a deliberate defense-in-depth
                // fallback - CONSOLE_WITH_BG_FS falls back to this exact
                // per-vertex bg color for any texture pixel whose RGB is
                // all <=0.1 (~25/255) or whose alpha isn't fully opaque
                // (see the glyph-32-adjacent near-black-pixel gotcha in
                // CLAUDE.md). The real fix is flooring every near-black
                // pixel in resources/battle_backgrounds.png to >=30/
                // channel so that fallback should never actually trigger
                // for real content - BLACK just means a pixel that
                // somehow still slips through blends into a dark scene
                // instead of standing out as a stark white fleck the way
                // an earlier, unfloored version of this art did.
                let mut backdrop = DrawBatch::new();
                backdrop.target(BATTLE_BACKDROP_CONSOLE);
                backdrop.set(Point::new(0, 0), ColorPair::new(WHITE, BLACK), row as FontCharType);
                backdrop.submit(0).expect("Batch error");
            } else {
                let mut arena = DrawBatch::new();
                arena.target(0);
                for y in 0..DISPLAY_HEIGHT {
                    for x in 0..DISPLAY_WIDTH {
                        let is_border =
                            x == 0 || y == 0 || x == DISPLAY_WIDTH - 1 || y == DISPLAY_HEIGHT - 1;
                        let (glyph, base) = if is_border {
                            (wall_glyph, wall_base)
                        } else {
                            (floor_glyph, floor_base)
                        };
                        let bg = vignette(base, x, y, DISPLAY_WIDTH, DISPLAY_HEIGHT);
                        let fg = RGB::from_f32(
                            (bg.r * 1.4).min(1.0),
                            (bg.g * 1.4).min(1.0),
                            (bg.b * 1.4).min(1.0),
                        );
                        arena.set(Point::new(x, y), ColorPair::new(fg, bg), glyph);
                    }
                }

                match scenery {
                    BattleScenery::ScatteredTrees => {
                        // A handful of large tree/foliage silhouettes,
                        // hand-placed clear of the portraits, their labels,
                        // and the message/menu panel. Drawn at full-strength
                        // color (not vignetted) so they read as distinct
                        // features wherever they land on the light/dark
                        // gradient above.
                        let canopy_color = RGB::from_f32(
                            (floor_base.r * 1.5).min(1.0),
                            (floor_base.g * 1.5).min(1.0),
                            (floor_base.b * 1.5).min(1.0),
                        );
                        let trunk_color = RGB::from_f32(
                            wall_base.r * 0.85,
                            wall_base.g * 0.85,
                            wall_base.b * 0.85,
                        );
                        for &(tx, ty) in &[(6, 3), (18, 3), (35, 15), (22, 19)] {
                            draw_tree(&mut arena, wall_glyph, canopy_color, trunk_color, tx, ty);
                        }
                    }
                    BattleScenery::RoomWalls => {
                        // Thick stone walls down the left/right sides, so the
                        // arena reads as an enclosed room rather than open
                        // ground. Full-strength color (not vignetted) - these
                        // are structural, not lighting, so they stay solid
                        // regardless of the floor's center-lit gradient.
                        const SIDE_WALL_THICKNESS: i32 = 4;
                        let fg = RGB::from_f32(
                            (wall_base.r * 1.4).min(1.0),
                            (wall_base.g * 1.4).min(1.0),
                            (wall_base.b * 1.4).min(1.0),
                        );
                        let wall_color_pair = ColorPair::new(fg, wall_base);
                        for y in 0..DISPLAY_HEIGHT {
                            for x in 0..SIDE_WALL_THICKNESS {
                                arena.set(Point::new(x, y), wall_color_pair, wall_glyph);
                                let rx = DISPLAY_WIDTH - 1 - x;
                                arena.set(Point::new(rx, y), wall_color_pair, wall_glyph);
                            }
                        }
                    }
                }

                arena.submit(0).expect("Batch error");
            }
        }

        // --- Portraits: each creature's own glyph, drawn once on the
        // coarse BATTLE_PORTRAIT_COLS x BATTLE_PORTRAIT_ROWS console, so it
        // renders far larger than its normal dungeon-map size. Enemies
        // stack down column 3 (see enemy_portrait_row); player sits
        // bottom-left. Whichever side is currently "Attacking" gets a
        // small shake instead of the plain draw - see
        // draw_wiggling_portrait below.
        let mut portraits = DrawBatch::new();
        portraits.target(BATTLE_PORTRAIT_CONSOLE);
        let mut wiggle = DrawBatch::new();
        wiggle.target(BATTLE_PORTRAIT_WIGGLE_CONSOLE);
        // Two-tier fallback for each enemy's own portrait, mirroring the
        // player's own tiered lookup below: enemy_battle.png's animated
        // Fight_Stance loop (see components::enemy_battle_glyph) when
        // this enemy's name has a row there, else the old plain
        // dungeonfont Render glyph - an enemy with no row there still
        // gets something reasonable rather than nothing.
        let mut enemy_battle_idle = DrawBatch::new();
        enemy_battle_idle.target(ENEMY_BATTLE_CONSOLE);
        let mut enemy_battle_wiggle = DrawBatch::new();
        enemy_battle_wiggle.target(ENEMY_BATTLE_WIGGLE_CONSOLE);
        // A new top tier, checked before the two above: enemy_attack.png's
        // played-once Attack animation (EnemyPortrait::attack_glyph, set
        // whenever this enemy's most recent action has a row on
        // components::attack_animation_for_enemy), for the duration of
        // its own ActionResult. Deliberately never wiggled, same reasoning
        // as the player's own technique/attack/defend tiers above - the
        // animation already shows real motion. Drawn via draw_portrait_
        // fancy (not the plain whole-cell draw_portrait) so a 2+ enemy
        // fight's fractional zigzag position (enemy_portrait_position)
        // still lines up correctly during the attack, same reason the
        // existing enemy_battle_wiggle tier below needs a fancy console
        // for its own >1-enemy case - registered as fancy in main.rs even
        // though nothing here ever applies an actual wiggle offset to it.
        let mut enemy_attack = DrawBatch::new();
        enemy_attack.target(ENEMY_ATTACK_CONSOLE);
        // Any boss corpses still playing their death animation (see
        // Battle::dying_effects) - drawn at their own frozen snapshot
        // position/color from the moment they died, entirely independent
        // of the live `enemies`/`enemy_count` formation loop below.
        let mut enemy_death = DrawBatch::new();
        enemy_death.target(ENEMY_DEATH_CONSOLE);
        for effect in dying_effects {
            let tinted = Render {
                color: effect.color,
                glyph: effect.animation.current_glyph(),
            };
            draw_portrait_fancy(&mut enemy_death, effect.col, effect.row, tinted);
        }
        let enemy_count = enemies.len();
        // See MapTheme::enemy_formation_rows' own doc comment - how far
        // up the screen this theme's own battle backdrop art lets the
        // zigzag formation reach before overlapping its perimeter
        // scenery.
        let formation_rows = self
            .resources
            .get::<Box<dyn MapTheme>>()
            .unwrap()
            .enemy_formation_rows();
        for (index, enemy) in enemies.iter().enumerate() {
            let (col, row) = enemy_portrait_position(enemy_count, index, formation_rows);
            let color = flash_tint(enemy.render.color, enemy.flash);
            if let Some(glyph) = enemy.attack_glyph {
                let tinted = Render { color, glyph };
                draw_portrait_fancy(&mut enemy_attack, col, row, tinted);
                continue;
            }
            match enemy_battle_glyph(&enemy.name, enemy.battle_idle_frame) {
                Some(glyph) => {
                    let tinted = Render { color, glyph };
                    if !draw_wiggling_portrait(&mut enemy_battle_wiggle, col, row, tinted, enemy.flash)
                    {
                        if enemy_count <= 1 {
                            draw_portrait(&mut enemy_battle_idle, col as i32, row as i32, tinted);
                        } else {
                            draw_portrait_fancy(&mut enemy_battle_wiggle, col, row, tinted);
                        }
                    }
                }
                None => {
                    let tinted = Render {
                        color,
                        glyph: enemy.render.glyph,
                    };
                    if !draw_wiggling_portrait(&mut wiggle, col, row, tinted, enemy.flash) {
                        if enemy_count <= 1 {
                            // Single enemy - the exact original whole-cell
                            // draw, unchanged, on console 3 like it always
                            // has been.
                            draw_portrait(&mut portraits, col as i32, row as i32, tinted);
                        } else {
                            // 2+ enemies - fractional position (see
                            // enemy_portrait_position), which the
                            // whole-cell-only draw_portrait can't express,
                            // so this goes through the fancy console
                            // instead even while idle.
                            draw_portrait_fancy(&mut wiggle, col, row, tinted);
                        }
                    }
                }
            }
        }
        // The player's own portrait - see this fn's own doc comment for
        // the 3-tier fallback. Each tier gets its own console pair so it
        // never has to share a font with the dungeonfont-sourced enemy
        // portraits above (or with each other).
        let mut battle_idle = DrawBatch::new();
        battle_idle.target(CHARACTER_BATTLE_CONSOLE);
        let mut battle_idle_wiggle = DrawBatch::new();
        battle_idle_wiggle.target(CHARACTER_BATTLE_WIGGLE_CONSOLE);
        let mut still_portrait = DrawBatch::new();
        still_portrait.target(CHARACTER_PORTRAIT_BIG_CONSOLE);
        let mut technique = DrawBatch::new();
        technique.target(CHARACTER_TECHNIQUE_CONSOLE);
        let mut attack = DrawBatch::new();
        attack.target(CHARACTER_ATTACK_CONSOLE);
        let mut defend = DrawBatch::new();
        defend.target(CHARACTER_DEFEND_CONSOLE);
        let mut victory_portrait = DrawBatch::new();
        victory_portrait.target(CHARACTER_VICTORY_CONSOLE);
        if let Some(render) = player_render {
            let color = flash_tint(render.color, player_flash);
            let class_ref = player_class.as_deref();
            let battle_glyph = player_idle_frame
                .and_then(|frame| class_ref.and_then(|class| character_battle_glyph(class, frame)));
            let portrait_glyph = class_ref.and_then(character_portrait_glyph);
            match (technique_glyph, victory_glyph, battle_glyph, portrait_glyph) {
                (Some(glyph), _, _, _) => {
                    // Deliberately never wiggled, even during an
                    // "Attacking" flash - the action's own animation
                    // already shows real motion, so stacking the shake
                    // on top of it read as redundant/busy (explicit user
                    // feedback, 2026-09-08). Every other tier still gets
                    // the wiggle. Routed to whichever of the three
                    // separate sheets/consoles this glyph index actually
                    // belongs to (see Battle::player_action_kind) -
                    // Technique is the fallback for the (should be
                    // unreachable, since the two are always set together)
                    // case action_kind is somehow still None here.
                    let tinted = Render { color, glyph };
                    match action_kind {
                        Some(PlayerActionKind::Attack) => draw_portrait(&mut attack, 1, 3, tinted),
                        Some(PlayerActionKind::Defend) => draw_portrait(&mut defend, 1, 3, tinted),
                        Some(PlayerActionKind::Technique) | None => {
                            draw_portrait(&mut technique, 1, 3, tinted)
                        }
                    }
                }
                (None, Some(glyph), _, _) => {
                    let tinted = Render { color, glyph };
                    draw_portrait(&mut victory_portrait, 1, 3, tinted);
                }
                (None, None, Some(glyph), _) => {
                    let tinted = Render { color, glyph };
                    if !draw_wiggling_portrait(&mut battle_idle_wiggle, 1.0, 3.0, tinted, player_flash)
                    {
                        draw_portrait(&mut battle_idle, 1, 3, tinted);
                    }
                }
                (None, None, None, Some(glyph)) => {
                    let tinted = Render { color, glyph };
                    if !draw_wiggling_portrait(&mut still_portrait, 1.0, 3.0, tinted, player_flash) {
                        draw_portrait(&mut still_portrait, 1, 3, tinted);
                    }
                }
                (None, None, None, None) => {
                    let tinted = Render {
                        color,
                        glyph: render.glyph,
                    };
                    if !draw_wiggling_portrait(&mut wiggle, 1.0, 3.0, tinted, player_flash) {
                        draw_portrait(&mut portraits, 1, 3, tinted);
                    }
                }
            }
        }
        portraits.submit(0).expect("Batch error");
        wiggle.submit(1).expect("Batch error");
        battle_idle.submit(2).expect("Batch error");
        battle_idle_wiggle.submit(3).expect("Batch error");
        still_portrait.submit(4).expect("Batch error");
        enemy_battle_idle.submit(5).expect("Batch error");
        enemy_battle_wiggle.submit(6).expect("Batch error");
        technique.submit(7).expect("Batch error");
        victory_portrait.submit(8).expect("Batch error");
        attack.submit(9).expect("Batch error");
        defend.submit(10).expect("Batch error");
        enemy_attack.submit(11).expect("Batch error");
        enemy_death.submit(12).expect("Batch error");
    }

    /// Records one enemy's death: stats, loot/gold (accumulated onto
    /// `battle` - see its gold_earned/loot_found/defeated_names fields),
    /// removes it from the ECS and from `battle.enemies`. If that was the
    /// last enemy in the fight, finishes the whole battle (BattleVictory)
    /// and returns true - the caller should stop touching `battle`
    /// immediately, same contract the old finish_battle_victory had.
    /// Returns false if the fight continues (other enemies remain).
    /// `enter_held` is whether Enter is the key currently down this frame
    /// (see battle_tick) - threaded all the way down here purely so
    /// finish_battle can arm pending_enter_release on the exact frame a
    /// kill actually transitions into BattleVictory, regardless of which
    /// of several call paths (a plain Attack, a Technique, a DoT tick, a
    /// Counter) led to this kill.
    fn record_enemy_kill(&mut self, battle: &mut Battle, target: Entity, enter_held: bool) -> bool {
        let target_name = battle
            .enemy(target)
            .map(|e| e.name.clone())
            .unwrap_or_else(|| "enemy".to_string());

        if let Some(class) = entity_class(&self.ecs, battle.player) {
            if let Some(mut stats) = self.resources.get_mut::<Stats>() {
                stats.record_enemy_killed(&class);
            }
        }

        // Read the enemy's Boss tag and the player's current Gold total
        // BEFORE the enemy is removed below - Gold's own presence (not a
        // separate Option<ArenaRun> check) is what decides whether this
        // is a Battle Arena kill at all, per that component's own doc
        // comment. Loot is rolled here too (before the CommandBuffer
        // below exists) since grant_random_battle_loot also needs
        // `&mut self.ecs` directly - same ordering finish_battle_victory
        // always used, for the same reason.
        let is_boss = self
            .ecs
            .entry_ref(target)
            .map(|e| e.get_component::<Boss>().is_ok())
            .unwrap_or(false);
        let current_gold = self
            .ecs
            .entry_ref(battle.player)
            .ok()
            .and_then(|e| e.get_component::<Gold>().ok().copied());

        // In the Battle Arena, a kill's reward is gold ONLY - the old
        // random ability-drop loot is deliberately not granted alongside
        // it. A Dungeon Crawl kill still rolls for loot exactly as
        // before - checked via Option<ArenaRun>, NOT Gold presence, since
        // Dungeon Crawl players now carry a Gold component too (see its
        // own doc comment) and would otherwise always skip this roll.
        let is_arena_run = self.resources.get::<Option<ArenaRun>>().unwrap().is_some();
        let mut rng = RandomNumberGenerator::new();
        let loot = if is_arena_run {
            None
        } else {
            grant_random_battle_loot(&mut self.ecs, &mut rng, battle.player, target)
        };
        if let Some(item) = loot {
            battle.loot_found.push(item);
        }

        // A boss's own death animation (see components::
        // death_animation_for_enemy - `None` for a basic enemy, which
        // still just vanishes below exactly as before) - captured here,
        // BEFORE removal, as a pure decorative Battle::dying_effects
        // entry rather than delaying anything below it. See that field's
        // own doc comment for why this doesn't block rewards/removal/the
        // fight-over check the way keeping the enemy "alive" in
        // `battle.enemies` until the animation finished would have.
        if let Some(animation) = death_animation_for_enemy(&target_name) {
            let color = entity_render_component(&self.ecs, target)
                .map(|r| r.color)
                .unwrap_or(ColorPair::new(WHITE, BLACK));
            let index = battle.enemies.iter().position(|e| e.entity == target);
            if let Some(index) = index {
                let formation_rows = self
                    .resources
                    .get::<Box<dyn MapTheme>>()
                    .unwrap()
                    .enemy_formation_rows();
                let (col, row) =
                    enemy_portrait_position(battle.enemies.len(), index, formation_rows);
                battle.dying_effects.push(DyingEnemyEffect {
                    col,
                    row,
                    color,
                    animation,
                });
            }
        }

        let mut cb = CommandBuffer::new(&mut self.ecs);
        if let Some(Gold(amount)) = current_gold {
            let reward = gold_reward_for_kill(is_boss);
            cb.add_component(battle.player, Gold(amount + reward));
            battle.gold_earned += reward;
        }
        cb.remove(target);
        cb.flush(&mut self.ecs);

        battle.push_log(format!("Defeated the {}!", target_name));
        battle.defeated_names.push(format!("the {}", target_name));
        battle.enemies.retain(|e| e.entity != target);

        if battle.enemies.is_empty() {
            self.finish_battle(battle, enter_held);
            true
        } else {
            false
        }
    }

    /// Ends the whole battle in victory, using whatever's accumulated on
    /// `battle` across every kill this fight (see record_enemy_kill) -
    /// called the instant `battle.enemies` becomes empty. `enter_held` -
    /// see this same parameter's doc comment on record_enemy_kill/
    /// battle_tick - arms pending_enter_release when true, so the exact
    /// same held Enter that landed this killing blow can't ALSO
    /// immediately dismiss the Victory screen this transitions into.
    fn finish_battle(&mut self, battle: &Battle, enter_held: bool) {
        // Checked fresh here (not just "was gold_earned > 0") so a
        // Battle Arena fight that somehow ended with 0 net gold (it
        // shouldn't - every kill grants at least a few) still correctly
        // shows "0 gold" rather than being mistaken for a Dungeon Crawl
        // fight with no loot. Uses Option<ArenaRun>, NOT Gold presence -
        // Dungeon Crawl players carry Gold too now (see its own doc
        // comment), so that check would otherwise always read "arena"
        // and hide this fight's real ability-loot find behind a
        // "gold_earned" the Victory screen never announces for Dungeon
        // Crawl anyway (see match arm below).
        let is_arena = self.resources.get::<Option<ArenaRun>>().unwrap().is_some();

        let portrait_animation = entity_class(&self.ecs, battle.player)
            .and_then(|class| victory_animation_for_class(&class));
        self.resources.insert(Some(BattleVictory {
            player: battle.player,
            enemy_names: battle.defeated_names.clone(),
            loot: battle.loot_found.clone(),
            gold_earned: if is_arena {
                Some(battle.gold_earned)
            } else {
                None
            },
            portrait_animation,
        }));
        self.resources.insert(None::<Battle>);
        self.resources.insert(TurnState::BattleVictory);
        // See this fn's own doc comment - guards against the same held
        // Enter that fired the killing action also immediately dismissing
        // the screen it just transitioned into.
        if enter_held {
            self.pending_enter_release = true;
        }
    }

    /// Fires `attacker`'s attack right now: ticks its own active Dot
    /// first (a lingering wound, not an action in its own right), checks
    /// whether that alone finished it off, then actually resolves the
    /// attack and enters ActionResult(Enemy(attacker)). Shared by two
    /// triggers: the normal "this enemy's gauge just reached
    /// ATB_GAUGE_MAX while nothing else was in progress" case (Filling),
    /// and - under True ATB (AtbMode::Active) only - an enemy's gauge
    /// filling WHILE the player is still stuck in their own menu (see
    /// BattleTurn::PlayerMenu's handling below) - "decide fast or take
    /// the hit" is the entire point of that mode. Returns true if
    /// `attacker` died to its own Dot tick before it could even swing (or
    /// that was the last enemy standing) - record_enemy_kill has already
    /// been called in that case, possibly ending the whole battle; the
    /// caller should return from battle_tick immediately without
    /// touching `battle` again.
    fn trigger_enemy_action(
        &mut self,
        battle: &mut Battle,
        attacker: Entity,
        enter_held: bool,
    ) -> bool {
        let dot_message = tick_dot(&mut self.ecs, battle, attacker);
        if let Some(dot_message) = dot_message {
            battle.push_log(dot_message);
        }

        let (hp_now, _) = entity_health(&self.ecs, attacker);
        if hp_now < 1 {
            return self.record_enemy_kill(battle, attacker, enter_held);
        }

        if let Some(enemy) = battle.enemy_mut(attacker) {
            enemy.attack_animation = attack_animation_for_enemy(&enemy.name);
        }
        resolve_enemy_attack(&mut self.ecs, battle, attacker);
        battle.enter_result(Combatant::Enemy(attacker));
        false
    }

    /// Resolves a player-chosen BattleAction: applies its effect, logs
    /// the result, clears the one-shot sneak_attack bonus, and enters
    /// ActionResult(Player). Shared by two callers: PlayerMenu's own
    /// immediate resolution (the common case - nothing else was
    /// happening) and Filling's queued-action resolution (True ATB only
    /// - see Battle::queued_player_action) for a choice made earlier,
    /// while some enemy's own result was still on screen. Both need the
    /// exact same effect logic; only WHEN it runs differs.
    ///
    /// Single-target actions (Attack, and every single-target Technique)
    /// hit whichever enemy Battle::primary_target currently picks - the
    /// player doesn't choose directly once there's more than one enemy.
    /// Recomputed fresh here rather than passed in, so a target chosen
    /// via queuing is still whoever's actually fastest-and-alive at the
    /// moment this finally runs, not whoever was fastest back when the
    /// player originally pressed the key.
    fn resolve_player_action(&mut self, battle: &mut Battle, chosen: BattleAction) {
        let target = battle.primary_target(&self.ecs);
        // Captured before the match below - BattleAction::Technique's own
        // branch removes the item entity from the ECS as part of
        // consuming it (via apply_player_technique), so its Name has to
        // be read before that happens, not after. Used only for
        // cross-battle cursor memory (MenuMemory) once resolution
        // finishes below - a separate, earlier read of the same
        // information Stats::record_ability_used's own inline capture
        // needs for the same reason.
        let action_name_for_memory = action_name(&self.ecs, chosen);
        // Every technique's mechanical effect is resolved in one place
        // (battle::apply_player_technique) rather than a match arm per
        // item here - adding a new class's technique needs no change here.
        match chosen {
            BattleAction::Attack => {
                if let Some(class) = entity_class(&self.ecs, battle.player) {
                    let anim = attack_animation_for_class(&class);
                    battle.player_action_kind =
                        anim.is_some().then(|| PlayerActionKind::Attack);
                    battle.player_action_animation = anim;
                }
                if let Some(target) = target {
                    let mut dmg = player_attack_damage(&self.ecs, battle.player);
                    if battle.sneak_attack {
                        dmg *= 3;
                    }
                    let dmg = damage::strike_enemy(&mut self.ecs, battle, target, dmg);
                    battle.push_log(damage::strike_message(dmg));
                }
            }
            BattleAction::Defend => {
                if let Some(class) = entity_class(&self.ecs, battle.player) {
                    let anim = defend_animation_for_class(&class);
                    battle.player_action_kind =
                        anim.is_some().then(|| PlayerActionKind::Defend);
                    battle.player_action_animation = anim;
                }
                battle.player_defending = true;
                battle.push_log("Defend.".to_string());
            }
            BattleAction::Flee => {
                battle.fled = true;
                battle.push_log("Flee.".to_string());
            }
            BattleAction::Technique(item) => {
                // Recorded BEFORE apply_player_technique runs - it
                // removes `item` from the ECS as part of consuming it,
                // so its Name/Class have to be read while it's still
                // there. Keyed off the item's own class, same reasoning
                // as Stats::record_ability_used's doc comment.
                if let Some(class) = entity_class(&self.ecs, item) {
                    let name = entity_name(&self.ecs, item);
                    if let Some(mut stats) = self.resources.get_mut::<Stats>() {
                        stats.record_ability_used(&class, &name);
                    }
                    // A real animation for this specific (class,
                    // technique) pair, if one exists yet (see
                    // components::technique_animation_for) - None leaves
                    // draw_battle_arena showing the ordinary
                    // Fight_Stance_Idle loop instead, same as before this
                    // existed. Cleared in dismiss_action_result once turn
                    // returns to Filling. A multi-hit/AOE technique's own
                    // HitQueue (battle::damage) keeps landing damage over
                    // a real span of time far longer than one play-
                    // through of the animation, so it loops instead of
                    // holding on its last frame for most of that span -
                    // see OneShotAnimation::repeat's own doc comment.
                    let repeats = matches!(
                        technique_effect(&self.ecs, item),
                        Some(TechniqueEffect::MultiHit(_))
                            | Some(TechniqueEffect::AoeMultiHit { .. })
                    );
                    let anim = technique_animation_for(&class, &name, repeats);
                    battle.player_action_kind =
                        anim.is_some().then(|| PlayerActionKind::Technique);
                    battle.player_action_animation = anim;
                }
                // A self-buff technique (Heal/Evade/WarCry/Counter) just
                // ignores `target` entirely inside apply_player_technique
                // - still needs SOME entity to satisfy the signature, so
                // this falls back to the player itself in the (should be
                // unreachable - see primary_target's own doc comment)
                // case there's no enemy to target at all.
                let result = apply_player_technique(
                    &mut self.ecs,
                    battle,
                    item,
                    target.unwrap_or(battle.player),
                );
                battle.push_log(result);
            }
        }
        // Sneak attack is a one-shot ambush bonus for the guaranteed
        // first action only - clear it here regardless of which action
        // was actually chosen, so it can never linger and apply again
        // later in the same fight.
        battle.sneak_attack = false;

        // Cross-battle cursor memory (MenuMemory) - remember whatever was
        // just chosen, per class, so a FUTURE battle's cursor can start
        // there (see MenuCursor's seeding in battle_tick above). Recorded
        // here rather than at each individual call site (PlayerMenu's
        // direct resolution AND Filling's queued-action resolution both
        // funnel through this one function) so neither path can forget
        // it. Uses action_name_for_memory, captured above BEFORE the
        // match ran (see that binding's own comment for why).
        if *self.resources.get::<MenuMemory>().unwrap() == MenuMemory::On {
            if let Some(class) = entity_class(&self.ecs, battle.player) {
                let mut memory = self.resources.get_mut::<LastBattleAction>().unwrap();
                memory.record(&class, &action_name_for_memory);
                let saved = memory.clone();
                drop(memory);
                saved.save();
            }
        }

        battle.enter_result(Combatant::Player);
    }

    /// Outcome of dismiss_action_result - see that function. `enter_held`
    /// - see battle_tick's own doc note on this parameter - is threaded
    /// through so BOTH exit paths that can lead somewhere Enter might
    /// also dismiss (GameOver here directly; BattleVictory via
    /// record_enemy_kill/finish_battle) can arm pending_enter_release.
    fn dismiss_action_result(&mut self, battle: &mut Battle, enter_held: bool) -> ResultOutcome {
        if battle.fled {
            self.resources.insert(None::<Battle>);
            self.resources.insert(TurnState::AwaitingInput);
            return ResultOutcome::EndBattleTick;
        }

        let (player_hp_now, _) = entity_health(&self.ecs, battle.player);
        if player_hp_now < 1 {
            self.resources.insert(None::<Battle>);
            self.resources.insert(TurnState::GameOver);
            // Same guard as finish_battle - the player's own held Enter
            // (from an earlier action, or from an enemy's ActionResult
            // dismiss) shouldn't also instantly dismiss Game Over.
            if enter_held {
                self.pending_enter_release = true;
            }
            return ResultOutcome::EndBattleTick;
        }

        // Sweep every remaining enemy for death rather than checking just
        // one specific entity - a Counter Attack can kill the enemy that
        // just attacked as a side effect of ITS OWN attack, the player's
        // own action might have killed whichever enemy it targeted, and
        // (once AOE exists) more than one could die from the same action
        // at once. record_enemy_kill on one enemy can end the whole
        // battle (all enemies gone) - stop immediately if so, same
        // contract trigger_enemy_action's own caller already follows.
        let dead: Vec<Entity> = battle
            .enemies
            .iter()
            .filter(|e| entity_health(&self.ecs, e.entity).0 < 1)
            .map(|e| e.entity)
            .collect();
        for target in dead {
            if self.record_enemy_kill(battle, target, enter_held) {
                return ResultOutcome::EndBattleTick;
            }
        }

        battle.turn = BattleTurn::Filling;
        // Whatever action animation (Attack/Defend/Technique) was playing
        // for the action just dismissed is done - clear it so
        // draw_battle_arena falls back to the ordinary Fight_Stance_Idle
        // loop for the next race, rather than holding on its last frame
        // indefinitely. Same for every enemy's own attack animation.
        //
        // player_flash/enemy.flash are cleared here too, not just left to
        // decay on their own timer - a real bug found 2026-09-13: this
        // fires the instant ANY key is pressed (see battle_tick's own
        // `ctx.key.is_some() || battle.result_timer_ms <= 0.0` check), not
        // only once flash_ms's duration (deliberately synced to the
        // animation's own length - see strike_enemy/strike_player) has
        // actually run out. Dismissing early cleared the animation but
        // left an active Attacking flash behind with real time still on
        // it; the very next frame's draw_battle_arena call then saw NO
        // animation glyph (falling through to the ordinary idle-portrait
        // tier) but STILL an active Attacking flash, and that tier - unlike
        // the animation tier - applies the old pre-real-animation wiggle
        // for exactly that flash. Result: a brief, leftover wiggle on the
        // idle portrait right after dismissing an action, reproducible
        // only when a key happens to be pressed before the flash would
        // have decayed on its own - which is why it never fired
        // consistently. Clearing both together here keeps them in sync
        // regardless of how (or how early) the result was dismissed.
        battle.player_action_animation = None;
        battle.player_action_kind = None;
        battle.player_flash = None;
        for enemy in battle.enemies.iter_mut() {
            enemy.attack_animation = None;
            enemy.flash = None;
        }
        ResultOutcome::Continue
    }

    /// Called from main.rs's tick() dispatcher, so this needs to be `pub`.
    pub fn battle_tick(&mut self, ctx: &mut BTerm) {
        let battle_snapshot = self.resources.get::<Option<Battle>>().unwrap().clone();
        let mut battle = match battle_snapshot {
            Some(b) => b,
            None => {
                // Shouldn't happen, but don't get stuck if it does.
                self.resources.insert(TurnState::AwaitingInput);
                return;
            }
        };

        // Whether Enter is the key physically down THIS frame - computed
        // once here and threaded into every call this frame that could
        // transition into a screen Enter also dismisses (GameOver,
        // BattleVictory), so that transition can arm
        // pending_enter_release regardless of which of several call
        // paths (a plain kill, a Counter kill, a DoT kill, the player's
        // own death) actually triggered it. See pending_enter_release's
        // own doc comment on State for the full mechanism.
        let enter_held = ctx.key == Some(VirtualKeyCode::Return);

        // Tick down any active post-action portrait flash/damage popup -
        // the player's own (still flat fields on Battle) and every
        // enemy's own (now on EnemyCombatant - see that struct's doc
        // comment).
        if let Some((_, remaining)) = &mut battle.player_flash {
            *remaining -= ctx.frame_time_ms;
            if *remaining <= 0.0 {
                battle.player_flash = None;
            }
        }
        if let Some(popup) = &mut battle.player_damage_popup {
            popup.remaining_ms -= ctx.frame_time_ms;
            if popup.remaining_ms <= 0.0 {
                battle.player_damage_popup = None;
            }
        }
        // The player's battle-idle portrait loop - see Battle::
        // player_idle_frame's own doc comment. Keeps advancing through
        // an attack wiggle too (the wiggle is a small shake applied ON
        // TOP of whichever frame this lands on, not a separate paused
        // state) - IDLE_FRAME_DURATION_MS matches the same per-frame
        // timing the dungeon-view IdleAnimation loop already uses.
        battle.player_idle_elapsed_ms += ctx.frame_time_ms;
        if battle.player_idle_elapsed_ms >= IDLE_FRAME_DURATION_MS {
            battle.player_idle_elapsed_ms -= IDLE_FRAME_DURATION_MS;
            battle.player_idle_frame += 1;
        }
        // A played-once action animation (Attack/Defend/Technique), if the
        // player's most recent action set one (see resolve_player_action's
        // BattleAction branches) - a no-op past its own last frame, and
        // cleared entirely once dismiss_action_result returns turn to
        // Filling, so it never lingers into the next race.
        if let Some(anim) = battle.player_action_animation.as_mut() {
            anim.tick(ctx.frame_time_ms);
        }
        for enemy in battle.enemies.iter_mut() {
            if let Some((_, remaining)) = &mut enemy.flash {
                *remaining -= ctx.frame_time_ms;
                if *remaining <= 0.0 {
                    enemy.flash = None;
                }
            }
            if let Some(popup) = &mut enemy.damage_popup {
                popup.remaining_ms -= ctx.frame_time_ms;
                if popup.remaining_ms <= 0.0 {
                    enemy.damage_popup = None;
                }
            }
            // This enemy's own battle-idle portrait loop - see
            // EnemyCombatant::battle_idle_frame's own doc comment.
            enemy.battle_idle_elapsed_ms += ctx.frame_time_ms;
            if enemy.battle_idle_elapsed_ms >= IDLE_FRAME_DURATION_MS {
                enemy.battle_idle_elapsed_ms -= IDLE_FRAME_DURATION_MS;
                enemy.battle_idle_frame += 1;
            }
            // This enemy's own played-once attack animation, if
            // trigger_enemy_action set one - same no-op-past-last-frame/
            // cleared-on-dismiss contract as the player's above.
            if let Some(anim) = enemy.attack_animation.as_mut() {
                anim.tick(ctx.frame_time_ms);
            }
        }

        // Any boss corpses still playing their death animation (see
        // Battle::dying_effects) - ticked unconditionally like every
        // other in-flight effect above, dropped the frame each one
        // finishes (OneShotAnimation::finished(), repeat: false) rather
        // than lingering held on its last frame forever.
        for effect in battle.dying_effects.iter_mut() {
            effect.animation.tick(ctx.frame_time_ms);
        }
        battle
            .dying_effects
            .retain(|effect| !effect.animation.finished());

        // A multi-hit technique (MultiHit/AoeMultiHit) still has hits
        // waiting to land one at a time - see battle::damage::HitQueue.
        // Ticked unconditionally, same as the flash/popup timers just
        // above, so it keeps landing hits regardless of which
        // BattleTurn is currently showing. See the ActionResult(Player)
        // arm further below - it holds off its own auto-advance timer
        // while this is still Some, so the summary line has actually
        // been pushed before the result screen can dismiss.
        damage::tick_hit_queue(&mut self.ecs, &mut battle, ctx.frame_time_ms);

        // --- ATB gauges: fill continuously from Speed (see
        // BattleTurn::Filling's doc comment and atb_fill_rate) - the
        // FFVII-style replacement for the old fixed "whoever's faster
        // goes first this round" system, now generalized to the player
        // plus every enemy in the fight (each with its own independent
        // gauge - see EnemyCombatant::gauge). Every fill rate is scaled
        // by the player's chosen BattleSpeed (Options screen) - a pure
        // pacing knob applied uniformly to everyone, changing how long
        // battles take to sit through without changing who's faster than
        // whom.
        //
        // WHICH gauges actually tick this frame depends on both the
        // current BattleTurn and the chosen AtbMode - see AtbMode's own
        // doc comment for the full rules; the decision is identical
        // whether there's one enemy or four, it's just applied to the
        // whole `enemies` list now instead of a single field. Filling
        // always ticks everyone, regardless of mode: nobody has anything
        // "in progress" to protect a wait-pause for. Wait mode freezes
        // everything outside Filling, exactly as before this setting
        // existed. Active (True ATB) mode additionally lets every
        // enemy's gauge fill during the player's own PlayerMenu (see the
        // interrupt check further down), and lets the player's AND every
        // enemy's gauge keep filling during any one enemy's own
        // ActionResult - but still freezes everything during the
        // PLAYER's own ActionResult, the one deliberate exception (see
        // AtbMode::Active's doc comment for why).
        let battle_speed = *self.resources.get::<BattleSpeed>().unwrap();
        let atb_mode = *self.resources.get::<AtbMode>().unwrap();
        let (tick_player, tick_enemies) = match battle.turn {
            BattleTurn::Filling => (true, true),
            BattleTurn::PlayerMenu => (false, atb_mode == AtbMode::Active),
            BattleTurn::ActionResult(Combatant::Enemy(_)) => {
                let active = atb_mode == AtbMode::Active;
                (active, active)
            }
            BattleTurn::ActionResult(Combatant::Player) => (false, false),
        };
        if tick_player || tick_enemies {
            let speed_mult = battle_speed.rate_multiplier();
            if tick_player {
                battle.player_gauge = (battle.player_gauge
                    + atb_fill_rate(&self.ecs, battle.player) * speed_mult * ctx.frame_time_ms)
                    .min(ATB_GAUGE_MAX);
            }
            if tick_enemies {
                for enemy in battle.enemies.iter_mut() {
                    let rate = atb_fill_rate(&self.ecs, enemy.entity);
                    enemy.gauge =
                        (enemy.gauge + rate * speed_mult * ctx.frame_time_ms).min(ATB_GAUGE_MAX);
                }
            }
        }

        if battle.turn == BattleTurn::Filling {
            if let Some(chosen) = battle.queued_player_action.take() {
                // A True ATB queued choice (see Battle::queued_player_action)
                // - resolve it right now rather than waiting for the
                // player_gauge check below: it's already at max (that's
                // what made queuing possible in the first place), and the
                // whole point of queuing was to not make the player wait
                // any longer than necessary once it's finally safe to act.
                self.resolve_player_action(&mut battle, chosen);
            } else if battle.player_gauge >= ATB_GAUGE_MAX {
                // A same-frame tie always favors the player - simplest
                // deterministic rule, and it means the player is never the
                // one left waiting an extra frame purely due to check order.
                battle.turn = BattleTurn::PlayerMenu;
            } else if let Some(attacker) = battle
                .enemies
                .iter()
                .find(|e| e.gauge >= ATB_GAUGE_MAX)
                .map(|e| e.entity)
            {
                // Whichever enemy is first in stable list order among
                // those ready wins a same-frame tie, mirroring the
                // player-favoring rule above - simple and deterministic,
                // not meant to imply anything about "real" simultaneity.
                if self.trigger_enemy_action(&mut battle, attacker, enter_held) {
                    return;
                }
            }
        }

        let (player_hp, player_max) = entity_health(&self.ecs, battle.player);
        let player_render = entity_render_component(&self.ecs, battle.player);

        let enemy_portraits: Vec<EnemyPortrait> = battle
            .enemies
            .iter()
            .filter_map(|e| {
                entity_render_component(&self.ecs, e.entity).map(|render| EnemyPortrait {
                    render,
                    flash: e.flash,
                    name: e.name.clone(),
                    battle_idle_frame: e.battle_idle_frame,
                    attack_glyph: e
                        .attack_animation
                        .as_ref()
                        .map(OneShotAnimation::current_glyph),
                })
            })
            .collect();
        let player_class = entity_class(&self.ecs, battle.player);
        let technique_glyph = battle
            .player_action_animation
            .as_ref()
            .map(OneShotAnimation::current_glyph);
        self.draw_battle_arena(
            &enemy_portraits,
            &battle.dying_effects,
            player_render,
            battle.player_flash,
            player_class,
            Some(battle.player_idle_frame),
            technique_glyph,
            battle.player_action_kind,
            None,
        );

        // --- "You can act" indicator: whether the player can issue an
        // action RIGHT NOW - either a normal open PlayerMenu, or (True
        // ATB only) the queuing window during some enemy's own
        // ActionResult (see Battle::queued_player_action's doc comment).
        // Drives the Actions box border color below (green normally,
        // yellow while this is true) rather than tinting the player's
        // own portrait - a portrait tint turned out to read as a stray
        // color change with no clear meaning, and worse, it silently
        // went dark again the instant an enemy interrupted (turn moved
        // off PlayerMenu) even though - under True ATB - the player
        // could very much still act in that moment via queuing. The box
        // color is checked here, once, against the SAME condition that
        // actually gates input capture in both spots below (PlayerMenu's
        // own key handling and ActionResult(Enemy(_))'s queuing capture),
        // so it can never drift out of sync with what's actually
        // interactive.
        let player_can_act = battle.turn == BattleTurn::PlayerMenu
            || (atb_mode == AtbMode::Active
                && battle.queued_player_action.is_none()
                && battle.player_gauge >= ATB_GAUGE_MAX);

        // --- Text: name + HP bar anchored next to each portrait, and a
        // message/menu panel centered in the gap between them.
        ctx.set_active_console(FINE_TEXT_CONSOLE);

        // Whichever enemy the player's own single-target actions will hit
        // right now (see Battle::primary_target) - highlighted so the
        // player has SOME visibility into who they're about to attack,
        // even though they can't choose it directly with more than one
        // enemy present.
        let primary_target = battle.primary_target(&self.ecs);
        let enemy_count = battle.enemies.len();
        // A narrower bar for multi-enemy - text now sits directly below
        // each enemy's own (narrower, spread-out) portrait slot instead
        // of one shared wide column, so the old width-16 bar (an 18+
        // character string once the current/max numbers are appended)
        // would run into the NEXT enemy's own text. Single-enemy keeps
        // the original width entirely unchanged.
        // A narrower bar for multi-enemy - even though the corrected
        // console-2 math above gives a genuine 32-column gap between
        // adjacent enemy columns (comfortable room for a full-width bar
        // on its own), keeping this a bit narrower leaves visible
        // breathing room on either side rather than filling the gap
        // edge-to-edge. Single-enemy keeps the original width entirely
        // unchanged.
        let bar_width = if enemy_count <= 1 { 16 } else { 10 };
        // See MapTheme::enemy_formation_rows' own doc comment.
        let formation_rows = self
            .resources
            .get::<Box<dyn MapTheme>>()
            .unwrap()
            .enemy_formation_rows();
        for (index, enemy) in battle.enemies.iter().enumerate() {
            let (enemy_hp, enemy_max) = entity_health(&self.ecs, enemy.entity);
            let (col, base) = enemy_text_position(enemy_count, index, formation_rows);
            let is_target = Some(enemy.entity) == primary_target;
            let name_color = if is_target { YELLOW } else { WHITE };
            let name_text = if is_target && enemy_count > 1 {
                format!("> {}", enemy.name)
            } else {
                enemy.name.clone()
            };
            ctx.print_color(col, base, name_color, BLACK, &name_text);
            ctx.print_color(
                col,
                base + 1,
                YELLOW,
                BLACK,
                &format!(
                    "{} {}/{}",
                    hp_bar_string(enemy_hp, enemy_max, bar_width),
                    enemy_hp.max(0),
                    enemy_max
                ),
            );
            // ATB gauge, drawn as the same bracket-style bar as the HP bar
            // just above it - reuses hp_bar_string's ratio/width logic
            // directly by treating the gauge as a "current/max" pair of
            // its own. CYAN (rather than the HP bar's implicit yellow-on-
            // black) so the two bars read as different things at a
            // glance. Full-ready shows in GREEN instead, as a clear
            // "it's ready" signal distinct from "it's filling."
            ctx.print_color(
                col,
                base + 2,
                if enemy.gauge >= ATB_GAUGE_MAX {
                    GREEN
                } else {
                    CYAN
                },
                BLACK,
                &hp_bar_string(enemy.gauge as i32, ATB_GAUGE_MAX as i32, bar_width),
            );
            if let Some(ActiveStatus::Dot {
                label,
                turns_remaining,
                ..
            }) = enemy.statuses.get(StatusKind::Dot)
            {
                ctx.print_color(
                    col,
                    base + 3,
                    RED,
                    BLACK,
                    &format!("{} ({}t)", label, turns_remaining),
                );
            }
        }

        ctx.print_color(32, 58, WHITE, BLACK, "You");
        ctx.print_color(
            32,
            59,
            WHITE,
            BLACK,
            &format!(
                "{} {}/{}",
                hp_bar_string(player_hp, player_max, 16),
                player_hp.max(0),
                player_max
            ),
        );
        ctx.print_color(
            32,
            56,
            if battle.player_gauge >= ATB_GAUGE_MAX {
                GREEN
            } else {
                CYAN
            },
            BLACK,
            &hp_bar_string(battle.player_gauge as i32, ATB_GAUGE_MAX as i32, 16),
        );

        // --- Active-status line for the player: previously Defending,
        // Ice Armor, and an active counter all existed as real state with
        // zero visual presence. One combined line, shown whenever any of
        // it is active, ABOVE the name/HP block instead of below: the
        // player portrait starts at pixel y=480 / row 60, so a status
        // line at row 60 would sit directly under the portrait on
        // console 3 (registered after console 2) and never actually be
        // visible - row 57 keeps clear.
        let mut player_statuses = Vec::new();
        if battle.player_defending {
            player_statuses.push("Defending".to_string());
        }
        if let Some(armor) = entity_ice_armor(&self.ecs, battle.player) {
            player_statuses.push(format!("Ice Armor ({} left)", armor.attacks_remaining));
        }
        if let Some(remaining) = buff::remaining(&battle, BuffKind::DamageReduction) {
            player_statuses.push(format!("Battle Cry ({} left)", remaining));
        }
        if let Some(remaining) = buff::remaining(&battle, BuffKind::Evasion) {
            let chance = buff::flat_value(&battle, BuffKind::Evasion);
            player_statuses.push(format!("Dodge (+{}% evasion, {} left)", chance, remaining));
        }
        if battle.player_statuses.is_active(StatusKind::Counter) {
            player_statuses.push("Countering".to_string());
        }
        if !player_statuses.is_empty() {
            ctx.print_color(32, 57, CYAN, BLACK, &player_statuses.join(" | "));
        }

        // --- Battle log: up to MAX_LOG_LINES most-recent lines, in a
        // bordered box centered above the player (not the whole screen) -
        // the player portrait spans console-2 columns 32-64, centered on
        // column 48, so the box is centered there too. Sits in the gap
        // between the enemy panel and the player's status/name/HP block
        // (starts row 57), with a line of padding on both sides.
        const MSG_BOX_X: i32 = 36;
        const MSG_BOX_Y: i32 = 45;
        const MSG_BOX_WIDTH: i32 = 24;
        const MSG_BOX_HEIGHT: i32 = MAX_LOG_LINES as i32 + 2;

        let mut log_batch = DrawBatch::new();
        log_batch.target(FINE_TEXT_CONSOLE);
        draw_ascii_box(
            &mut log_batch,
            MSG_BOX_X,
            MSG_BOX_Y,
            MSG_BOX_WIDTH,
            MSG_BOX_HEIGHT,
            ColorPair::new(WHITE, BLACK),
        );
        log_batch.submit(0).expect("Batch error");

        for (i, line) in battle.log.iter().enumerate() {
            ctx.print_color(MSG_BOX_X + 2, MSG_BOX_Y + 1 + i as i32, WHITE, BLACK, line);
        }

        // --- Floating damage numbers: bigger (32px cells, same "big
        // text" font used for title/class-select screens), and centered
        // directly over each portrait now rather than off to the side -
        // big enough now to read clearly on top of the sprite instead of
        // needing to dodge it. Drawn on DAMAGE_POPUP_CONSOLE specifically
        // (not BIG_TEXT_CONSOLE, despite sharing its exact grid/font) -
        // BIG_TEXT_CONSOLE sits BELOW BATTLE_PORTRAIT_WIGGLE_CONSOLE in
        // z-order, and every non-wiggling multi-enemy portrait now draws
        // on that wiggle console too (see draw_portrait_fancy - it needs
        // a fancy console for fractional positions even when nothing's
        // actually shaking), which meant an idle enemy portrait was
        // painting directly over its own damage number every frame.
        // DAMAGE_POPUP_CONSOLE is registered last, so it renders above
        // every portrait regardless of which console that portrait used.
        // Every console gets ctx.cls()'d at the top of every frame (see
        // State::tick), so nothing lingers once a popup's timer expires.
        //
        // Both the portrait console (5x5) and this one (40x25) cover the
        // same physical 1280x800 window. Player portrait spans columns
        // 8-16 (center 12), rows 15-20 (center 17) - unaffected by enemy
        // count. Each enemy's own popup centers over wherever its own
        // portrait actually is now (see enemy_damage_popup_position),
        // rather than a single shared column/row. print_color draws
        // left-to-right from the given column, so the start column is
        // nudged left by half the number's length to actually center it
        // rather than just its left edge.
        ctx.set_active_console(DAMAGE_POPUP_CONSOLE);
        // See MapTheme::enemy_formation_rows' own doc comment.
        let formation_rows = self
            .resources
            .get::<Box<dyn MapTheme>>()
            .unwrap()
            .enemy_formation_rows();
        for (index, enemy) in battle.enemies.iter().enumerate() {
            if let Some(popup) = &enemy.damage_popup {
                let text = format!("-{}", popup.amount);
                let (center_col, row) = enemy_damage_popup_position(enemy_count, index, formation_rows);
                let start_col = center_col - (text.chars().count() as i32) / 2;
                ctx.print_color(start_col, row, RED, BLACK, &text);
            }
        }
        if let Some(popup) = &battle.player_damage_popup {
            let text = format!("-{}", popup.amount);
            let start_col = 12 - (text.chars().count() as i32) / 2;
            ctx.print_color(start_col, 17, RED, BLACK, &text);
        }
        ctx.set_active_console(FINE_TEXT_CONSOLE);

        // --- Actions box, on the HUD console (107x67 grid, ~12px cells -
        // the same "1.5x" size used for the dungeon HUD) rather than the
        // fine-text console (8px, too small) or the big-text title console
        // (32px, too big) - a middle ground per your feedback. BOX_X=44 is
        // deliberate: the player portrait is drawn at column 1 of the
        // 5-column portrait console (256-512px), and 44*~12=528px clears
        // that portrait's right edge (512px) with a little margin, at any
        // box height, since only the box's top edge moves with action
        // count.
        //
        // BOX_Y aligns the box's top edge with the player portrait's top
        // edge instead of bottom-anchoring to the HUD console. The player
        // portrait sits at row 3 of the 5-row portrait console (both
        // consoles cover the same physical 1280x800 window): row 3 starts
        // at 3 * (800/5) = 480px down, which lands at HUD row
        // 480 / (800/67) = ~40 on the HUD console's finer grid.
        //
        // Drawn here, before the match on battle.turn, so it's visible on
        // every battle_tick frame - Filling, PlayerMenu, and ActionResult
        // alike - rather than disappearing while a result message is on
        // screen (it's only interactive during PlayerMenu, but staying
        // visible the rest of the time avoids it popping in and out).
        // `actions` is computed here too since both
        // the box's labels and PlayerMenu's key-selection logic below need
        // the same list.
        let actions = available_actions(&self.ecs, battle.player);

        // Two columns instead of one long stacked list, which used to
        // blend the always-available capability actions in with the
        // class's technique roster - hard to scan at a glance,
        // especially once a class has 3-4 techniques. Left column: the
        // 3 capability actions (Attack/Defend/Flee), identified by
        // BattleAction variant rather than position, since Flee sits at
        // the END of the underlying Vec, after every technique (see
        // battle::available_actions) - it still needs to land in the
        // left column visually. Right column: the class's technique
        // roster, owned or not. Splitting is purely a DRAWING decision -
        // `i` below is still each entry's real index into `actions`, so
        // PlayerMenu's number-key selection further down (which does
        // `actions.get(i)`) is completely unaffected by which column
        // something is drawn in.
        let is_main_action = |entry: &&BattleMenuEntry| {
            matches!(
                entry.action,
                Some(BattleAction::Attack) | Some(BattleAction::Defend) | Some(BattleAction::Flee)
            )
        };
        let main_actions: Vec<(usize, &BattleMenuEntry)> = actions
            .iter()
            .enumerate()
            .filter(|(_, entry)| is_main_action(entry))
            .collect();
        let other_actions: Vec<(usize, &BattleMenuEntry)> = actions
            .iter()
            .enumerate()
            .filter(|(_, entry)| !is_main_action(entry))
            .collect();

        // Dropped the old "(locked)" suffix on an unowned technique - the
        // greyed-out DARK_GRAY color already says the same thing, and
        // the suffix was some of the longest text in the whole menu,
        // which was its own part of the "everything blends together"
        // problem.
        fn menu_row_label(i: usize, entry: &BattleMenuEntry) -> (String, (u8, u8, u8)) {
            if entry.action.is_some() {
                let label = match entry.count {
                    Some(n) => format!("{}) {} x{}", i + 1, entry.label, n),
                    None => format!("{}) {}", i + 1, entry.label),
                };
                (label, GREEN)
            } else {
                (format!("{}) {}", i + 1, entry.label), DARK_GRAY)
            }
        }

        // One-time cross-battle cursor memory seed - see MenuCursor's own
        // doc comment. Runs at most once per battle (menu_cursor_seeded
        // guards it), checked every frame regardless of battle.turn since
        // a battle can sit in Filling for a moment before the player's
        // first PlayerMenu ever shows - seeding early means the pointer
        // is already in the right place the very first time the box
        // becomes interactive, not one frame late.
        if !battle.menu_cursor_seeded {
            battle.menu_cursor_seeded = true;
            let memory_on = *self.resources.get::<MenuMemory>().unwrap() == MenuMemory::On;
            if memory_on {
                if let Some(class) = entity_class(&self.ecs, battle.player) {
                    let remembered = self
                        .resources
                        .get::<LastBattleAction>()
                        .unwrap()
                        .get(&class)
                        .map(str::to_string);
                    if let Some(remembered) = remembered {
                        if let Some(row) = main_actions
                            .iter()
                            .position(|(_, entry)| entry.label == remembered)
                        {
                            battle.menu_cursor.col = 0;
                            battle.menu_cursor.row = row;
                        } else if let Some(row) = other_actions
                            .iter()
                            .position(|(_, entry)| entry.label == remembered)
                        {
                            battle.menu_cursor.col = 1;
                            battle.menu_cursor.row = row;
                        }
                    }
                }
            }
        }

        // Arrow-key cursor navigation. In Wait mode, only while the menu
        // is actually open for input (PlayerMenu) - time is fully frozen
        // there anyway (that's the whole premise of Wait mode), so
        // there's no benefit to moving the cursor any earlier, and
        // nothing to pre-aim toward before it's even your turn. In
        // Active (True ATB) mode, movement is allowed in EVERY
        // BattleTurn state - gauges keep racing/enemies keep acting
        // around you there, so being able to pre-aim the cursor while
        // waiting for your own gauge to fill (or while an enemy's result
        // is still showing) is exactly the kind of thing that mode
        // should support, per the "harder mode, more to juggle" framing.
        // Processed here, before the row labels are drawn, so a moved
        // cursor's highlight/pointer show up the SAME frame the key was
        // pressed rather than one frame late.
        if atb_mode == AtbMode::Active || battle.turn == BattleTurn::PlayerMenu {
            let current_col_len = if battle.menu_cursor.col == 0 {
                main_actions.len()
            } else {
                other_actions.len()
            };
            match ctx.key {
                Some(VirtualKeyCode::Up) => battle.menu_cursor.move_vertical(-1, current_col_len),
                Some(VirtualKeyCode::Down) => battle.menu_cursor.move_vertical(1, current_col_len),
                Some(VirtualKeyCode::Left) => {
                    battle.menu_cursor.move_horizontal(0, main_actions.len())
                }
                Some(VirtualKeyCode::Right) => {
                    battle.menu_cursor.move_horizontal(1, other_actions.len())
                }
                _ => {}
            }
        }

        // Whichever entry the cursor currently sits on, if any - shared
        // by PlayerMenu's direct Enter-confirm below AND the True-ATB
        // queuing window (ActionResult(Enemy(_))), so cursor+Enter
        // behaves identically in both places, same as number keys
        // already do.
        let cursor_entry = if battle.menu_cursor.col == 0 {
            main_actions.get(battle.menu_cursor.row)
        } else {
            other_actions.get(battle.menu_cursor.row)
        };

        const BOX_X: i32 = 44;
        // Anchored to the player's own portrait top edge (see the doc
        // comment below) for a single enemy, but pushed down further
        // for 2+ - a zigzag formation's own front row (see
        // MapTheme::enemy_formation_rows) puts an enemy's own name/HP
        // text as low as row 67 of the 100-row FINE_TEXT_CONSOLE (67%
        // down the screen) for Forest specifically - its own front row
        // (2.3) is the LOWEST any theme currently uses, so 47 is
        // calibrated to that worst case and stays a safe margin for
        // Dungeon/Sewer's higher-up front row (1.9) too, without needing
        // to vary by theme itself. Applies regardless of whether the
        // fight has 2, 3, or 4 enemies, since any of them can put an
        // enemy on the front row (only single-enemy fights never use the
        // zigzag at all, so they alone keep the higher/earlier value).
        let box_y_base = if battle.enemies.len() <= 1 { 40 } else { 47 };
        const BOX_COL_WIDTH: i32 = 20;
        let box_width = BOX_COL_WIDTH * 2 + 3;
        let box_content_rows = main_actions.len().max(other_actions.len()) as i32;
        let box_height = box_content_rows + 4;
        // Clamp so a tall action list (more techniques than fit below row
        // 40) never runs off the bottom of the console.
        let box_y = box_y_base.min(HUD_ROWS - box_height);

        let mut menu_batch = DrawBatch::new();
        menu_batch.target(HUD_CONSOLE);
        // Border color reflects player_can_act (see its own doc comment
        // above) - yellow whenever the player can issue an action right
        // now, green otherwise. Replaces the earlier attempt at tinting
        // the player's own portrait, which read as an unexplained color
        // change and, worse, dropped out the instant the enemy
        // interrupted even when queuing (True ATB) still meant the
        // player could act.
        let box_border_color = if player_can_act { YELLOW } else { GREEN };
        draw_ascii_box(
            &mut menu_batch,
            BOX_X,
            box_y,
            box_width,
            box_height,
            ColorPair::new(box_border_color, BLACK),
        );
        menu_batch.submit(0).expect("Batch error");

        ctx.set_active_console(HUD_CONSOLE);
        ctx.print_color(BOX_X + 1, box_y + 1, YELLOW, BLACK, "Actions");
        for (row, (i, entry)) in main_actions.iter().enumerate() {
            let (label, color) = menu_row_label(*i, entry);
            let selected = battle.menu_cursor.col == 0 && battle.menu_cursor.row == row;
            print_menu_row_left(
                ctx,
                BOX_X + 1,
                box_y + 3 + row as i32,
                color,
                &label,
                selected,
            );
        }
        for (row, (i, entry)) in other_actions.iter().enumerate() {
            let (label, color) = menu_row_label(*i, entry);
            let selected = battle.menu_cursor.col == 1 && battle.menu_cursor.row == row;
            print_menu_row_left(
                ctx,
                BOX_X + 1 + BOX_COL_WIDTH + 1,
                box_y + 3 + row as i32,
                color,
                &label,
                selected,
            );
        }
        ctx.set_active_console(FINE_TEXT_CONSOLE);

        match battle.turn {
            BattleTurn::Filling => {
                // Nothing to show beyond the gauges already drawn above -
                // there's no menu to interact with and no result to
                // dismiss while everyone is still racing to full.
            }
            BattleTurn::PlayerMenu => {
                let chosen = match ctx.key {
                    Some(VirtualKeyCode::Return) => {
                        cursor_entry.and_then(|(_, entry)| entry.action)
                    }
                    Some(key) => number_key_index(key)
                        .and_then(|i| actions.get(i))
                        .and_then(|entry| entry.action),
                    None => None,
                };
                if let Some(chosen) = chosen {
                    self.resolve_player_action(&mut battle, chosen);
                }

                // True ATB interrupt: if the player DIDN'T just act above
                // (battle.turn is still PlayerMenu - the resolve call
                // above would have moved it to ActionResult(Player)
                // otherwise) and some enemy's gauge has since filled all
                // the way (see this function's tick_enemies logic, which
                // only lets enemies fill during PlayerMenu under
                // AtbMode::Active), that enemy attacks right now instead
                // of waiting for a menu choice that never came in time.
                // This is the entire point of True ATB - "select an
                // attack ASAP or take the hit." The player isn't locked
                // out afterward, though - see ActionResult(Enemy(_))
                // below, which keeps accepting a choice (queued rather
                // than resolved immediately) for exactly this situation.
                let interrupting =
                    if atb_mode == AtbMode::Active && battle.turn == BattleTurn::PlayerMenu {
                        battle
                            .enemies
                            .iter()
                            .find(|e| e.gauge >= ATB_GAUGE_MAX)
                            .map(|e| e.entity)
                    } else {
                        None
                    };
                if let Some(attacker) = interrupting {
                    if self.trigger_enemy_action(&mut battle, attacker, enter_held) {
                        return;
                    }
                }
            }
            BattleTurn::ActionResult(Combatant::Enemy(_)) => {
                // True ATB queuing: some enemy's own result may still be
                // playing while gauges keep moving underneath it (see
                // this function's tick_player/tick_enemies match - Active
                // mode lets both continue here). If the player's gauge
                // is already full and they haven't queued anything yet,
                // let them choose right now instead of forcing them to
                // wait for a fresh PlayerMenu prompt once this dismisses
                // - see Battle::queued_player_action's own doc comment
                // for why this was the actual fix needed: without it, an
                // enemy interrupt (just above) used to fully lock the
                // player out of choosing anything until its result
                // finished, making it very hard to ever land a hit under
                // True ATB. Wait mode never reaches this branch with
                // player_gauge at max in the first place (tick_player is
                // false for it here), so this is a no-op there. Accepts
                // cursor+Enter here too, not just number keys - same
                // chosen-resolution shape as PlayerMenu above, sharing
                // the same cursor_entry computed earlier.
                if atb_mode == AtbMode::Active
                    && battle.queued_player_action.is_none()
                    && battle.player_gauge >= ATB_GAUGE_MAX
                {
                    let chosen = match ctx.key {
                        Some(VirtualKeyCode::Return) => {
                            cursor_entry.and_then(|(_, entry)| entry.action)
                        }
                        Some(key) => number_key_index(key)
                            .and_then(|i| actions.get(i))
                            .and_then(|entry| entry.action),
                        None => None,
                    };
                    if let Some(chosen) = chosen {
                        battle.queued_player_action = Some(chosen);
                    }
                }

                battle.result_timer_ms -= ctx.frame_time_ms;
                if ctx.key.is_some() || battle.result_timer_ms <= 0.0 {
                    if let ResultOutcome::EndBattleTick =
                        self.dismiss_action_result(&mut battle, enter_held)
                    {
                        return;
                    }
                }
            }
            BattleTurn::ActionResult(Combatant::Player) => {
                // No queuing capture here (unlike the Enemy variant above)
                // - gauges are fully frozen during the player's OWN
                // action result in every mode (see this function's
                // tick_player/tick_enemies match), so player_gauge can't
                // possibly be back at max yet for there to be anything
                // to queue.
                //
                // While a multi-hit technique still has hits left to land
                // (battle.hit_queue - ticked unconditionally above, near
                // the flash/popup timers), hold off entirely: don't count
                // down the auto-advance timer, and ignore a keypress
                // dismiss too. Otherwise the result screen could dismiss
                // itself (or a keypress could dismiss it) before the
                // player ever saw every hit land - tick_hit_queue resets
                // result_timer_ms once the queue actually finishes, so
                // normal dismissal resumes automatically right after.
                if battle.hit_queue.is_none() {
                    battle.result_timer_ms -= ctx.frame_time_ms;
                    if ctx.key.is_some() || battle.result_timer_ms <= 0.0 {
                        if let ResultOutcome::EndBattleTick =
                            self.dismiss_action_result(&mut battle, enter_held)
                        {
                            return;
                        }
                    }
                }
            }
        }

        self.resources.insert(Some(battle));
    }

    /// Called from main.rs's tick() dispatcher, so this needs to be `pub`.
    pub fn battle_victory_tick(&mut self, ctx: &mut BTerm) {
        let victory_snapshot = self
            .resources
            .get::<Option<BattleVictory>>()
            .unwrap()
            .clone();
        let mut victory = match victory_snapshot {
            Some(v) => v,
            None => {
                // Shouldn't happen, but don't get stuck if it does.
                self.resources.insert(TurnState::AwaitingInput);
                return;
            }
        };

        // Advance the victory-pose animation, if this class has one (see
        // BattleVictory::portrait_animation's own doc comment) - a no-op
        // past its own last frame. The ticked copy is written back into
        // the resource below, the same "snapshot, mutate, re-insert"
        // shape battle_tick already uses for `battle` itself.
        if let Some(anim) = victory.portrait_animation.as_mut() {
            anim.tick(ctx.frame_time_ms);
        }
        let victory_glyph = victory
            .portrait_animation
            .as_ref()
            .map(OneShotAnimation::current_glyph);

        let player_render = entity_render_component(&self.ecs, victory.player);
        let player_class = entity_class(&self.ecs, victory.player);
        self.draw_battle_arena(
            &[],
            &[],
            player_render,
            None,
            player_class,
            None,
            None,
            None,
            victory_glyph,
        );
        self.resources.insert(Some(victory.clone()));

        ctx.set_active_console(FINE_TEXT_CONSOLE);
        ctx.print_color_centered(
            45,
            GREEN,
            BLACK,
            &format!("You defeated {}!", join_enemy_names(&victory.enemy_names)),
        );
        match (victory.loot.is_empty(), victory.gold_earned) {
            (false, _) => {
                ctx.print_color_centered(
                    48,
                    YELLOW,
                    BLACK,
                    &format!("You found: {}!", victory.loot.join(", ")),
                );
            }
            (true, Some(gold)) => {
                ctx.print_color_centered(48, YELLOW, BLACK, &format!("You found: {} gold!", gold));
            }
            (true, None) => {
                ctx.print_color_centered(48, WHITE, BLACK, "No loot this time.");
            }
        }
        ctx.print_color_centered(51, YELLOW, BLACK, "Press ENTER to continue.");

        // Requires Enter specifically, not "any key" - this is the one
        // dismiss point in the whole battle flow that genuinely needed
        // it. Holding down an attack hotkey to keep queuing attacks ASAP
        // under Fast + True ATB (see Battle::queued_player_action) means
        // that key can still be held the instant the last enemy actually
        // dies. If this screen dismissed on any key, that same held key
        // would instantly exit back to the dungeon map - where, if it
        // happens to double as a Potion/Map slot key, it would keep
        // "using" that item and consuming a real dungeon turn on every
        // single frame it's still held, potentially burning through an
        // entire stack of potions and handing the monsters a pile of
        // free turns before the player even lets go of the key. The
        // in-battle ActionResult screens deliberately keep dismissing on
        // any key, unlike this one - blowing through those as fast as
        // possible while held is exactly the desired behavior there.
        // pending_enter_release guards against the exact same held Enter
        // that fired the killing blow (via cursor+Enter or the True-ATB
        // queuing window) also instantly dismissing this screen - see
        // that field's own doc comment on State and finish_battle, which
        // arms it.
        if ctx.key == Some(VirtualKeyCode::Return) && !self.pending_enter_release {
            self.resources.insert(None::<BattleVictory>);
            let arena_run = self.resources.get::<Option<ArenaRun>>().unwrap().clone();
            match arena_run {
                Some(run) => self.handle_arena_kill(run),
                None => {
                    self.resources.insert(TurnState::AwaitingInput);
                }
            }
        }
    }
}

/// Headless class-survivability simulation - a permanent fixture (marked
/// `#[ignore]` so it doesn't run as part of the normal `cargo test`,
/// since it takes real time even in release mode), rerun by hand after
/// any class-balance change: `cargo test --release
/// class_survivability_report -- --ignored --nocapture` (Dungeon Crawl)
/// or `cargo test --release arena_class_survivability_report -- --ignored
/// --nocapture` (Battle Arena) from ever_space_rrpg/. The Dungeon Crawl
/// one answers "how many runs actually reach the first shop" for each
/// class; the Arena one (added later, sharing every helper below except
/// its own navigation/shopping policy - see simulate_one_arena_run)
/// answers "how many runs clear the whole 3-level run, and where do the
/// rest die" - Arena has no ambient floor loot at all, so a real shopping
/// policy (see shop_step) matters a lot more to its numbers than the
/// Dungeon Crawl bot's simpler potion-only handling. Both use the REAL
/// game logic end to end wherever possible - the actual schedulers,
/// movement, item pickup, chest interaction, and shop transition, not a
/// separate simplified model.
///
/// Combat itself goes straight through the same
/// resolve_player_action/trigger_enemy_action/dismiss_action_result
/// functions battle_tick calls (turn order simplified to "player acts,
/// then every living enemy acts back" instead of real-time ATB gauge
/// filling - no real frame loop to drive that headlessly) rather than
/// battle_tick/chest_loot_tick/battle_victory_tick themselves, which do
/// real `ctx.set_active_console`/`ctx.print_*` calls that need a live
/// window's console registry and panic without one. Placed here (not
/// main.rs) specifically so it can call
/// resolve_player_action/trigger_enemy_action/dismiss_action_result
/// directly - those are private to this module, and Rust privacy lets a
/// module's descendants (including a nested test module) see its own
/// private items, not the reverse.
///
/// The player's in-battle policy (see choose_battle_action) isn't
/// optimal, but it's not naive either: flee once critically low on
/// health, use an owned offensive Technique when one's available (they're
/// one-time-use, consumed on cast, so this naturally tapers off to plain
/// Attack once a run's kit is spent), otherwise Attack. Between fights,
/// drink a Healing Potion below half health. A genuinely optimal player
/// (perfect target/technique selection, exact HP thresholds) would likely
/// do somewhat better than these numbers - treat this as a realistic
/// lower-middle bound, not a hard floor.
#[cfg(test)]
mod class_survivability_diagnostic {
    use super::*;
    use crate::State;

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum RunOutcome {
        ReachedShop,
        Died,
        TimedOut,
    }

    fn find_exit_tile(map: &Map) -> Point {
        let idx = map
            .tiles
            .iter()
            .position(|t| *t == TileType::Exit)
            .expect("no Exit tile on this floor");
        map.index_to_point2d(idx)
    }

    /// A correct, from-scratch breadth-first distance field from `target`
    /// across every 4-directionally-connected `can_enter_tile` cell.
    /// Written in-house rather than using `bracket_pathfinding::
    /// DijkstraMap`, after that library function proved genuinely
    /// unreliable across two separate real reproductions during this
    /// bot's development:
    ///
    /// 1. `DijkstraMap::find_lowest_exit`'s own pick considers diagonal
    ///    exits too (`get_available_exits` allows them), which this bot
    ///    has no way to act on (`Action` only has Up/Down/Left/Right, no
    ///    diagonals) - decomposing a diagonal suggestion into a single
    ///    cardinal step by dx/dy sign doesn't reliably reduce distance,
    ///    and caused a real back-and-forth stall right next to a chest's
    ///    one-tile approach corridor. Picking directly among the 4
    ///    cardinal neighbors by their own reported distance (see
    ///    step_toward) was the first fix - but:
    /// 2. Confirmed from `bracket-pathfinding`'s own source
    ///    (`DijkstraMap::build`): the seed tile's own array slot is NEVER
    ///    explicitly set to 0.0 - it only gets overwritten later by a
    ///    neighbor's own relaxation pass, landing at roughly the edge
    ///    cost back to that neighbor (~2.0 for one cardinal hop) instead
    ///    of the true 0. Worse, since `build`'s open list is a plain
    ///    FIFO queue rather than a priority queue, that wrong value can
    ///    itself get used as a base by further relaxations, corrupting
    ///    more than just the seed's own single cell - confirmed by a
    ///    second real reproduction (a DIFFERENT stable 2-cycle a couple
    ///    of tiles away from an already-special-cased target, after the
    ///    first fix). Patching individual symptomatic cells wasn't going
    ///    to hold indefinitely, hence this from-scratch replacement.
    ///
    /// Every step this bot ever takes costs exactly 1, so a plain BFS
    /// isn't a workaround here, it's the textbook-correct algorithm for
    /// this anyway - no priority queue needed, and no possibility of the
    /// seed/relaxation-order bugs above, since the seed's distance is
    /// set to 0 directly rather than relying on any later relaxation
    /// pass to (maybe) get it right. Returns a full
    /// SCREEN_WIDTH*SCREEN_HEIGHT-sized field (unreached cells stay at
    /// i32::MAX) so callers can index it the same way the old
    /// DijkstraMap.map Vec was indexed.
    fn bfs_distance_field(map: &Map, target: Point) -> Vec<i32> {
        let mut field = vec![i32::MAX; (SCREEN_WIDTH * SCREEN_HEIGHT) as usize];
        if !map.in_bounds(target) {
            return field;
        }
        let target_idx = map.point2d_to_index(target);
        field[target_idx] = 0;
        let mut queue: std::collections::VecDeque<Point> = std::collections::VecDeque::new();
        queue.push_back(target);
        while let Some(current) = queue.pop_front() {
            let current_dist = field[map.point2d_to_index(current)];
            for delta in [
                Point::new(0, -1),
                Point::new(0, 1),
                Point::new(-1, 0),
                Point::new(1, 0),
            ] {
                let neighbor = current + delta;
                if neighbor != target && !map.can_enter_tile(neighbor) {
                    continue;
                }
                if !map.in_bounds(neighbor) {
                    continue;
                }
                let idx = map.point2d_to_index(neighbor);
                if field[idx] == i32::MAX {
                    field[idx] = current_dist + 1;
                    queue.push_back(neighbor);
                }
            }
        }
        field
    }

    /// One step toward `target`, going through the real player_input
    /// system (via the same `key` resource main.rs's tick() sets from a
    /// live keypress) so enemy-bump battle-starts/item auto-pickup/chest
    /// interaction all happen exactly as they do for a real player -
    /// only the SOURCE of the key (computed here, not read from a
    /// window) differs. Picks directly among the 4 CARDINAL neighbors by
    /// their own bfs_distance_field value - see that function's own doc
    /// comment for why this doesn't just trust a library pathfinder.
    fn step_toward(state: &mut State, target: Point) {
        let (_, player_pt) = find_player(&state.ecs).unwrap();
        let action = {
            let map = state.resources.get::<Map>().unwrap();
            let field = bfs_distance_field(&map, target);
            let candidates: [(Action, Point); 4] = [
                (Action::MoveUp, Point::new(player_pt.x, player_pt.y - 1)),
                (Action::MoveDown, Point::new(player_pt.x, player_pt.y + 1)),
                (Action::MoveLeft, Point::new(player_pt.x - 1, player_pt.y)),
                (Action::MoveRight, Point::new(player_pt.x + 1, player_pt.y)),
            ];
            let mut best: Option<(Action, i32)> = None;
            for (action, dest) in candidates {
                // A candidate exactly on the target is always enterable
                // for navigation purposes even if it's a special tile
                // type (a shop counter item's own square, the map's Exit
                // tile) - can_enter_tile already allows Exit, and shop
                // items sit on Counter tiles that step_toward's callers
                // only ever pass as an ADJACENCY target, never actually
                // stepped onto, so this never actually walks onto a true
                // wall.
                if dest != target && !map.can_enter_tile(dest) {
                    continue;
                }
                if !map.in_bounds(dest) {
                    continue;
                }
                let dist = field[map.point2d_to_index(dest)];
                if dist == i32::MAX {
                    continue;
                }
                let better = match best {
                    Some((_, best_dist)) => dist < best_dist,
                    None => true,
                };
                if better {
                    best = Some((action, dist));
                }
            }
            best.map(|(action, _)| action)
        };
        let action = match action {
            Some(a) => a,
            None => return,
        };
        let key = state.resources.get::<Keymap>().unwrap().key_for(action);
        state.resources.insert(Some(key));
        state
            .input_systems
            .execute(&mut state.ecs, &mut state.resources);
        state.resources.insert(None::<VirtualKeyCode>);
    }

    /// Uses one carried Healing Potion, exactly like pressing its Item
    /// Bar slot would - returns false (does nothing) if none carried.
    fn use_potion(state: &mut State) -> bool {
        let (player, _) = find_player(&state.ecs).unwrap();
        let potion = <(Entity, &Name, &Carried)>::query()
            .iter(&state.ecs)
            .find(|(_, n, c)| n.0 == "Healing Potion" && c.0 == player)
            .map(|(e, _, _)| *e);
        let potion = match potion {
            Some(p) => p,
            None => return false,
        };
        state.ecs.push((ActivateItem {
            used_by: player,
            item: potion,
        },));
        state.resources.insert(TurnState::PlayerTurn);
        state
            .player_systems
            .execute(&mut state.ecs, &mut state.resources);
        true
    }

    /// Picks this turn's battle action - see this module's own doc
    /// comment for the overall policy. Below a quarter health, flee
    /// rather than risk dying outright (a real player would); otherwise
    /// spend an owned offensive Technique if one's available (they're
    /// one-time-use, so this naturally exhausts a run's kit rather than
    /// spamming forever), falling back to a plain Attack.
    fn choose_battle_action(ecs: &World, player: Entity) -> BattleAction {
        let (hp, max_hp) = {
            let h = <&Health>::query()
                .filter(component::<Player>())
                .iter(ecs)
                .next()
                .unwrap();
            (h.current, h.max)
        };
        // Fleeing only actually helps if there's a Healing Potion to
        // drink afterward (flee, heal, go back in) - with none carried,
        // fleeing accomplishes nothing a real player would want, so this
        // bot fights on instead (a Technique if one's owned, else a
        // plain Attack) rather than wasting the flee.
        let has_potion = <(&Name, &Carried)>::query()
            .iter(ecs)
            .any(|(n, c)| n.0 == "Healing Potion" && c.0 == player);
        if hp * 4 < max_hp && has_potion {
            return BattleAction::Flee;
        }
        available_actions(ecs, player)
            .into_iter()
            .find_map(|entry| match entry.action {
                Some(BattleAction::Technique(item)) => Some(BattleAction::Technique(item)),
                _ => None,
            })
            .unwrap_or(BattleAction::Attack)
    }

    /// Resolves a whole fight synchronously - see this module's own doc
    /// comment for the simplified (non-ATB) turn order.
    fn resolve_battle(state: &mut State) {
        let mut battle = state
            .resources
            .get::<Option<Battle>>()
            .unwrap()
            .clone()
            .unwrap();
        for _ in 0..200 {
            let action = choose_battle_action(&state.ecs, battle.player);
            state.resolve_player_action(&mut battle, action);
            if matches!(
                state.dismiss_action_result(&mut battle, false),
                ResultOutcome::EndBattleTick
            ) {
                return;
            }
            if battle.fled {
                return;
            }
            let enemies_this_round: Vec<Entity> =
                battle.enemies.iter().map(|e| e.entity).collect();
            for enemy in enemies_this_round {
                if !battle.enemies.iter().any(|e| e.entity == enemy) {
                    continue; // already died earlier this round
                }
                if state.trigger_enemy_action(&mut battle, enemy, false) {
                    return; // ended via a DOT-kill inside trigger_enemy_action itself
                }
                if matches!(
                    state.dismiss_action_result(&mut battle, false),
                    ResultOutcome::EndBattleTick
                ) {
                    return;
                }
            }
        }
    }

    fn simulate_one_run(class: &str, max_actions: usize) -> RunOutcome {
        let mut state = State::new();
        state.start_game(class);
        // Per-frame resources main.rs's tick() normally sets from a live
        // ctx/window before dispatching - never inserted by start_game
        // itself, since that's a real-input concern, not a new-run one.
        state.resources.insert(AbilityBarMousePos(Point::zero()));
        state.resources.insert(MouseLeftJustPressed(false));
        state.resources.insert(FrameTime(16.0));
        state.resources.insert(Point::zero()); // raw mouse_pos - see tooltips_system

        for _ in 0..max_actions {
            let current = *state.resources.get::<TurnState>().unwrap();
            match current {
                TurnState::AwaitingInput => {
                    let (hp, max_hp) = {
                        let h = <&Health>::query()
                            .filter(component::<Player>())
                            .iter(&state.ecs)
                            .next()
                            .unwrap();
                        (h.current, h.max)
                    };
                    if hp * 2 < max_hp && use_potion(&mut state) {
                        continue;
                    }
                    let chest_pt = <(&Chest, &Point)>::query()
                        .iter(&state.ecs)
                        .next()
                        .map(|(_, p)| *p);
                    let target = match chest_pt {
                        Some(p) => p,
                        None => find_exit_tile(&state.resources.get::<Map>().unwrap()),
                    };
                    step_toward(&mut state, target);
                }
                TurnState::PlayerTurn => {
                    state
                        .player_systems
                        .execute(&mut state.ecs, &mut state.resources);
                }
                TurnState::MonsterTurn => {
                    state
                        .monster_systems
                        .execute(&mut state.ecs, &mut state.resources);
                }
                TurnState::InBattle => {
                    resolve_battle(&mut state);
                }
                TurnState::BattleVictory => {
                    // Same dismiss logic as battle_victory_tick's Enter
                    // branch, minus the rendering it also does.
                    state.resources.insert(None::<BattleVictory>);
                    state.resources.insert(TurnState::AwaitingInput);
                }
                TurnState::ChestOpened => {
                    // Same dismiss logic as chest_loot_tick's Enter
                    // branch, minus the rendering it also does.
                    state.resources.insert(None::<ChestLoot>);
                    state.resources.insert(TurnState::AwaitingInput);
                }
                TurnState::DungeonShopTransition => {
                    state.dungeon_shop_transition();
                    return RunOutcome::ReachedShop;
                }
                TurnState::GameOver => {
                    return RunOutcome::Died;
                }
                _ => {}
            }
        }
        RunOutcome::TimedOut
    }

    #[test]
    #[ignore]
    fn class_survivability_report() {
        let classes = ["Barbarian", "Mage", "Rogue", "Amazon", "Hunter"];
        let runs_per_class = 10;
        let mut report = String::new();
        for class in classes {
            let mut reached = 0;
            let mut died = 0;
            let mut timed_out = 0;
            for _ in 0..runs_per_class {
                match simulate_one_run(class, 2000) {
                    RunOutcome::ReachedShop => reached += 1,
                    RunOutcome::Died => died += 1,
                    RunOutcome::TimedOut => timed_out += 1,
                }
            }
            report.push_str(&format!(
                "{class}: {reached}/{runs_per_class} reached the shop ({died} died, {timed_out} timed out)\n"
            ));
        }
        println!("\n{}", report);
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum ArenaOutcome {
        Won,
        Died { level: u8, wave: u8, boss_active: bool },
        TimedOut,
    }

    /// The 5 tiles (own position + 4 cardinal neighbors) buy_nearby_item
    /// itself checks (see components::shop_item_near) - duplicated here
    /// rather than reusing that function directly since this only needs
    /// the adjacency test, not the full stock lookup it also does.
    const SHOP_ADJACENT: [Point; 5] = [
        Point { x: 0, y: 0 },
        Point { x: 0, y: -1 },
        Point { x: 0, y: 1 },
        Point { x: -1, y: 0 },
        Point { x: 1, y: 0 },
    ];

    /// One shopping decision: Arena has no ambient floor loot at all (see
    /// this module's own updated doc comment above), so unlike the
    /// Dungeon Crawl bot - which only ever needs to drink potions it
    /// already has - this one also has to actually SHOP for them.
    /// Deliberately simple (Healing Potions only, buy while affordable
    /// and in stock, ignore weapons/abilities entirely) rather than a
    /// fully optimal shopper - same "realistic lower-middle bound, not a
    /// hard floor" spirit this module's own doc comment already sets for
    /// the in-battle policy.
    /// One attempt at buying `item_name` from the current shop - `true`
    /// means "did something this tick" (moved toward it, or bought one),
    /// so the caller should stop and let the next tick reassess; `false`
    /// means "nothing left to do here" (not sold in this shop, out of
    /// stock, or unaffordable), so the caller should move on to its next
    /// shopping priority.
    fn try_buy_item(state: &mut State, item_name: &str) -> bool {
        let (_, player_pos) = find_player(&state.ecs).unwrap();
        let found = <(&ShopStock, &Point, &Name, &Price)>::query()
            .iter(&state.ecs)
            .find(|(_, _, name, _)| name.0 == item_name)
            .map(|(stock, pos, _, price)| (stock.0, *pos, price.0));
        let (stock_count, item_pos, price) = match found {
            Some(f) => f,
            None => return false,
        };
        // Afford/stock is checked BEFORE adjacency, not after - checking
        // adjacency first meant that once gold ran out, an entity NOT
        // currently standing next to this item would still blindly walk
        // toward it every single tick (only bailing out once actually
        // adjacent), fighting whatever else this same tick's OTHER
        // priority wanted to do the moment it stepped away again - a
        // real, reproducible 2-tile stall confirmed during development
        // (walk toward the exit, walk back toward an unaffordable
        // potion, forever).
        let gold = <&Gold>::query()
            .filter(component::<Player>())
            .iter(&state.ecs)
            .next()
            .map(|g| g.0)
            .unwrap_or(0);
        if stock_count <= 0 || gold < price {
            return false;
        }
        if !SHOP_ADJACENT.iter().any(|&d| item_pos == player_pos + d) {
            step_toward(state, item_pos);
            return true;
        }
        state.resources.insert(Some(VirtualKeyCode::Return));
        state
            .input_systems
            .execute(&mut state.ecs, &mut state.resources);
        state.resources.insert(None::<VirtualKeyCode>);
        true
    }

    /// One shopping decision, in priority order: the class's own weapon
    /// for this level FIRST (a player_attack_damage of base Damage alone,
    /// with no weapon at all, is often 0 after an enemy's own Defense -
    /// confirmed for real during development: Barbarian's damage=1 base
    /// couldn't land a single point of damage unarmed, stalling every
    /// single fight forever with neither side able to finish the other -
    /// so skipping the weapon isn't a simplification, it breaks combat
    /// outright), THEN Healing Potions with whatever gold is left, THEN
    /// head for the exit once neither is affordable or there's nothing
    /// left to buy.
    fn shop_step(state: &mut State) {
        let (player_entity, _) = find_player(&state.ecs).unwrap();
        let already_has_weapon = <(&Weapon, &Carried)>::query()
            .iter(&state.ecs)
            .any(|(_, c)| c.0 == player_entity);
        if !already_has_weapon {
            let run = state.resources.get::<Option<ArenaRun>>().unwrap().clone();
            if let Some(run) = run {
                let class = entity_class(&state.ecs, player_entity).unwrap_or_default();
                if let Some(weapon_name) = weapon_name_for_class_level(&class, run.template_level())
                {
                    if try_buy_item(state, &weapon_name) {
                        return;
                    }
                }
            }
        }
        if try_buy_item(state, "Healing Potion") {
            return;
        }
        let exit_pt = find_exit_tile(&state.resources.get::<Map>().unwrap());
        step_toward(state, exit_pt);
    }

    /// One wave-combat movement decision: walk toward whichever living
    /// Enemy is closest (straight-line distance - step_toward's own
    /// Dijkstra pathing handles the actual route), triggering a real
    /// bump-battle exactly like a player walking into a monster would.
    fn wave_step(state: &mut State) {
        let (_, player_pos) = find_player(&state.ecs).unwrap();
        let target = <(&Enemy, &Point)>::query()
            .iter(&state.ecs)
            .map(|(_, p)| *p)
            .min_by_key(|p| {
                let dx = p.x - player_pos.x;
                let dy = p.y - player_pos.y;
                dx * dx + dy * dy
            });
        if let Some(target) = target {
            step_toward(state, target);
        }
    }

    fn simulate_one_arena_run(class: &str, max_actions: usize) -> ArenaOutcome {
        let mut state = State::new();
        state.start_arena(class);
        state.resources.insert(AbilityBarMousePos(Point::zero()));
        state.resources.insert(MouseLeftJustPressed(false));
        state.resources.insert(FrameTime(16.0));
        state.resources.insert(Point::zero()); // raw mouse_pos - see tooltips_system

        for _ in 0..max_actions {
            let current = *state.resources.get::<TurnState>().unwrap();
            match current {
                TurnState::AwaitingInput => {
                    let (hp, max_hp) = {
                        let h = <&Health>::query()
                            .filter(component::<Player>())
                            .iter(&state.ecs)
                            .next()
                            .unwrap();
                        (h.current, h.max)
                    };
                    if hp * 2 < max_hp && use_potion(&mut state) {
                        continue;
                    }
                    let shopping = state
                        .resources
                        .get::<Option<ShoppingActive>>()
                        .unwrap()
                        .is_some();
                    if shopping {
                        shop_step(&mut state);
                    } else {
                        wave_step(&mut state);
                    }
                }
                TurnState::PlayerTurn => {
                    state
                        .player_systems
                        .execute(&mut state.ecs, &mut state.resources);
                }
                TurnState::MonsterTurn => {
                    state
                        .monster_systems
                        .execute(&mut state.ecs, &mut state.resources);
                }
                TurnState::InBattle => {
                    resolve_battle(&mut state);
                }
                TurnState::BattleVictory => {
                    // Same dismiss logic as battle_victory_tick's Enter
                    // branch, minus the rendering it also does - that
                    // real branch is what actually decides the next wave/
                    // boss/shop/Victory via handle_arena_kill.
                    state.resources.insert(None::<BattleVictory>);
                    let run = state.resources.get::<Option<ArenaRun>>().unwrap().clone();
                    match run {
                        Some(r) => state.handle_arena_kill(r),
                        None => {
                            state.resources.insert(TurnState::AwaitingInput);
                        }
                    }
                }
                TurnState::ArenaTransition => {
                    // Same logic as arena_transition_tick, minus the
                    // rendering it also does.
                    let run = state
                        .resources
                        .get::<Option<ArenaRun>>()
                        .unwrap()
                        .clone()
                        .expect("ArenaTransition reached without an active ArenaRun");
                    state.arena_begin_wave(run.level, 1);
                }
                TurnState::ArenaWaveCleared => {
                    // Same logic as arena_wave_cleared_tick - shouldn't
                    // actually trigger given this bot always fights in
                    // real battles rather than using a ranged/Trap kill,
                    // but handled for correctness regardless.
                    let run = state
                        .resources
                        .get::<Option<ArenaRun>>()
                        .unwrap()
                        .clone()
                        .expect("ArenaWaveCleared reached without an active ArenaRun");
                    state.handle_arena_kill(run);
                }
                TurnState::GameOver => {
                    let run = state.resources.get::<Option<ArenaRun>>().unwrap().clone();
                    return match run {
                        Some(r) => ArenaOutcome::Died {
                            level: r.level,
                            wave: r.wave,
                            boss_active: r.boss_active,
                        },
                        None => ArenaOutcome::Died {
                            level: 0,
                            wave: 0,
                            boss_active: false,
                        },
                    };
                }
                TurnState::Victory => {
                    return ArenaOutcome::Won;
                }
                _ => {}
            }
        }
        ArenaOutcome::TimedOut
    }

    #[test]
    #[ignore]
    fn arena_class_survivability_report() {
        let classes = ["Barbarian", "Mage", "Rogue", "Amazon", "Hunter"];
        let runs_per_class = 10;
        let mut report = String::new();
        for class in classes {
            let mut won = 0;
            let mut timed_out = 0;
            let mut deaths: Vec<(u8, u8, bool)> = Vec::new();
            for _ in 0..runs_per_class {
                match simulate_one_arena_run(class, 6000) {
                    ArenaOutcome::Won => won += 1,
                    ArenaOutcome::TimedOut => timed_out += 1,
                    ArenaOutcome::Died {
                        level,
                        wave,
                        boss_active,
                    } => deaths.push((level, wave, boss_active)),
                }
            }
            let death_summary = if deaths.is_empty() {
                "none".to_string()
            } else {
                deaths
                    .iter()
                    .map(|(level, wave, boss_active)| {
                        if *boss_active {
                            format!("L{level} boss")
                        } else {
                            format!("L{level}W{wave}")
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            report.push_str(&format!(
                "{class}: {won}/{runs_per_class} won the full run ({timed_out} timed out) - deaths: {death_summary}\n"
            ));
        }
        println!("\n{}", report);
    }
}
