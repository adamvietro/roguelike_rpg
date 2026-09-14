use crate::prelude::*;
use crate::State;

impl State {
    /// Fills console 0 with the current run's dungeon theme (floor/wall
    /// tiles + vignette - same ingredients as draw_battle_arena's
    /// background, minus the battle-specific scenery/border), tinted
    /// toward `tint`. Used by game_over/victory so those screens show a
    /// moody dimmed/glowing version of the actual dungeon instead of a
    /// flat black backdrop - the map/theme resources are still the
    /// current run's, since neither death nor victory wipes them
    /// (only return_to_title does, once the player dismisses the screen).
    fn draw_end_screen_background(&mut self, tint: RGB) {
        let theme = self.resources.get::<Box<dyn MapTheme>>().unwrap();
        let floor_glyph = theme.tile_to_render(TileType::Floor);
        let wall_glyph = theme.tile_to_render(TileType::Wall);
        let floor_base = tint_color(theme.floor_color(), tint);
        let wall_base = tint_color(theme.wall_color(), tint);
        drop(theme);

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
        arena.submit(0).expect("Batch error");
    }

    /// Draws a real painted End-screen backdrop when `row` is `Some`
    /// (one glyph on BATTLE_BACKDROP_CONSOLE/`battle_backgrounds.png`,
    /// exactly the same "one glyph = one full-screen image" mechanism
    /// draw_battle_arena uses for its own themed backgrounds - see
    /// components::VictoryBackground/DefeatBackground's own doc comments
    /// for why this shares that console/atlas rather than needing a new
    /// one), falling back to the old procedural tinted-fill background
    /// (draw_end_screen_background) when `row` is `None` - a variant
    /// without real art yet.
    fn draw_end_screen_backdrop(&mut self, row: Option<u16>, tint: RGB) {
        match row {
            Some(row) => {
                let mut backdrop = DrawBatch::new();
                backdrop.target(BATTLE_BACKDROP_CONSOLE);
                backdrop.set(Point::new(0, 0), ColorPair::new(WHITE, BLACK), row as FontCharType);
                backdrop.submit(0).expect("Batch error");
            }
            None => self.draw_end_screen_background(tint),
        }
    }

