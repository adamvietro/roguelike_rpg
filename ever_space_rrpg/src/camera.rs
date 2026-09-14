use crate::prelude::*;

/// `right_x`/`bottom_y` are each other's exclusive/inclusive opposite by
/// convention, not a typo: `map_render.rs`/`entity_render.rs` consume
/// `left_x..right_x` (EXCLUSIVE - `right_x = left_x + DISPLAY_WIDTH`
/// gives exactly `DISPLAY_WIDTH` columns) but `top_y..=bottom_y`
/// (INCLUSIVE - so `bottom_y` must be `top_y + DISPLAY_HEIGHT - 1`, one
/// LESS than the X-axis pattern, to give exactly `DISPLAY_HEIGHT` rows).
/// Confirmed as a real, shipped regression (2026-09-11): `bottom_y` was
/// briefly defined the same way as `right_x` (`top_y + DISPLAY_HEIGHT`,
/// no `- 1`) when the camera-clamping rework replaced the old symmetric
/// `player.y +/- DISPLAY_HEIGHT/2` formula - that old formula happened
/// to produce a correct 25-row span purely because `DISPLAY_HEIGHT` (25)
/// is odd, masking that `map_render.rs`'s Y-loop was ever inclusive at
/// all. The off-by-one silently dropped the camera viewport's entire
/// bottom row while the player stood still - `SimpleConsole::set`
/// bounds-checks and drops out-of-range writes with no panic, so nothing
/// crashed - and only that row's own tiles/entities re-appeared for the
/// ~220ms of any glide, since the mid-glide render path uses a
/// `FlexiConsole`/`set_fancy`, which has no such bounds check. Reported
/// live as "we only see the counter and the stairs when the character is
/// moving" - not actually specific to those two tile types, just
/// whatever happened to be sitting on the clipped row at the time.
pub struct Camera {
    pub left_x: i32,
    pub right_x: i32,
    pub top_y: i32,
    pub bottom_y: i32,
    /// The area the DISPLAY_WIDTH x DISPLAY_HEIGHT window is clamped to
    /// stay within - defaults to the full SCREEN_WIDTH x SCREEN_HEIGHT map
    /// via `new`/`on_player_move`, but a map that only reveals a smaller
    /// "reveal rectangle" inside that same 80x50 grid (see `MapBuilder::
    /// new_arena_wave`/`new_arena_shop` - a deliberately small Arena map
    /// reads as small because everything outside its own reveal rectangle
    /// stays permanently unrevealed Wall, not because the underlying `Map`
    /// itself is actually any smaller) needs the SAME smaller area here
    /// too, via `new_bounded` - otherwise the camera can still pan past
    /// the reveal rectangle's own real edge into that unrevealed space,
    /// which renders as black since nothing ever draws there. Confirmed
    /// as a real, live bug on Arena's wave map specifically (2026-09-14) -
    /// a player near the clearing's own edge (still well within the full
    /// 80x50 map's outer SCREEN_WIDTH/HEIGHT bounds, so the old clamp
    /// never engaged) exposed real black space past the reveal
    /// rectangle's right edge.
    bounds_x0: i32,
    bounds_y0: i32,
    bounds_w: i32,
    bounds_h: i32,
}

impl Camera {
    /// The standard camera for a real full-size map (every ordinary
    /// dungeon floor) - clamped against the whole SCREEN_WIDTH x
    /// SCREEN_HEIGHT grid, same as before this struct grew bounds
    /// awareness. See `new_bounded` for a map with a smaller reveal
    /// rectangle.
    pub fn new(player_position: Point) -> Self {
        Self::new_bounded(player_position, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT)
    }

