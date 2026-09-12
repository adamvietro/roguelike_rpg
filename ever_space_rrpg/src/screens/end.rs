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

    /// Draws the player's own glyph, big, recolored solid `tint` rather
    /// than the entity's normal sprite color, so it reads as a silhouette
    /// (grey for defeat, gold for victory) instead of looking like an
    /// active battle portrait. Row is always the console's middle row;
    /// `col` lets victory() place the hero to one side of center instead
    /// of dead-center, so the Amulet (see draw_end_screen_amulet) can sit
    /// beside it without overlapping. Used by victory (upright); game_over
    /// uses draw_end_screen_fallen_portrait instead, which is rotated.
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
    fn draw_end_screen_portrait(&mut self, col: i32, tint: RGB) {
        let player = <(Entity, &Player)>::query()
            .iter(&self.ecs)
            .map(|(e, _)| *e)
            .nth(0);
        let render = player.and_then(|p| entity_render_component(&self.ecs, p));
        if let (Some(render), Some(anim)) = (render, self.victory_animation.as_ref()) {
            let mut victory = DrawBatch::new();
            victory.target(CHARACTER_VICTORY_CONSOLE);
            victory.set(
                Point::new(col, BATTLE_PORTRAIT_ROWS / 2),
                render.color,
                anim.current_glyph(),
            );
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
            portrait.set(
                Point::new(col, BATTLE_PORTRAIT_ROWS / 2),
                ColorPair::new(tint, BLACK),
                glyph,
            );
            portrait.submit(0).expect("Batch error");
        }
    }

    /// Draws the Amulet of Yala's own glyph ('|', see
    /// spawner::spawn_amulet_of_yala) big, on the same battle-portrait
    /// console/grid as draw_end_screen_portrait, at `col` - used by
    /// victory() to show it beside the hero. Hardcodes the glyph rather
    /// than looking up the actual AmuletOfYala entity, since nothing
    /// guarantees that entity still exists in the ECS by the time the
    /// Victory screen is showing (the run is already over) - the glyph
    /// itself is a fixed constant either way, so there's nothing gained
    /// by depending on the entity still being present.
    fn draw_end_screen_amulet(&mut self, col: i32, tint: RGB) {
        let mut amulet = DrawBatch::new();
        amulet.target(BATTLE_PORTRAIT_CONSOLE);
        amulet.set(
            Point::new(col, BATTLE_PORTRAIT_ROWS / 2),
            ColorPair::new(tint, BLACK),
            to_cp437('|'),
        );
        amulet.submit(0).expect("Batch error");
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

    /// Same lazy-build-then-tick shape as tick_death_animation, for
    /// self.victory_animation instead - see that field's own doc comment.
    fn tick_victory_animation(&mut self, ctx: &BTerm) {
        if self.victory_animation.is_none() {
            let class = <(Entity, &Player)>::query()
                .iter(&self.ecs)
                .map(|(e, _)| *e)
                .nth(0)
                .and_then(|p| entity_class(&self.ecs, p));
            self.victory_animation = class.and_then(|class| victory_animation_for_class(&class));
        }
        if let Some(anim) = self.victory_animation.as_mut() {
            anim.tick(ctx.frame_time_ms);
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
    fn draw_end_screen_fallen_portrait(&mut self, icon_tint: RGB) {
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
        if let (Some(render), Some(anim)) = (render, self.death_animation.as_ref()) {
            let mut death = DrawBatch::new();
            death.target(CHARACTER_DEATH_CONSOLE);
            death.set(
                Point::new(1, BATTLE_PORTRAIT_ROWS / 2),
                render.color,
                anim.current_glyph(),
            );
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
        // Red, dimmed version of the actual dungeon the run ended in,
        // plus the fallen hero's own glyph - rotated onto its side, tinted
        // red, with a genuinely transparent background this time (see
        // draw_end_screen_fallen_portrait) instead of an opaque quad
        // matched to the arena color.
        self.tick_death_animation(ctx);
        self.draw_end_screen_background(RGB::from_f32(1.0, 0.4, 0.4));
        self.draw_end_screen_fallen_portrait(RED.into());

        // Header on BIG_TEXT_CONSOLE (32px cells - same
        // one the title screen's "EVER SPACE RRPG" uses). Body text below
        // it moved from console 2 (fine 8px) to HUD_CONSOLE (the HUD
        // console, ~12px cells - the same "1.5x bigger" text already used
        // for the dungeon HUD) so it isn't dwarfed by the header, and row
        // positions are worked out in pixels (not row counts) so nothing
        // overlaps across these differently-scaled consoles: header row 2
        // on BIG_TEXT_CONSOLE bottoms out at (2+1)*32 = 96px -> HUD_CONSOLE row 9
        // (~108px) clears it; the fallen portrait (see
        // draw_end_screen_fallen_portrait) is centered at y=400px and, at
        // END_SCREEN_FALLEN_SCALE, spans roughly 304-496px -> HUD_CONSOLE
        // row 45 (~537px) clears its bottom edge with margin.
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
        // Below this point: the fallen portrait, centered on-screen (see
        // draw_end_screen_fallen_portrait). These two lines sit clear
        // beneath it.
        ctx.print_color_centered(
            45,
            YELLOW,
            BLACK,
            "Don't worry, you can always try again with a new hero.",
        );
        ctx.print_color_centered(
            48,
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

        // Warm gold version of the actual dungeon/arena the run was won
        // in, plus the hero's own glyph, glowing gold - see
        // draw_end_screen_background/draw_end_screen_portrait. The
        // Amulet of Yala icon only makes sense for a dungeon-crawl win -
        // an Arena win has no amulet at all, so it's skipped entirely
        // rather than drawing a prop that doesn't apply.
        self.tick_victory_animation(ctx);
        self.draw_end_screen_background(RGB::from_f32(1.0, 0.85, 0.45));
        self.draw_end_screen_portrait(1, YELLOW.into());
        if !is_arena {
            self.draw_end_screen_amulet(3, YELLOW.into());
        }

        // Same layout approach as game_over: header on the big-text
        // console (BIG_TEXT_CONSOLE, 32px cells), body on HUD_CONSOLE (the HUD
        // console, ~12px cells) positioned in real pixels to clear the
        // header above and the hero/Amulet icons below - see game_over's
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
        // Below this point: the hero (+ Amulet, dungeon-crawl only)
        // icons, centered on-screen.
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