    /// Draws the player's own glyph, big, recolored solid `tint` rather
    /// than the entity's normal sprite color, so it reads as a silhouette
    /// (grey for defeat, gold for victory) instead of looking like an
    /// active battle portrait. Position comes from the active
    /// VictoryBackground's own portrait_grid_position/walk_away_position
    /// (see components::VictoryBackground) rather than a fixed spot -
    /// each painted scene wants the hero somewhere different, and
    /// (2026-09-13) there's no longer an Amulet icon needing dead-center
    /// to stay clear for. Used by victory (upright); game_over uses
    /// draw_end_screen_fallen_portrait instead, which is rotated.
    ///
    /// Prefers a real played-once victory animation from
    /// character_victory.png (self.victory_animation, see
    /// tick_victory_animation - called by victory() before this) over the
    /// class's own still portrait from character_portrait.png
    /// (CHARACTER_PORTRAIT_BIG_CONSOLE - same lookup draw_battle_arena
    /// uses), falling back further to the old dungeonfont glyph (console
    /// 3) for a class with neither. The animation draws in the player's
    /// own render color (WHITE - full color, since the sheet already has
    /// real color art) rather than the solid `tint` silhouette the two
    /// static fallbacks below still use.
    fn draw_end_screen_portrait(&mut self, tint: RGB) {
        let player = <(Entity, &Player)>::query()
            .iter(&self.ecs)
            .map(|(e, _)| *e)
            .nth(0);
        let render = player.and_then(|p| entity_render_component(&self.ecs, p));
        let background = self.victory_background.unwrap_or(VictoryBackground::arena());
        let pose = background.pose();

        // VictoryPose::WalkAway draws from a completely different sheet/
        // console (character_idle.png's own North-facing walk loop) than
        // the rest of this function's FaceCamera/ClimbAway path
        // (character_victory.png via CHARACTER_VICTORY_CONSOLE) - handled
        // first and returns early. Drawn via set_fancy on
        // CHARACTER_IDLE_GLIDE_CONSOLE (not the plain CHARACTER_IDLE_
        // CONSOLE) specifically for VICTORY_WALK_AWAY_SCALE - see that
        // constant's own doc comment for why a native 1x tile-scale
        // render read as an all-but-invisible speck (real user
        // screenshot, 2026-09-13). Position is background.
        // walk_away_position() - a first guess, still not confirmed
        // against real play at this new scale.
        if pose == VictoryPose::WalkAway {
            if let (Some(render), Some(anim)) = (render, self.victory_walk_animation.as_ref()) {
                let (x, y) = background.walk_away_position();
                let mut walk = DrawBatch::new();
                walk.target(CHARACTER_IDLE_GLIDE_CONSOLE);
                let bg_transparent = RGBA::from_f32(0.0, 0.0, 0.0, 0.0);
                walk.set_fancy(
                    PointF::new(x, y),
                    0,
                    Degrees::new(0.0),
                    PointF::new(VICTORY_WALK_AWAY_SCALE, VICTORY_WALK_AWAY_SCALE),
                    ColorPair::new(render.color.fg, bg_transparent),
                    anim.current_glyph(),
                );
                walk.submit(0).expect("Batch error");
                return;
            }
        }

        // VictoryPose::ClimbAway - same sheet/console/grid as the
        // FaceCamera path just below (character_victory.png via
        // CHARACTER_VICTORY_CONSOLE, same coarse BATTLE_PORTRAIT
        // placement), just a different row block and source animation
        // (self.victory_climb_animation instead of self.victory_
        // animation) - see components::victory_climb_animation_for_class.
        if pose == VictoryPose::ClimbAway {
            if let (Some(render), Some(anim)) = (render, self.victory_climb_animation.as_ref()) {
                let (col, row) = background.portrait_grid_position();
                let mut climb = DrawBatch::new();
                climb.target(CHARACTER_VICTORY_CONSOLE);
                climb.set(Point::new(col, row), render.color, anim.current_glyph());
                climb.submit(0).expect("Batch error");
                return;
            }
        }

        let (col, row) = background.portrait_grid_position();
        if let (Some(render), Some(anim)) = (render, self.victory_animation.as_ref()) {
            let mut victory = DrawBatch::new();
            victory.target(CHARACTER_VICTORY_CONSOLE);
            victory.set(Point::new(col, row), render.color, anim.current_glyph());
            victory.submit(0).expect("Batch error");
            return;
        }
        let portrait_glyph = player
            .and_then(|p| entity_class(&self.ecs, p))
            .and_then(|class| character_portrait_glyph(&class));
        if let Some(render) = render {
            let (console, glyph) = match portrait_glyph {
                Some(glyph) => (CHARACTER_PORTRAIT_BIG_CONSOLE, glyph),
                None => (3, render.glyph),
            };
            let mut portrait = DrawBatch::new();
            portrait.target(console);
            portrait.set(Point::new(col, row), ColorPair::new(tint, BLACK), glyph);
            portrait.submit(0).expect("Batch error");
        }
    }

    /// Lazily builds self.death_animation the first time this runs for a
    /// run that just ended in death (State::death_animation starts `None`
    /// and only return_to_title resets it - see that field's own doc
    /// comment), then advances it by ctx.frame_time_ms. A no-op once the
    /// animation has reached its last frame (OneShotAnimation::tick
    /// already handles that), so calling this every frame of game_over()
    /// is fine. Does nothing (leaves it `None`) for a class with no row
    /// on character_death.png yet - draw_end_screen_fallen_portrait falls
    /// back to the old rotated-glyph look in that case.
    fn tick_death_animation(&mut self, ctx: &BTerm) {
        if self.death_animation.is_none() {
            let class = <(Entity, &Player)>::query()
                .iter(&self.ecs)
                .map(|(e, _)| *e)
                .nth(0)
                .and_then(|p| entity_class(&self.ecs, p));
            self.death_animation = class.and_then(|class| death_animation_for_class(&class));
        }
        if let Some(anim) = self.death_animation.as_mut() {
            anim.tick(ctx.frame_time_ms);
        }
    }

