use crate::prelude::*;

pub struct Camera {
    pub left_x: i32,
    pub right_x: i32,
    pub top_y: i32,
    pub bottom_y: i32,
}

impl Camera {
    pub fn new(player_position: Point) -> Self {
        let (left_x, top_y) = Camera::clamped_top_left(player_position);
        Self {
            left_x,
            right_x: left_x + DISPLAY_WIDTH,
            top_y,
            bottom_y: top_y + DISPLAY_HEIGHT,
        }
    }

    pub fn on_player_move(&mut self, player_position: Point) {
        let (left_x, top_y) = Camera::clamped_top_left(player_position);
        self.left_x = left_x;
        self.right_x = left_x + DISPLAY_WIDTH;
        self.top_y = top_y;
        self.bottom_y = top_y + DISPLAY_HEIGHT;
    }

    /// The camera's top-left corner if centered on `target`, clamped so
    /// the DISPLAY_WIDTH x DISPLAY_HEIGHT window never extends past the
    /// map's own fixed SCREEN_WIDTH x SCREEN_HEIGHT bounds - the black
    /// void previously visible whenever a target point was close enough
    /// to any map edge (confirmed real both in normal dungeon-crawl play
    /// and, worse, on the title screen's decorative background, whose
    /// one-shot camera placement uses whatever random point a map
    /// architect picked as `player_start`, frequently nowhere near the
    /// map's actual center). `target` is no longer guaranteed to land
    /// exactly in the center of the resulting window once clamped - that
    /// trade-off (the player's own on-screen position shifts off-center
    /// near an edge, rather than ever showing black past the map's real
    /// bounds) is deliberate.
    ///
    /// Public (not just used internally by `new`/`on_player_move`) so
    /// `components::camera_render_offset` can apply the exact same clamp
    /// to a glide's start/end tile before interpolating between them -
    /// otherwise the smooth sub-pixel pan during a step near a map edge
    /// would disagree with where the discrete camera actually ends up
    /// once the step commits.
    pub fn clamped_top_left(target: Point) -> (i32, i32) {
        let left_x = (target.x - DISPLAY_WIDTH / 2).clamp(0, SCREEN_WIDTH - DISPLAY_WIDTH);
        let top_y = (target.y - DISPLAY_HEIGHT / 2).clamp(0, SCREEN_HEIGHT - DISPLAY_HEIGHT);
        (left_x, top_y)
    }
}