    /// Same as `new`, but the camera's DISPLAY_WIDTH x DISPLAY_HEIGHT
    /// window is clamped to stay within (bounds_x0, bounds_y0, bounds_w,
    /// bounds_h) instead of the full SCREEN_WIDTH x SCREEN_HEIGHT map -
    /// pass the exact same reveal rectangle `MapBuilder::new_arena_wave`/
    /// `new_arena_shop` already returns (and that `reveal_and_freeze_fov`
    /// already consumes) for any map built that way. See this struct's
    /// own `bounds_*` field doc comment for why this exists.
    pub fn new_bounded(
        player_position: Point,
        bounds_x0: i32,
        bounds_y0: i32,
        bounds_w: i32,
        bounds_h: i32,
    ) -> Self {
        let mut camera = Self {
            left_x: 0,
            right_x: DISPLAY_WIDTH,
            top_y: 0,
            bottom_y: DISPLAY_HEIGHT - 1,
            bounds_x0,
            bounds_y0,
            bounds_w,
            bounds_h,
        };
        camera.on_player_move(player_position);
        camera
    }

    pub fn on_player_move(&mut self, player_position: Point) {
        let (left_x, top_y) = self.clamped_top_left(player_position);
        self.left_x = left_x;
        self.right_x = left_x + DISPLAY_WIDTH;
        self.top_y = top_y;
        self.bottom_y = top_y + DISPLAY_HEIGHT - 1;
    }

    /// The camera's top-left corner if centered on `target`, clamped so
    /// the DISPLAY_WIDTH x DISPLAY_HEIGHT window never extends past this
    /// camera's own `bounds_*` (the whole map for an ordinary floor, or a
    /// smaller reveal rectangle for an Arena map - see that field's own
    /// doc comment) - the black void previously visible whenever a target
    /// point was close enough to any map edge (confirmed real both in
    /// normal dungeon-crawl play and, worse, on the title screen's
    /// decorative background, whose one-shot camera placement uses
    /// whatever random point a map architect picked as `player_start`,
    /// frequently nowhere near the map's actual center). `target` is no
    /// longer guaranteed to land exactly in the center of the resulting
    /// window once clamped - that trade-off (the player's own on-screen
    /// position shifts off-center near an edge, rather than ever showing
    /// black past the bounds' real edge) is deliberate.
    ///
    /// Public (not just used internally by `new`/`on_player_move`) so
    /// `components::camera_render_offset` can apply the exact same clamp,
    /// against this SAME camera's own bounds, to a glide's start/end tile
    /// before interpolating between them - otherwise the smooth sub-pixel
    /// pan during a step near a map/reveal-rectangle edge would disagree
    /// with where the discrete camera actually ends up once the step
    /// commits.
    pub fn clamped_top_left(&self, target: Point) -> (i32, i32) {
        let left_x = Self::clamp_axis(target.x, DISPLAY_WIDTH, self.bounds_x0, self.bounds_w);
        let top_y = Self::clamp_axis(target.y, DISPLAY_HEIGHT, self.bounds_y0, self.bounds_h);
        (left_x, top_y)
    }

    /// One axis of `clamped_top_left`'s clamp. When `bounds_len` is at
    /// least as wide/tall as the display window, this is a plain
    /// center-on-target-then-clamp, same as the original unbounded
    /// formula. When `bounds_len` is SMALLER than the display window (the
    /// wave map's reveal rectangle is 34 wide vs. DISPLAY_WIDTH's 40 -
    /// `new_arena_wave`'s own REVEAL_W/REVEAL_H doc comment already notes
    /// this margin is intentional), there's no position where the window
    /// could ever fully fit inside the bounds at all - a plain `.clamp()`
    /// with min > max would panic - so this centers the window on the
    /// bounds instead and ignores `target` entirely for this axis; the
    /// small, fixed margin on both sides is the deliberate cosmetic
    /// framing already described there, not something to pan away from.
    fn clamp_axis(target: i32, display_len: i32, bounds_0: i32, bounds_len: i32) -> i32 {
        if bounds_len <= display_len {
            bounds_0 - (display_len - bounds_len) / 2
        } else {
            (target - display_len / 2).clamp(bounds_0, bounds_0 + bounds_len - display_len)
        }
    }
}