    /// Lazily picks self.victory_background the first time this runs for
    /// a run that just ended in victory (Arena's is fixed; Dungeon
    /// Crawl's is randomized once, keyed to whichever `EndSceneTheme`
    /// the live `Box<dyn MapTheme>` resource resolves to, and then left
    /// alone - see VictoryBackground::arena/random_for_theme and that
    /// field's own doc comment on State), then lazy-build-then-ticks
    /// whichever of self.victory_animation/self.victory_walk_animation
    /// that background's pose() actually calls for - same shape as
    /// tick_death_animation otherwise.
    fn tick_victory_animation(&mut self, ctx: &BTerm) {
        if self.victory_background.is_none() {
            let is_arena = self
                .resources
                .get::<Option<ArenaRun>>()
                .map(|r| r.is_some())
                .unwrap_or(false);
            self.victory_background = Some(if is_arena {
                VictoryBackground::arena()
            } else {
                let theme = self.resources.get::<Box<dyn MapTheme>>().unwrap().end_scene_theme();
                VictoryBackground::random_for_theme(theme)
            });
        }
        match self.victory_background.unwrap().pose() {
            VictoryPose::WalkAway => {
                if self.victory_walk_animation.is_none() {
                    let class = <(Entity, &Player)>::query()
                        .iter(&self.ecs)
                        .map(|(e, _)| *e)
                        .nth(0)
                        .and_then(|p| entity_class(&self.ecs, p));
                    self.victory_walk_animation =
                        class.and_then(|class| victory_walk_away_animation(&class));
                }
                if let Some(anim) = self.victory_walk_animation.as_mut() {
                    anim.tick(ctx.frame_time_ms);
                }
            }
            VictoryPose::ClimbAway => {
                if self.victory_climb_animation.is_none() {
                    let class = <(Entity, &Player)>::query()
                        .iter(&self.ecs)
                        .map(|(e, _)| *e)
                        .nth(0)
                        .and_then(|p| entity_class(&self.ecs, p));
                    self.victory_climb_animation =
                        class.and_then(|class| victory_climb_animation_for_class(&class));
                }
                if let Some(anim) = self.victory_climb_animation.as_mut() {
                    anim.tick(ctx.frame_time_ms);
                }
            }
            VictoryPose::FaceCamera => {
                if self.victory_animation.is_none() {
                    let class = <(Entity, &Player)>::query()
                        .iter(&self.ecs)
                        .map(|(e, _)| *e)
                        .nth(0)
                        .and_then(|p| entity_class(&self.ecs, p));
                    self.victory_animation =
                        class.and_then(|class| victory_animation_for_class(&class));
                }
                if let Some(anim) = self.victory_animation.as_mut() {
                    anim.tick(ctx.frame_time_ms);
                }
            }
        }
    }

    /// Draws the player's own glyph rotated 90 degrees - lying on its side,
    /// for the GameOver screen specifically. draw_portrait/
    /// draw_end_screen_portrait can only place a glyph on a fixed grid
    /// cell, upright - there's no rotation available on a plain
    /// "simple console". Actual rotation needs bracket-terminal's "fancy
    /// console" feature (DrawBatch::set_fancy, on END_SCREEN_FALLEN_CONSOLE
    /// - see main()), which nothing in this project has used before now.
    ///
    /// set_fancy's rotation parameter needs `Into<Radians>`, not a plain
    /// f32 - confirmed by your build's own compiler error, which also
    /// confirmed bracket-geometry's `Degrees` type is the thing that
    /// converts into it.
    ///
    /// THE UNTESTED PART: a flat opaque background quad (from two earlier
    /// attempts, both confirmed by screenshot) can't blend into the
    /// radial vignette behind it no matter what color it's given - a flat
    /// rectangle inside a gradient always shows a seam. The real fix is a
    /// genuinely transparent background instead of a matched one. This
    /// tries that via `RGBA` with alpha 0, betting that ColorPair is
    /// actually built on RGBA under the hood (with plain RGB silently
    /// converting to alpha=1 opaque, which is consistent with every
    /// ColorPair::new(RGB, RGB) call elsewhere in this file compiling
    /// fine) rather than RGB-only - nothing in this codebase has needed
    /// alpha before, so this specific call is a genuine guess, not a
    /// proven pattern. If it doesn't compile, the error will say exactly
    /// what type is actually expected, and I can fix it precisely from
    /// that - or fall back to the "deliberate framed plaque" approach if
    /// transparency turns out not to be available here at all.
    fn draw_end_screen_fallen_portrait(&mut self, defeat_background: DefeatBackground, icon_tint: RGB) {
        let player = <(Entity, &Player)>::query()
            .iter(&self.ecs)
            .map(|(e, _)| *e)
            .nth(0);
        let render = player.and_then(|p| entity_render_component(&self.ecs, p));
        // A real played-once death animation (self.death_animation, see
        // tick_death_animation - called by game_over() before this) takes
        // priority over the rotated-glyph fallback below: the animation
        // already shows a natural top-down collapse, drawn upright (no
        // rotation, no scale, full color) on the coarse BATTLE_PORTRAIT
        // grid just like draw_end_screen_portrait's victory animation.
        // Position comes from the active DefeatBackground's own
        // fallen_portrait_position (see components::DefeatBackground) -
        // fixes a real bug found 2026-09-13 via screenshot: this used to
        // be a fixed col=1, left off-center on the Arena courtyard's own
        // centered staircase, never updated when Victory's Amulet-of-
        // Yala icon (which col=1 used to leave room for) was removed.
        // Drawn via set_fancy on CHARACTER_DEATH_GLIDE_CONSOLE (not the
        // whole-cell-only CHARACTER_DEATH_CONSOLE used before) - a
        // second real screenshot round on Swamp's own Defeat scene
        // proved the coarse integer grid alone couldn't place the corpse
        // on solid ground AND clear of the frame edge at once. Reuses
        // the same draw_portrait_fancy helper the multi-enemy battle
        // formation already relies on for fractional positions on this
        // exact grid shape.
        if let (Some(render), Some(anim)) = (render, self.death_animation.as_ref()) {
            let (col, row) = defeat_background.fallen_portrait_position();
            let mut death = DrawBatch::new();
            death.target(CHARACTER_DEATH_GLIDE_CONSOLE);
            draw_portrait_fancy(&mut death, col, row, Render { color: render.color, glyph: anim.current_glyph() });
            death.submit(0).expect("Batch error");
            return;
        }
        let portrait_glyph = player
            .and_then(|p| entity_class(&self.ecs, p))
            .and_then(|class| character_portrait_glyph(&class));
        if let Some(render) = render {
            let cx = DISPLAY_WIDTH / 2;
            let cy = DISPLAY_HEIGHT / 2;

            let (console, glyph) = match portrait_glyph {
                Some(glyph) => (END_SCREEN_FALLEN_PORTRAIT_CONSOLE, glyph),
                None => (END_SCREEN_FALLEN_CONSOLE, render.glyph),
            };
            let mut fallen = DrawBatch::new();
            fallen.target(console);
            // Center of the console's square-celled DISPLAY_WIDTH x
            // DISPLAY_HEIGHT grid (same 32px cells as the main dungeon
            // view, so a 90-degree turn doesn't stretch/squash the glyph).
            let fg: RGBA = icon_tint.into();
            let bg: RGBA = RGBA::from_f32(0.0, 0.0, 0.0, 0.0);
            fallen.set_fancy(
                PointF::new(cx as f32, cy as f32),
                0,
                Degrees::new(90.0),
                PointF::new(END_SCREEN_FALLEN_SCALE, END_SCREEN_FALLEN_SCALE),
                ColorPair::new(fg, bg),
                glyph,
            );
            fallen.submit(0).expect("Batch error");
        }
    }

    /// Called from main.rs's tick() dispatcher, so this needs to be `pub`.
    pub fn game_over(&mut self, ctx: &mut BTerm) {
        // Which painted defeat scene matches this run's own theme/mode
        // (see components::DefeatBackground) - deterministic (no
        // randomization the way Victory's background gets), so no
        // caching needed either; recomputed every frame, harmlessly
        // cheap either way. Paired with the fallen hero's own glyph -
        // rotated onto its side, tinted red, with a genuinely
        // transparent background this time (see draw_end_screen_fallen_
        // portrait) instead of an opaque quad matched to the arena color.
        let is_arena = self
            .resources
            .get::<Option<ArenaRun>>()
            .map(|r| r.is_some())
            .unwrap_or(false);
        let defeat_background = if is_arena {
            DefeatBackground::arena()
        } else {
            let theme = self.resources.get::<Box<dyn MapTheme>>().unwrap().end_scene_theme();
            DefeatBackground::for_theme(theme)
        };

        self.tick_death_animation(ctx);
        self.draw_end_screen_backdrop(
            defeat_background.background_row(),
            RGB::from_f32(1.0, 0.4, 0.4),
        );
        self.draw_end_screen_fallen_portrait(defeat_background, RED.into());

        // Header on BIG_TEXT_CONSOLE (32px cells - same
        // one the title screen's "EVER SPACE RRPG" uses). Body text below
        // it moved from console 2 (fine 8px) to HUD_CONSOLE (the HUD
        // console, ~12px cells - the same "1.5x bigger" text already used
        // for the dungeon HUD) so it isn't dwarfed by the header, and row
        // positions are worked out in pixels (not row counts) so nothing
        // overlaps across these differently-scaled consoles: header row 2
        // on BIG_TEXT_CONSOLE bottoms out at (2+1)*32 = 96px -> HUD_CONSOLE row 9
        // (~108px) clears it.
        ctx.set_active_console(BIG_TEXT_CONSOLE);
        ctx.print_color_centered(2, RED, BLACK, "Your quest has ended.");

        ctx.set_active_console(HUD_CONSOLE);
        ctx.print_color_centered(
            10,
            WHITE,
            BLACK,
            "Slain by a monster, your hero's journey has come to a premature end.",
        );
        ctx.print_color_centered(
            13,
            WHITE,
            BLACK,
            "The Amulet of Yala remains unclaimed, and your home town is not saved.",
        );
        // "Don't worry, you can always try again..." removed (2026-09-13,
        // explicit user call) - it sat at row 45, which the fallen
        // portrait's own per-background position (see components::
        // DefeatBackground::fallen_portrait_position) can overlap depending
        // on the active theme, and it wasn't earning its own line either
        // way. "Press Enter..." alone now sits at row 60 - same row
        // Victory's own equivalent line uses (see victory() below), so
        // the two end screens read consistently instead of putting this
        // line in two different places.
        ctx.print_color_centered(
            60,
            GREEN,
            BLACK,
            "Press Enter to return to the title screen.",
        );

        // pending_enter_release guards against the same held Enter that
        // was down when the player took their fatal hit (e.g. holding
        // Enter to keep queuing attacks under True ATB while an enemy's
        // counter-attack finishes them off) also instantly dismissing
        // this screen - see that field's own doc comment on State and
        // screens/battle.rs's dismiss_action_result, which arms it.
        if ctx.key == Some(VirtualKeyCode::Return) && !self.pending_enter_release {
            self.return_to_title();
        }
    }

    /// Called from main.rs's tick() dispatcher, so this needs to be `pub`.
    pub fn victory(&mut self, ctx: &mut BTerm) {
        let is_arena = self
            .resources
            .get::<Option<ArenaRun>>()
            .map(|r| r.is_some())
            .unwrap_or(false);

        // Warm gold version of the actual painted scene this run was won
        // in/on (see components::VictoryBackground - tick_victory_
        // animation picks it, Arena fixed/Dungeon Crawl randomized, the
        // instant Victory is entered), plus the hero's own glyph, glowing
        // gold - see draw_end_screen_backdrop/draw_end_screen_portrait.
        // No Amulet of Yala icon anymore (removed 2026-09-13) - it was
        // designed for the old plain-fill background and read as a
        // mismatched flat glyph next to these painted scenes; the body
        // text below already carries that narrative beat on its own.
        self.tick_victory_animation(ctx);
        let background_row = self
            .victory_background
            .and_then(VictoryBackground::background_row);
        self.draw_end_screen_backdrop(background_row, RGB::from_f32(1.0, 0.85, 0.45));
        self.draw_end_screen_portrait(YELLOW.into());

        // Same layout approach as game_over: header on the big-text
        // console (BIG_TEXT_CONSOLE, 32px cells), body on HUD_CONSOLE (the HUD
        // console, ~12px cells) positioned in real pixels to clear the
        // header above and the hero's own icon below - see game_over's
        // comment for the exact pixel math this mirrors. "Press 1..." is
        // pushed down near the bottom of the screen instead of sitting
        // right under the body text.
        ctx.set_active_console(BIG_TEXT_CONSOLE);
        ctx.print_color_centered(2, GREEN, BLACK, "You have won!");

        ctx.set_active_console(HUD_CONSOLE);
        if is_arena {
            ctx.print_color_centered(
                10,
                WHITE,
                BLACK,
                "You cleared all three levels of the Battle Arena!",
            );
            ctx.print_color_centered(
                13,
                WHITE,
                BLACK,
                "Every wave, every boss - the arena is yours.",
            );
        } else {
            ctx.print_color_centered(
                10,
                WHITE,
                BLACK,
                "You put on the Amulet of Yala and feel its power course through your veins.",
            );
            ctx.print_color_centered(
                13,
                WHITE,
                BLACK,
                "Your town is saved, and you can return to your normal life.",
            );
        }
        // Below this point: the hero's own icon, positioned per the
        // active background (see components::VictoryBackground::
        // portrait_grid_position/walk_away_position).
        ctx.print_color_centered(
            60,
            GREEN,
            BLACK,
            "Press Enter to return to the title screen.",
        );

        if ctx.key == Some(VirtualKeyCode::Return) {
            self.return_to_title();
        }
    }
}
