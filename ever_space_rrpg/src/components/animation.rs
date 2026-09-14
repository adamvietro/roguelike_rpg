use crate::prelude::*;

/// How long a single tile-to-tile glide takes, in milliseconds. Shared by
/// tick_animations (systems/animation.rs, which advances/expires it) and
/// entity_render (which reads elapsed_ms to interpolate the drawn
/// position) so both stay in lockstep. Raised from 150 to 220 alongside
/// the FPS cap going from 30 to 60 (see main()'s BTermBuilder chain) -
/// together these give a glide roughly 3x the frames it had before
/// (~4-5 frames -> ~13), which is what actually fixed the visible
/// jumpiness; either change alone would have helped some, but not as
/// much as both together.
pub const MOVE_ANIM_DURATION_MS: f32 = 220.0;

/// Attached alongside the instant Point update in movement.rs so a
/// creature's *logical* position (and therefore FOV/turn-state/anything
/// else that reads Point) updates immediately, while entity_render draws
/// it sliding from `start` to `end` over MOVE_ANIM_DURATION_MS instead of
/// popping straight to the destination tile. Removed by tick_animations
/// once the glide finishes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MovingAnimation {
    pub start: Point,
    pub end: Point,
    pub elapsed_ms: f32,
}

/// How many "standing still" frame columns `resources/character_idle.png`
/// has per class row - see CHARACTER_IDLE_COLS, which is defined in terms
/// of this. Currently 6, matching Hunter's real PixelLab-exported Walk
/// cycle (the first class to get real walk-in-place art from that
/// pipeline) - Rogue/Amazon's own (older, different-pipeline) real frames
/// only had 5, so their 6th column is just a duplicate of their own first
/// frame rather than a genuinely distinct pose, and Barbarian/Mage's
/// placeholder rows repeat their single existing dungeon portrait across
/// all 6 regardless. Any Vec length up to this works with zero code
/// changes elsewhere in IdleAnimation itself.
pub const MAX_IDLE_FRAMES: usize = 6;

/// How many idle frames a class/enemy gets by default, and how long each
/// one shows before advancing to the next - see idle_frames_for. Every
/// entity in the game currently gets DEFAULT_IDLE_FRAME_COUNT frames that
/// all point at the exact same glyph as its base Render (see
/// idle_frames_for's own doc comment) - kept below MAX_IDLE_FRAMES so
/// there's room to grow a specific class/enemy up to 5 real, visually
/// distinct frames later without touching this constant.
pub const DEFAULT_IDLE_FRAME_COUNT: usize = 3;

pub const IDLE_FRAME_DURATION_MS: f32 = 350.0;

/// Which sprite sheet/console an IdleAnimation's `frames` glyphs are cells
/// in - see systems/entity_render.rs, which needs this to route each
/// entity's draw call to the matching console (CHARACTER_IDLE_CONSOLE's
/// trio for `CharacterIdle`, ENEMY_IDLE_CONSOLE's trio for `EnemyIdle`,
/// the plain dungeonfont ones for `Dungeon`).
/// Deliberately a field on the existing IdleAnimation component rather
/// than a new separate marker component - a new component would need its
/// own `#[read_component]` declaration added everywhere IdleAnimation is
/// already queried, exactly the class of legion access-panic this project
/// has been bitten by before (see CLAUDE.md); a new field on an
/// already-declared component needs no new declarations anywhere.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdleSpriteSheet {
    Dungeon,
    CharacterIdle,
    EnemyIdle,
    /// `resources/character_effect.png`'s own console trio
    /// (CHARACTER_EFFECT_CONSOLE/_SCROLL_/_GLIDE_) - added 2026-09-11
    /// alongside `EffectAnimation`. Not actually read off `IdleAnimation`
    /// like the other three variants are (`idle_sheet` in
    /// entity_render.rs returns this directly whenever an entity has an
    /// `EffectAnimation`, checked before it ever looks at `IdleAnimation`
    /// at all) - included in this enum anyway rather than a separate
    /// one, since it's still exactly the same "which console does this
    /// frame's glyph index belong to" question every other variant here
    /// answers.
    CharacterEffect,
    /// `resources/shopkeeper_idle.png`'s own console trio
    /// (SHOPKEEPER_IDLE_CONSOLE/_SCROLL_/_GLIDE_) - added 2026-09-13 so
    /// the Shopkeeper always plays its real `Idle_Selling` loop instead
    /// of a static glyph. Its own dedicated sheet/lookup domain rather
    /// than more rows on CharacterIdle/EnemyIdle, same reasoning enemies
    /// got their own sheet instead of more character_idle.png rows - the
    /// Shopkeeper is neither a playable class nor a real enemy. The
    /// GLIDE console specifically is never actually drawn to in
    /// practice (the Shopkeeper never moves, so gliding_position never
    /// returns Some for it) - kept anyway purely so this variant's match
    /// arms stay exhaustive/consistent with every sibling here, the same
    /// "correct even if provably unused" call this codebase already
    /// makes elsewhere.
    Shopkeeper,
}

/// Which of the 4 cardinal directions an entity most recently moved -
/// added 2026-09-11 alongside real directional Walk art, so the same
/// walk-cycle sheets (`character_idle.png`/`enemy_idle.png`, now 4 rows
/// per class/enemy instead of 1) can show the entity actually facing the
/// way it's moving instead of only ever facing `South`. Only 4-way, not
/// 8-way - `systems/player_input.rs` only ever produces a cardinal
/// `delta` (no diagonal movement exists in this game), so there's
/// nothing to derive a diagonal facing FROM. `South` is the default/
/// initial facing for a freshly spawned entity that hasn't moved yet -
/// matches this whole system's previous south-only-forever behavior
/// exactly, so a never-moved entity looks identical to before this.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    South,
    North,
    East,
    West,
}

impl Direction {
    /// The cardinal direction of a one-tile move from `start` to `end` -
    /// `None` if they're the same point (shouldn't happen for a real
    /// commited move, but this is a plain data function, not a panic
    /// site) or diagonal (shouldn't happen either - see this type's own
    /// doc comment - but handled by picking whichever axis actually
    /// moved, defensively, rather than assuming). systems/movement.rs
    /// calls this once per committed move to decide whether to rebuild
    /// the mover's IdleAnimation frames for a new facing.
    pub fn from_move(start: Point, end: Point) -> Option<Direction> {
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        if dx == 0 && dy == 0 {
            return None;
        }
        // Whichever axis moved further decides it (a defensive
        // tie-break for the diagonal case that shouldn't occur) -
        // vertical wins ties, an arbitrary but consistent choice.
        if dy.abs() >= dx.abs() {
            Some(if dy < 0 { Direction::North } else { Direction::South })
        } else {
            Some(if dx < 0 { Direction::West } else { Direction::East })
        }
    }
}

/// The "walking in place" idle loop: a small set of glyphs a stationary
/// entity cycles through, advancing one frame every IDLE_FRAME_DURATION_MS
/// (see systems/animation.rs's tick_idle_animation) and wrapping back to
/// frame 0 after the last. `sheet` says which console's font `frames`
/// indexes into (see IdleSpriteSheet) - both fields' actual values come
/// from whichever of idle_frames_for/idle_frames_for_class built this,
/// and `frames` gets REBUILT in place (same length, so `frame_index`
/// stays valid and isn't reset) by systems/movement.rs every time this
/// entity's Direction changes.
///
/// Advances during an in-flight MovingAnimation too, not just while
/// standing still (changed 2026-09-11, alongside real directional Walk
/// art) - this used to deliberately pause during a glide ("real movement
/// already has its own glide animation" - true of the POSITION tween,
/// but there was never a real per-frame walk cycle underneath it, so
/// pausing this just froze the character's pose for the whole glide,
/// reported as "the player and enemies become static" while moving).
/// Now this IS the real walk-cycle, playing continuously whether the
/// entity is idling in place or actually sliding between tiles - glide
/// time now counts toward each frame's IDLE_FRAME_DURATION_MS the same
/// as standing-still time does, so a walk cycle no longer full-stops for
/// the ~220ms of every single step.
#[derive(Clone, Debug, PartialEq)]
pub struct IdleAnimation {
    pub frames: Vec<FontCharType>,
    pub frame_index: usize,
    pub elapsed_ms: f32,
    pub sheet: IdleSpriteSheet,
}

impl IdleAnimation {
    /// The glyph this animation is currently showing.
    pub fn current_glyph(&self) -> FontCharType {
        self.frames
            .get(self.frame_index)
            .copied()
            .unwrap_or_default()
    }
}

/// Builds a fresh, dungeonfont-based placeholder IdleAnimation for a
/// creature whose base dungeon-view glyph is `base_glyph` - called for
/// every Enemy (spawner/template.rs's spawn_entity, which has no class to
/// look up a real sheet row for) and as the fallback for any player class
/// idle_frames_for_class doesn't recognize. Every frame is the SAME glyph
/// as the entity's base Render - a deliberate placeholder (the animation
/// is frame-complete and genuinely cycling under the hood, it just has no
/// visible effect) until real per-enemy walk-cycle art exists.
pub fn idle_frames_for(base_glyph: FontCharType) -> IdleAnimation {
    IdleAnimation {
        frames: vec![base_glyph; DEFAULT_IDLE_FRAME_COUNT],
        frame_index: 0,
        elapsed_ms: 0.0,
        sheet: IdleSpriteSheet::Dungeon,
    }
}

/// How many idle-loop frame columns `resources/character_idle.png` has
/// per class row - see that file's own doc comment in main.rs
/// (CHARACTER_IDLE_CONSOLE) for the full sheet layout, and
/// MAX_IDLE_FRAMES's own doc comment for why this is 6 rather than every
/// class actually having 6 genuinely distinct frames.
pub const CHARACTER_IDLE_COLS: u16 = MAX_IDLE_FRAMES as u16;

/// Which row a class occupies on `resources/character_portrait.png` -
/// `None` for anything without a row there. Safe to reuse a plain
/// 0-based row assignment here (unlike character_idle_row/
/// character_battle_row below) because character_portrait_glyph ONLY
/// EVER populates column 0 of a class's row - a plain console's `cls()`
/// default glyph (32) lands at column 32 % CHARACTER_IDLE_COLS == 2 on
/// this 6-column sheet, and column 2 is guaranteed blank on every row
/// regardless of which row that is, so there's no row this sheet
/// specifically needs to keep empty. (character_idle.png doesn't have
/// this luxury - it fills every column of a class's row with a real
/// walk-cycle frame - which is exactly why it needs its own
/// character_idle_row instead of sharing this one.)
pub fn class_sheet_row(class: &str) -> Option<u16> {
    match class {
        "Barbarian" => Some(0),
        "Rogue" => Some(1),
        "Amazon" => Some(2),
        "Hunter" => Some(3),
        "Mage" => Some(4),
        // Debug is the hidden dev/test class (see title.rs::class_select's
        // 'D' hotkey) - not in CLASS_ROSTER, but still a real Player
        // entity that needs real idle/portrait art like any other class.
        "Debug" => Some(5),
        _ => None,
    }
}

/// The exact `resources/character_portrait.png` glyph for `class`'s
/// single static still portrait - `None` if `class` has no row there
/// (see class_sheet_row). Column 0 only (this sheet only ever holds one
/// pose per class - PixelLab's own `rotations/south.png`).
pub fn character_portrait_glyph(class: &str) -> Option<FontCharType> {
    let row = class_sheet_row(class)?;
    Some(row * CHARACTER_IDLE_COLS)
}

/// Which row a class occupies on `resources/character_idle.png`
/// SPECIFICALLY - deliberately NOT class_sheet_row (see CLAUDE.md's
/// glyph-32 standing gotcha: a plain console's `cls()` default fills
/// row `32 / cols`, which real content can't safely occupy). This sheet
/// fills every column of a class's row with a real walk-cycle frame, so
/// its forbidden row (32 / 6 == 5) has to stay permanently unassigned.
/// 4 rows per class (one per `Direction`, since 2026-09-11's directional
/// Walk art) means Rogue's North would naturally land on row 5 - moved
/// to row 24 (appended after every class's block) instead. Shared
/// source of truth for `idle_frames_for_class` (spawn time) and the
/// Class Select screen's highlighted-class preview
/// (`screens/title.rs::class_select`).
fn character_idle_row(class: &str, direction: Direction) -> Option<u16> {
    let base = match class {
        "Barbarian" => 0,
        "Rogue" => 4,
        "Amazon" => 8,
        "Hunter" => 12,
        "Mage" => 16,
        "Debug" => 20,
        _ => return None,
    };
    if class == "Rogue" && direction == Direction::North {
        return Some(24);
    }
    let offset = match direction {
        Direction::South => 0,
        Direction::North => 1,
        Direction::East => 2,
        Direction::West => 3,
    };
    Some(base + offset)
}

/// The exact `resources/character_idle.png` glyph for `class` facing
/// `direction`, at idle frame `frame_index` (wrapped modulo
/// CHARACTER_IDLE_COLS, so any ever-increasing counter can be passed
/// directly) - `None` if `class` has no row on that sheet (see
/// character_idle_row).
pub fn character_idle_glyph(
    class: &str,
    frame_index: usize,
    direction: Direction,
) -> Option<FontCharType> {
    let row = character_idle_row(class, direction)?;
    let col = (frame_index as u16) % CHARACTER_IDLE_COLS;
    Some(row * CHARACTER_IDLE_COLS + col)
}

/// The full frame list for `class` facing `direction` on
/// `resources/character_idle.png` - `None` if `class` has no row there.
/// Shared by `idle_frames_for_class` (spawn time, builds a whole fresh
/// `IdleAnimation`) and `systems::movement` (rebuilds just the `frames`
/// field in place whenever an entity's facing changes, preserving
/// `frame_index`/`elapsed_ms` so a walk cycle doesn't restart mid-step
/// just because it turned a corner).
pub fn character_idle_frames(class: &str, direction: Direction) -> Option<Vec<FontCharType>> {
    let row = character_idle_row(class, direction)?;
    Some(
        (0..CHARACTER_IDLE_COLS)
            .map(|col| row * CHARACTER_IDLE_COLS + col)
            .collect(),
    )
}

/// How many battle-idle frame columns `resources/character_battle.png`
/// has per class row - 8, matching Hunter's real PixelLab-exported
/// Fight_Stance_Idle/east cycle (see main.rs's CHARACTER_BATTLE_CONSOLE
/// for the full sheet layout).
pub const CHARACTER_BATTLE_COLS: u16 = 8;

/// Which row a class occupies on `resources/character_battle.png`
/// SPECIFICALLY - its own mapping, not shared with character_idle_row/
/// class_sheet_row (see CLAUDE.md's glyph-32 gotcha: this sheet's
/// forbidden row is 32 / 8 == 4, different from character_idle.png's
/// row 5 purely because the column counts differ). Row 4 is left
/// blank; Mage moved to row 5 instead.
fn character_battle_row(class: &str) -> Option<u16> {
    match class {
        "Barbarian" => Some(0),
        "Rogue" => Some(1),
        "Amazon" => Some(2),
        "Hunter" => Some(3),
        // Row 4 deliberately skipped - see this fn's own doc comment.
        "Mage" => Some(5),
        "Debug" => Some(6),
        _ => None,
    }
}

/// The exact `resources/character_battle.png` glyph for `class`'s
/// battle-idle frame `frame_index` (wrapped modulo CHARACTER_BATTLE_COLS)
/// - `None` if `class` has no row on that sheet (see
/// character_battle_row). Used by the battle screen's own portrait loop,
/// NOT IdleAnimation - the player's battle portrait isn't a dungeon-view
/// entity, so this is driven by a plain frame counter on `Battle` itself
/// instead (see Battle::player_idle_frame).
pub fn character_battle_glyph(class: &str, frame_index: usize) -> Option<FontCharType> {
    let row = character_battle_row(class)?;
    let col = (frame_index as u16) % CHARACTER_BATTLE_COLS;
    Some(row * CHARACTER_BATTLE_COLS + col)
}

/// Builds a fresh IdleAnimation for a player class, pulling real frames
/// from `resources/character_idle.png` when `class` has a row there (see
/// character_idle_row - every current class does, including the hidden
/// Debug one). Falls back to the plain dungeonfont placeholder
/// (idle_frames_for) for anything without a row there - a future class
/// added without art yet. Called at spawn time with `Direction::South`
/// (a fresh entity hasn't moved yet) - systems::movement rebuilds
/// `frames` in place afterward via character_idle_frames whenever this
/// entity's facing actually changes.
pub fn idle_frames_for_class(class: &str, base_glyph: FontCharType) -> IdleAnimation {
    let frames = match character_idle_frames(class, Direction::South) {
        Some(f) => f,
        None => return idle_frames_for(base_glyph),
    };
    IdleAnimation {
        frames,
        frame_index: 0,
        elapsed_ms: 0.0,
        sheet: IdleSpriteSheet::CharacterIdle,
    }
}

/// How many idle-loop frame columns `resources/enemy_idle.png` has per
/// enemy row - see CHARACTER_IDLE_COLS's own doc comment; this sheet
/// follows the identical "one row per <thing>, MAX_IDLE_FRAMES columns"
/// convention, just for enemies on their own dedicated sheet instead of
/// playable classes (see docs/ideas.md's "Add PixelLab art for enemies"
/// backlog item, and the design conversation that preceded it: enemies
/// get their own sheets rather than more rows on the class sheets, since
/// those only have one free row left and enemies are a different lookup
/// domain entirely - keyed by name, not class).
pub const ENEMY_IDLE_COLS: u16 = MAX_IDLE_FRAMES as u16;

/// 4 rows per enemy (one per `Direction`, since 2026-09-11's directional
/// Walk art). Row 5 is this sheet's forbidden row (32 / 6 == 5, see
/// CLAUDE.md's glyph-32 gotcha), so Orc's own North (which would
/// naturally land there) moves to row 32 instead - same shape as
/// character_idle_row's Rogue/North exception.
fn enemy_idle_row(name: &str, direction: Direction) -> Option<u16> {
    let base = match name {
        "Goblin" => 0,
        "Orc" => 4,
        "Ogre" => 8,
        "Ettin" => 12,
        "Goblin Chieftain" => 16,
        // "Orc Warlord" was held back through 2026-09-08's first batch
        // (a genuine PixelLab generation defect - a thin off-model
        // sliver instead of a full character, on both Walk and
        // Fight_Stance_Idle) - the redo batch (same day) came back
        // clean, confirmed by screenshot. Its Walk/south came back with
        // 8 frames, more than this sheet's own 6-column ceiling
        // (MAX_IDLE_FRAMES) allows - 6 of the 8 were evenly sampled
        // (indices 0,1,3,4,6,7) rather than just truncated, so the walk
        // cycle doesn't visibly skip its back half. Now direction-aware
        // like every other enemy on this sheet.
        "Orc Warlord" => 20,
        "Ogre Warlord" => 24,
        "Ettin Overlord" => 28,
        _ => return None,
    };
    if name == "Orc" && direction == Direction::North {
        return Some(32);
    }
    let offset = match direction {
        Direction::South => 0,
        Direction::North => 1,
        Direction::East => 2,
        Direction::West => 3,
    };
    Some(base + offset)
}

/// How many battle-idle frame columns `resources/enemy_battle.png` has
/// per enemy row - see CHARACTER_BATTLE_COLS's own doc comment; same
/// convention, enemy-specific sheet.
pub const ENEMY_BATTLE_COLS: u16 = 8;

/// Which row an enemy occupies on `resources/enemy_battle.png`
/// SPECIFICALLY - its own mapping (see character_battle_row). Shares
/// character_battle.png's 8-column layout, so its forbidden row is also
/// 4 (32 / 8 == 4).
fn enemy_battle_row(name: &str) -> Option<u16> {
    match name {
        "Goblin" => Some(0),
        "Orc" => Some(1),
        "Ogre" => Some(2),
        "Ettin" => Some(3),
        // Row 4 deliberately skipped - see this fn's own doc comment.
        "Goblin Chieftain" => Some(5),
        // "Orc Warlord" - see enemy_idle_row's own doc comment for the
        // redo/defect history. Its Fight_Stance_Idle/south-west came
        // back with exactly 8 frames, matching this sheet's own column
        // count - no sampling needed here, unlike the idle sheet.
        "Orc Warlord" => Some(6),
        "Ogre Warlord" => Some(7),
        "Ettin Overlord" => Some(8),
        _ => None,
    }
}

/// The exact `resources/enemy_battle.png` glyph for enemy `name`'s
/// battle-idle frame `frame_index` (wrapped modulo ENEMY_BATTLE_COLS) -
/// `None` if `name` has no row on that sheet (see enemy_battle_row). Used
/// by the battle screen's own per-enemy portrait loop (screens/battle.rs)
/// the same way character_battle_glyph drives the player's.
pub fn enemy_battle_glyph(name: &str, frame_index: usize) -> Option<FontCharType> {
    let row = enemy_battle_row(name)?;
    let col = (frame_index as u16) % ENEMY_BATTLE_COLS;
    Some(row * ENEMY_BATTLE_COLS + col)
}

/// Which row an enemy occupies on `resources/enemy_attack.png` (added
/// 2026-09-11's full animation batch, giving enemies their own real
/// played-once attack animation instead of just their ordinary
/// Idle_Battle_Stance loop the whole time - see docs/ideas.md's
/// "Battle screen redesign" backlog item). Shares `enemy_battle_row`'s
/// exact layout/forbidden-row (both sheets use ENEMY_BATTLE_COLS), kept
/// as its own independent mapping per this file's usual convention.
fn enemy_attack_row(name: &str) -> Option<u16> {
    match name {
        "Goblin" => Some(0),
        "Orc" => Some(1),
        "Ogre" => Some(2),
        "Ettin" => Some(3),
        // Row 4 deliberately skipped - see enemy_battle_row's own doc
        // comment (identical reasoning, this sheet's own column count).
        "Goblin Chieftain" => Some(5),
        "Orc Warlord" => Some(6),
        "Ogre Warlord" => Some(7),
        "Ettin Overlord" => Some(8),
        _ => None,
    }
}

/// Builds a fresh `OneShotAnimation` playing enemy `name`'s attack
/// animation - `None` if `name` has no row yet, in which case the
/// caller keeps showing the ordinary Idle_Battle_Stance loop instead.
pub fn attack_animation_for_enemy(name: &str) -> Option<OneShotAnimation> {
    let row = enemy_attack_row(name)?;
    let frames = (0..ENEMY_BATTLE_COLS)
        .map(|col| row * ENEMY_BATTLE_COLS + col)
        .collect();
    Some(OneShotAnimation {
        frames,
        frame_index: 0,
        elapsed_ms: 0.0,
        frame_duration_ms: TECHNIQUE_FRAME_DURATION_MS,
        repeat: false,
    })
}

/// Which row an enemy occupies on `resources/enemy_death.png` (added
/// 2026-09-13, boss enemies only) - `None` for a basic enemy (Goblin/
/// Orc/Ogre/Ettin), which still just vanishes instantly on death rather
/// than playing anything, per the design decision recorded in
/// docs/ideas.md. Same `EXTRA_ANIM_COLS`-wide, forbidden-row-3 shape as
/// `character_death_row` rather than `ENEMY_BATTLE_COLS` - Death is a
/// played-once 9-frame animation like the player's own, not a looping
/// battle-idle.
fn enemy_death_row(name: &str) -> Option<u16> {
    match name {
        "Orc Warlord" => Some(0),
        "Ogre Warlord" => Some(1),
        "Ettin Overlord" => Some(2),
        // Row 3 deliberately skipped - see this fn's own doc comment.
        "Goblin Chieftain" => Some(4),
        _ => None,
    }
}

/// Builds a fresh `OneShotAnimation` playing enemy `name`'s death
/// sequence - `None` if `name` has no row (see `enemy_death_row`), in
/// which case the caller keeps the old instant-removal behavior.
pub fn death_animation_for_enemy(name: &str) -> Option<OneShotAnimation> {
    let row = enemy_death_row(name)?;
    let frames = (0..EXTRA_ANIM_COLS)
        .map(|col| row * EXTRA_ANIM_COLS + col)
        .collect();
    Some(OneShotAnimation {
        frames,
        frame_index: 0,
        elapsed_ms: 0.0,
        frame_duration_ms: IDLE_FRAME_DURATION_MS,
        repeat: false,
    })
}

/// The full frame list for enemy `name` facing `direction` on
/// `resources/enemy_idle.png` - `None` if `name` has no row there. Same
/// role as `character_idle_frames`, enemy side: shared by
/// `idle_frames_for_enemy` (spawn time) and `systems::movement` (rebuilds
/// `frames` in place on a facing change).
pub fn enemy_idle_frames(name: &str, direction: Direction) -> Option<Vec<FontCharType>> {
    let row = enemy_idle_row(name, direction)?;
    Some(
        (0..ENEMY_IDLE_COLS)
            .map(|col| row * ENEMY_IDLE_COLS + col)
            .collect(),
    )
}

/// How many frame columns `resources/shopkeeper_idle.png` has - its own
/// constant rather than reusing CHARACTER_IDLE_COLS/ENEMY_IDLE_COLS,
/// since this sheet holds the Shopkeeper's real 9-frame Idle_Selling
/// loop in full rather than sampling down to MAX_IDLE_FRAMES the way a
/// directional Walk cycle does - there's no walking here to justify that
/// budget, and the Shopkeeper never needs more than this one row.
pub const SHOPKEEPER_IDLE_COLS: u16 = 9;

/// Builds the Shopkeeper's permanent `IdleAnimation` - always the same
/// 9-frame Idle_Selling loop on row 0 of `resources/shopkeeper_idle.png`
/// (south-facing only; the Shopkeeper never turns, so unlike
/// character_idle_row/enemy_idle_row there's no Direction to key on).
/// Called once at spawn time (see spawner::spawn_shopkeeper) - never
/// rebuilt afterward the way a moving entity's frames are on a facing
/// change, since this entity never moves.
pub fn idle_frames_for_shopkeeper() -> IdleAnimation {
    let frames = (0..SHOPKEEPER_IDLE_COLS).collect();
    IdleAnimation {
        frames,
        frame_index: 0,
        elapsed_ms: 0.0,
        sheet: IdleSpriteSheet::Shopkeeper,
    }
}

/// Builds a fresh IdleAnimation for an enemy, pulling real walk-cycle
/// frames from `resources/enemy_idle.png` when `name` has a row there
/// (see enemy_idle_row) - falls back to the plain dungeonfont placeholder
/// (idle_frames_for) for any enemy without real art yet, the same
/// fallback idle_frames_for_class uses for an unrecognized class. Called
/// at spawn time with `Direction::South` - systems::movement rebuilds
/// `frames` afterward via enemy_idle_frames on a facing change.
pub fn idle_frames_for_enemy(name: &str, base_glyph: FontCharType) -> IdleAnimation {
    let frames = match enemy_idle_frames(name, Direction::South) {
        Some(f) => f,
        None => return idle_frames_for(base_glyph),
    };
    IdleAnimation {
        frames,
        frame_index: 0,
        elapsed_ms: 0.0,
        sheet: IdleSpriteSheet::EnemyIdle,
    }
}

/// A non-looping animation: plays through `frames` once, then holds on
/// the last one - the shape Death, Victory, and a single-hit technique's
/// own battle animation share, as opposed to `IdleAnimation`'s permanent
/// loop. Deliberately its own type rather than a flag on `IdleAnimation`
/// - those two only ever need the opposite behavior from each other,
/// and every existing `IdleAnimation` call site would need to start
/// handling a "hold at the end" case it never actually hits.
///
/// `repeat` (added 2026-09-08, explicit user feedback after seeing
/// Flurry's animation in a real fight) makes this loop back to frame 0
/// instead of holding once it reaches the end - for a multi-hit/AOE
/// technique, whose HitQueue keeps landing damage over a real span of
/// time far longer than one play-through, a single strike pose held
/// motionless for most of that span read as broken/frozen rather than an
/// ongoing flurry of hits. `frame_duration_ms` is baked in per-animation
/// rather than a shared global constant like `IdleAnimation`'s
/// `IDLE_FRAME_DURATION_MS`, since a technique needs a much faster,
/// punchier pace than a breathing idle stance or a held Death/Victory
/// pose - see `TECHNIQUE_FRAME_DURATION_MS`.
#[derive(Clone, Debug, PartialEq)]
pub struct OneShotAnimation {
    pub frames: Vec<FontCharType>,
    pub frame_index: usize,
    pub elapsed_ms: f32,
    pub frame_duration_ms: f32,
    pub repeat: bool,
}

impl OneShotAnimation {
    pub fn current_glyph(&self) -> FontCharType {
        self.frames
            .get(self.frame_index)
            .copied()
            .unwrap_or_default()
    }

    /// This animation's full one-time playthrough length, in ms - used to
    /// line up a combatant's post-hit color flash (see
    /// `Battle::player_flash`/`EnemyCombatant::flash`,
    /// `PORTRAIT_FLASH_DURATION_MS`) with the actual swing it belongs to,
    /// rather than the flash's own much shorter default duration fading
    /// out mid-swing. Meaningless for a `repeat` animation (which has no
    /// real "total" length) - not called for one today, since only multi-
    /// hit/AOE techniques repeat and those don't drive a flash this way.
    pub fn total_duration_ms(&self) -> f32 {
        self.frames.len() as f32 * self.frame_duration_ms
    }

    /// True once this animation has reached its last frame and has
    /// nothing left to advance to - always false for a `repeat` one,
    /// which by definition never reaches a permanent end.
    pub fn finished(&self) -> bool {
        !self.repeat && self.frame_index + 1 >= self.frames.len()
    }

    /// Advances by `dt_ms` of real time, at this animation's own
    /// `frame_duration_ms` pace - a no-op once `finished()`, so the
    /// caller never has to check that separately before ticking. A
    /// `repeat` animation wraps back to frame 0 instead of stopping.
    pub fn tick(&mut self, dt_ms: f32) {
        if self.finished() {
            return;
        }
        self.elapsed_ms += dt_ms;
        if self.elapsed_ms >= self.frame_duration_ms {
            self.elapsed_ms -= self.frame_duration_ms;
            self.frame_index += 1;
            if self.repeat && self.frame_index >= self.frames.len() {
                self.frame_index = 0;
            }
        }
    }
}

/// Columns on `resources/character_death.png` / `character_victory.png` /
/// `character_technique.png` - all three happen to share this column
/// count because Rogue's own Death/Victory/Flurry exports all came back
/// with exactly 9 frames; a future class/technique with more frames
/// would need this bumped (and the sheets rebuilt wider) the same way
/// any other sheet's column count has grown before.
pub const EXTRA_ANIM_COLS: u16 = 9;

/// Real ms each frame of a technique animation holds before advancing -
/// see `OneShotAnimation`'s own doc comment for why this needs its own,
/// much faster pace than `IDLE_FRAME_DURATION_MS` (350ms): at that pace,
/// a 9-frame technique animation like Flurry's would only get through
/// ~3 frames before `RESULT_AUTO_ADVANCE_MS` (1100ms) auto-dismisses a
/// single-hit ActionResult - an attack needs to read as fast and punchy,
/// not like a slow held pose.
pub const TECHNIQUE_FRAME_DURATION_MS: f32 = 80.0;

/// Which row a class occupies on `resources/character_attack.png` (the
/// generic basic-Attack one-shot animation, added 2026-09-11's full
/// animation batch - previously "Attack" just showed the ordinary
/// Fight_Stance/Idle_Battle_Stance loop with no animation of its own).
/// Same layout/forbidden-row convention as `character_battle_row`
/// (shares its 8-row-sheet shape, row 3 skipped - 32 / 9 == 3, the
/// EXTRA_ANIM_COLS column count this sheet actually uses, NOT
/// CHARACTER_BATTLE_COLS's own row 4).
fn character_attack_row(class: &str) -> Option<u16> {
    match class {
        "Barbarian" => Some(0),
        "Rogue" => Some(1),
        "Amazon" => Some(2),
        // Row 3 deliberately skipped - see this fn's own doc comment.
        "Hunter" => Some(4),
        "Mage" => Some(5),
        "Debug" => Some(6),
        _ => None,
    }
}

/// Builds a fresh `OneShotAnimation` playing `class`'s generic Attack
/// animation - `None` if `class` has no row yet, in which case the
/// caller keeps showing the ordinary battle-idle loop instead, same
/// fallback shape as `technique_animation_for`.
pub fn attack_animation_for_class(class: &str) -> Option<OneShotAnimation> {
    let row = character_attack_row(class)?;
    let frames = (0..EXTRA_ANIM_COLS)
        .map(|col| row * EXTRA_ANIM_COLS + col)
        .collect();
    Some(OneShotAnimation {
        frames,
        frame_index: 0,
        elapsed_ms: 0.0,
        frame_duration_ms: TECHNIQUE_FRAME_DURATION_MS,
        repeat: false,
    })
}

/// Which row a class occupies on `resources/character_defend.png` (the
/// generic Defend one-shot animation, added alongside `character_
/// attack_row` in the same batch - same layout/forbidden-row reasoning).
fn character_defend_row(class: &str) -> Option<u16> {
    match class {
        "Barbarian" => Some(0),
        "Rogue" => Some(1),
        "Amazon" => Some(2),
        // Row 3 deliberately skipped - see character_attack_row's own
        // doc comment for why (identical reasoning, this sheet's own
        // column count).
        "Hunter" => Some(4),
        "Mage" => Some(5),
        "Debug" => Some(6),
        _ => None,
    }
}

/// Builds a fresh `OneShotAnimation` playing `class`'s generic Defend
/// animation - `None` if `class` has no row yet.
pub fn defend_animation_for_class(class: &str) -> Option<OneShotAnimation> {
    let row = character_defend_row(class)?;
    let frames = (0..EXTRA_ANIM_COLS)
        .map(|col| row * EXTRA_ANIM_COLS + col)
        .collect();
    Some(OneShotAnimation {
        frames,
        frame_index: 0,
        elapsed_ms: 0.0,
        frame_duration_ms: TECHNIQUE_FRAME_DURATION_MS,
        repeat: false,
    })
}

/// Which row a class occupies on `resources/character_death.png` -
/// `None` for a class without one yet, same "grow as art arrives"
/// shape as every other per-class sheet. Row 3 is this sheet's own
/// forbidden row (32 / 9 == 3, see the glyph-32 gotcha in CLAUDE.md) -
/// skipped permanently, independent of character_idle_row/
/// character_battle_row's own forbidden rows on their different
/// column counts.
fn character_death_row(class: &str) -> Option<u16> {
    match class {
        "Rogue" => Some(0),
        "Debug" => Some(1),
        "Hunter" => Some(2),
        // Row 3 deliberately skipped - see this fn's own doc comment.
        "Barbarian" => Some(4),
        "Amazon" => Some(5),
        "Mage" => Some(6),
        _ => None,
    }
}

/// Builds a fresh `OneShotAnimation` playing `class`'s death sequence -
/// `None` if `class` has no row yet (caller keeps whatever fallback it
/// already had, e.g. the rotated-glyph approach).
pub fn death_animation_for_class(class: &str) -> Option<OneShotAnimation> {
    let row = character_death_row(class)?;
    let frames = (0..EXTRA_ANIM_COLS)
        .map(|col| row * EXTRA_ANIM_COLS + col)
        .collect();
    Some(OneShotAnimation {
        frames,
        frame_index: 0,
        elapsed_ms: 0.0,
        frame_duration_ms: IDLE_FRAME_DURATION_MS,
        repeat: false,
    })
}

/// Which row a class occupies on `resources/character_victory.png` -
/// same shape/forbidden-row (3) as `character_death_row`, own dedicated
/// mapping since this is a different sheet.
fn character_victory_row(class: &str) -> Option<u16> {
    match class {
        "Rogue" => Some(0),
        "Debug" => Some(1),
        "Hunter" => Some(2),
        // Row 3 deliberately skipped - see character_death_row's own
        // doc comment for why (identical reasoning, this sheet's own
        // column count).
        "Barbarian" => Some(4),
        "Amazon" => Some(5),
        "Mage" => Some(6),
        _ => None,
    }
}

/// Builds a fresh `OneShotAnimation` playing `class`'s victory pose -
/// `None` if `class` has no row yet.
pub fn victory_animation_for_class(class: &str) -> Option<OneShotAnimation> {
    let row = character_victory_row(class)?;
    let frames = (0..EXTRA_ANIM_COLS)
        .map(|col| row * EXTRA_ANIM_COLS + col)
        .collect();
    Some(OneShotAnimation {
        frames,
        frame_index: 0,
        elapsed_ms: 0.0,
        frame_duration_ms: IDLE_FRAME_DURATION_MS,
        repeat: false,
    })
}

/// Which row a class occupies on `resources/character_victory.png` for
/// its `VictoryPose::ClimbAway` pose (2026-09-13) - a second block of 6
/// rows (8-13) appended after the original 8-row sheet, one per class,
/// rather than a separate sheet: same "grow the sheet" convention
/// `character_idle.png` used for 4-directional Walk. Forbidden-row 3
/// (32 / EXTRA_ANIM_COLS == 3) is unaffected by adding more rows below
/// it - that formula only depends on column count. North-east facing
/// (a 3/4 back view, "walking/climbing away") rather than this sheet's
/// usual south/face-camera content.
fn character_victory_stairs_row(class: &str) -> Option<u16> {
    match class {
        "Rogue" => Some(8),
        "Debug" => Some(9),
        "Hunter" => Some(10),
        "Barbarian" => Some(11),
        "Amazon" => Some(12),
        "Mage" => Some(13),
        _ => None,
    }
}

/// Builds a fresh `OneShotAnimation` playing `class`'s ClimbAway pose -
/// see `character_victory_stairs_row`. Same shape as
/// `victory_animation_for_class`, just a different row block on the same
/// sheet.
pub fn victory_climb_animation_for_class(class: &str) -> Option<OneShotAnimation> {
    let row = character_victory_stairs_row(class)?;
    let frames = (0..EXTRA_ANIM_COLS)
        .map(|col| row * EXTRA_ANIM_COLS + col)
        .collect();
    Some(OneShotAnimation {
        frames,
        frame_index: 0,
        elapsed_ms: 0.0,
        frame_duration_ms: IDLE_FRAME_DURATION_MS,
        repeat: false,
    })
}

/// Which painted backdrop a Victory screen shows, and (via `pose`) which
/// pose the player's own animation needs to strike against it - see
/// `screens/end.rs::victory`. Every variant shares `resources/
/// battle_backgrounds.png`'s existing 6x6 glyph grid (see
/// BATTLE_BACKDROP_CONSOLE in main.rs) rather than a dedicated sheet/
/// console of its own - that atlas is already padded to 36 cells for the
/// glyph-32 gotcha and only used 3 of them (one per MapTheme), so there's
/// plenty of room without registering anything new.
///
/// Keyed by the run's own `EndSceneTheme` (Forest/Dungeon/Sewer) plus
/// Arena, NOT by a generic "Dungeon Crawl" bucket - a 2026-09-13 replan
/// after the original generic-dungeon pool could hand a Forest run a
/// stone-vault Victory scene that made no sense for the theme actually
/// explored. Each Dungeon-Crawl theme gets 2 variants (randomized
/// between them - see `random_for_theme`); Arena gets 1 (nothing to
/// randomize).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VictoryBackground {
    Arena,
    ForestStairs,
    ForestStance,
    DungeonStance,
    DungeonCorridor,
    SewerWalk,
    SewerStance,
    SwampStance,
    SwampWalk,
}

impl VictoryBackground {
    /// Battle Arena's win always shows the same courtyard - only one
    /// Arena victory scene exists, so there's nothing to randomize.
    pub fn arena() -> Self {
        VictoryBackground::Arena
    }

    /// Dungeon Crawl's win picks one of `theme`'s own 2 scenes at random
    /// - deliberately unconditional (doesn't check whether that specific
    /// variant's art/pose is actually ready yet), since `background_row`/
    /// `pose` below both degrade gracefully on their own (old procedural
    /// fill, existing face-camera animation) when they aren't. That means
    /// a variant "goes live" the moment a real Some(row)/real pose is
    /// filled in below - no other code changes needed.
    pub fn random_for_theme(theme: EndSceneTheme) -> Self {
        let mut rng = RandomNumberGenerator::new();
        match theme {
            EndSceneTheme::Forest => {
                const POOL: [VictoryBackground; 2] =
                    [VictoryBackground::ForestStairs, VictoryBackground::ForestStance];
                *rng.random_slice_entry(&POOL).unwrap_or(&POOL[0])
            }
            EndSceneTheme::Dungeon => {
                const POOL: [VictoryBackground; 2] = [
                    VictoryBackground::DungeonStance,
                    VictoryBackground::DungeonCorridor,
                ];
                *rng.random_slice_entry(&POOL).unwrap_or(&POOL[0])
            }
            EndSceneTheme::Sewer => {
                const POOL: [VictoryBackground; 2] =
                    [VictoryBackground::SewerWalk, VictoryBackground::SewerStance];
                *rng.random_slice_entry(&POOL).unwrap_or(&POOL[0])
            }
            EndSceneTheme::Swamp => {
                const POOL: [VictoryBackground; 2] =
                    [VictoryBackground::SwampStance, VictoryBackground::SwampWalk];
                *rng.random_slice_entry(&POOL).unwrap_or(&POOL[0])
            }
        }
    }

    /// Row on `resources/battle_backgrounds.png` - `None` until a clean
    /// (non-watermarked) final image is composited in for that variant,
    /// same Option<u16>-with-fallback shape as `MapTheme::
    /// battle_background_row`; callers fall back to the old procedural
    /// tinted-fill background (see draw_end_screen_background).
    pub fn background_row(self) -> Option<u16> {
        match self {
            VictoryBackground::Arena => Some(3),
            VictoryBackground::ForestStairs => Some(4),
            VictoryBackground::ForestStance => Some(6),
            VictoryBackground::DungeonStance => Some(7),
            VictoryBackground::DungeonCorridor => Some(13),
            VictoryBackground::SewerWalk => Some(8),
            VictoryBackground::SewerStance => Some(12),
            VictoryBackground::SwampStance => Some(15),
            VictoryBackground::SwampWalk => Some(16),
        }
    }

    /// Which pose the player's own animation should strike against this
    /// background - see `VictoryPose`.
    pub fn pose(self) -> VictoryPose {
        match self {
            VictoryBackground::Arena
            | VictoryBackground::ForestStance
            | VictoryBackground::DungeonStance
            | VictoryBackground::SewerStance
            | VictoryBackground::SwampStance => VictoryPose::FaceCamera,
            VictoryBackground::DungeonCorridor
            | VictoryBackground::SewerWalk
            | VictoryBackground::SwampWalk => VictoryPose::WalkAway,
            VictoryBackground::ForestStairs => VictoryPose::ClimbAway,
        }
    }

    /// Roughly where this background's own painted composition wants the
    /// hero standing - a coarse BATTLE_PORTRAIT_COLS x BATTLE_PORTRAIT_
    /// ROWS grid position (see main.rs), used by the FaceCamera/ClimbAway
    /// poses (both draw onto that grid via CHARACTER_VICTORY_CONSOLE -
    /// see screens/end.rs::draw_end_screen_portrait). A first guess from
    /// looking at each scene (2026-09-13), not yet screenshot-verified
    /// live - see CLAUDE.md's own convention for bracket-lib layout
    /// guesses made without being able to render and check; expect a
    /// correction round once real play confirms it. Replaces the old
    /// single fixed col=1 used by every background alike, which left
    /// enough room beside it for the now-removed Amulet of Yala icon -
    /// with that gone, every FaceCamera/ClimbAway background can center
    /// the hero properly instead.
    pub fn portrait_grid_position(self) -> (i32, i32) {
        match self {
            VictoryBackground::Arena => (2, 3),
            VictoryBackground::ForestStance => (2, 3),
            // Was row 4 - confirmed too low via a real screenshot
            // (2026-09-13): sat directly on top of "Press Enter..."
            // (row 60/67 = ~90%, same band as row 4's ~90% center).
            // Row 3 matches every other FaceCamera background's own
            // foreground placement and clears that text.
            VictoryBackground::ForestStairs => (2, 3),
            VictoryBackground::DungeonStance => (2, 3),
            VictoryBackground::SewerStance => (2, 3),
            VictoryBackground::SwampStance => (2, 3),
            // WalkAway poses never read this - see walk_away_position
            // instead. Included only for match exhaustiveness.
            VictoryBackground::DungeonCorridor
            | VictoryBackground::SewerWalk
            | VictoryBackground::SwampWalk => (2, 2),
        }
    }

    /// Same idea as `portrait_grid_position`, for `VictoryPose::WalkAway`
    /// specifically - that pose draws on the fine DISPLAY_WIDTH x
    /// DISPLAY_HEIGHT dungeon-tile grid via CHARACTER_IDLE_CONSOLE
    /// instead of the coarse BATTLE_PORTRAIT one every other pose uses,
    /// a completely different scale/console (see draw_end_screen_
    /// portrait). Both current WalkAway scenes (a torch-lit corridor, a
    /// sewer corridor) share the same "centered, a bit above true
    /// midground" composition, hence the same value - kept as an
    /// explicit per-variant match anyway rather than a single constant,
    /// matching this file's own convention, in case a future WalkAway
    /// scene needs to differ.
    /// `f32` (not the coarse grid's `i32`) - this pose draws via
    /// `set_fancy` (see `WALK_AWAY_SCALE`'s own doc comment) for
    /// fractional positioning/scaling, not the plain whole-cell `.set()`
    /// every other pose uses.
    pub fn walk_away_position(self) -> (f32, f32) {
        let midground = (DISPLAY_WIDTH as f32 / 2.0, DISPLAY_HEIGHT as f32 * 0.6);
        match self {
            VictoryBackground::DungeonCorridor => midground,
            VictoryBackground::SewerWalk => midground,
            VictoryBackground::SwampWalk => midground,
            _ => midground,
        }
    }
}

/// See `VictoryBackground::pose`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VictoryPose {
    /// The existing character_victory.png OneShotAnimation, unchanged -
    /// used whenever the background looks straight at the player.
    FaceCamera,
    /// Reuses character_idle.png's real North-facing walk-loop frames
    /// (see `victory_walk_away_animation`) - the background looks away
    /// from the player, down a receding corridor, so the pose should
    /// too. Already fully playable - no new art needed.
    WalkAway,
    /// The player looking up and climbing away up the Forest Stairs
    /// scene - a real north-east-facing animation (see
    /// `victory_climb_animation_for_class`), added 2026-09-13 as a new
    /// row block on `character_victory.png` rather than a separate sheet.
    ClimbAway,
}

/// Builds a looping (`repeat: true`, so it never `finished()`s) "walking
/// away" pose from `class`'s existing North-facing walk frames on
/// `resources/character_idle.png` - `None` for a class with no idle art
/// at all (shouldn't happen for any real player class, but mirrors every
/// other `_for_class` builder's Option shape rather than assuming). Used
/// for `VictoryPose::WalkAway` instead of a dedicated victory animation,
/// since the existing directional walk-cycle art already fits a
/// "receding down a corridor" pose with no new art needed.
pub fn victory_walk_away_animation(class: &str) -> Option<OneShotAnimation> {
    let frames = character_idle_frames(class, Direction::North)?;
    Some(OneShotAnimation {
        frames,
        frame_index: 0,
        elapsed_ms: 0.0,
        frame_duration_ms: IDLE_FRAME_DURATION_MS,
        repeat: true,
    })
}

/// Which painted backdrop a Game Over screen shows - see
/// `VictoryBackground`'s own doc comment for the shared-atlas reasoning
/// and the 2026-09-13 theme-keyed replan. Unlike Victory, there's no
/// randomization (see `screens/end.rs::game_over`) - always the one
/// scene matching the run's own theme/mode, paired with the existing
/// character_death.png animation regardless of which variant this is
/// (Defeat's pose never changes, only the backdrop behind it does).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefeatBackground {
    Arena,
    Forest,
    Dungeon,
    Sewer,
    Swamp,
}

impl DefeatBackground {
    pub fn arena() -> Self {
        DefeatBackground::Arena
    }

    pub fn for_theme(theme: EndSceneTheme) -> Self {
        match theme {
            EndSceneTheme::Forest => DefeatBackground::Forest,
            EndSceneTheme::Dungeon => DefeatBackground::Dungeon,
            EndSceneTheme::Sewer => DefeatBackground::Sewer,
            EndSceneTheme::Swamp => DefeatBackground::Swamp,
        }
    }

    /// Same Option<u16>/shared-atlas shape as `VictoryBackground::
    /// background_row`.
    pub fn background_row(self) -> Option<u16> {
        match self {
            DefeatBackground::Arena => Some(5),
            DefeatBackground::Dungeon => Some(9),
            DefeatBackground::Sewer => Some(10),
            DefeatBackground::Forest => Some(11),
            DefeatBackground::Swamp => Some(17),
        }
    }

    /// Same idea as `VictoryBackground::portrait_grid_position` - a
    /// coarse BATTLE_PORTRAIT grid position, used by
    /// `draw_end_screen_fallen_portrait`'s death-animation branch. Fixes
    /// a real bug found 2026-09-13: that branch kept using a fixed
    /// col=1 (left over from before Victory's own position rework) that
    /// was never updated when the Amulet-of-Yala icon it used to leave
    /// room for was removed - confirmed via a real screenshot showing
    /// the corpse sitting well off-center on the Arena courtyard's own
    /// centered staircase. Every Defeat scene is a roughly-symmetric
    /// "centered path/focal point" composition (unlike Victory's much
    /// more varied set), so all four currently share the same value -
    /// kept as an explicit per-variant match anyway, matching this
    /// file's own convention, in case a future Defeat scene needs to
    /// differ. First guess, not yet screenshot-verified for the other
    /// three themes.
    pub fn portrait_grid_position(self) -> (i32, i32) {
        let centered = (2, 3);
        match self {
            DefeatBackground::Arena => centered,
            DefeatBackground::Dungeon => centered,
            DefeatBackground::Sewer => centered,
            DefeatBackground::Forest => centered,
            DefeatBackground::Swamp => centered,
        }
    }
}

/// Which row a (class, technique name) pair occupies on
/// `resources/character_technique.png` - keyed by a COMPOUND identity
/// rather than by class alone, since a class can end up with several of
/// these over time ("assume we will add a lot more battle animations for
/// each class" - 2026-09-08). Adding the next one is one more match arm
/// with the next free row, same shape as every other per-thing row
/// function in this file - row 3 is this sheet's own forbidden row
/// (32 / 9 == 3), skipped permanently.
fn technique_animation_row(class: &str, technique: &str) -> Option<u16> {
    match (class, technique) {
        ("Rogue", "Flurry") => Some(0),
        ("Hunter", "Arrow Volley") => Some(1),
        ("Barbarian", "Whirlwind") => Some(2),
        // Row 3 deliberately skipped - see this fn's own doc comment.
        ("Amazon", "Javelin Volley") => Some(4),
        ("Mage", "Blizzard") => Some(5),
        // 2026-09-11's full animation batch added a real animation for
        // every remaining technique at once - `resources/
        // character_technique.png` widened from 8 rows to 20 to fit them
        // (row 3 stays the sheet's one permanently forbidden row,
        // unaffected by the extra height).
        ("Barbarian", "Deathblow") => Some(6),
        ("Barbarian", "Quick Attack") => Some(7),
        ("Barbarian", "Rend") => Some(8),
        ("Rogue", "Dodge") => Some(9),
        ("Rogue", "Garrote") => Some(10),
        ("Amazon", "Battle Cry") => Some(11),
        ("Amazon", "Poison Spear") => Some(12),
        ("Hunter", "Feint") => Some(13),
        ("Hunter", "Poison Shot") => Some(14),
        ("Hunter", "Stun") => Some(15),
        ("Mage", "Fireball") => Some(16),
        ("Mage", "Burn") => Some(17),
        // A follow-up zip added this animation after the initial batch.
        ("Barbarian", "Counter Attack") => Some(18),
        _ => None,
    }
}

/// Builds a fresh `OneShotAnimation` playing `class`'s own animation for
/// `technique` - `None` if that specific (class, technique) pair has no
/// row yet, in which case the caller keeps showing the ordinary
/// Fight_Stance_Idle loop instead (see `Battle::player_technique_
/// animation`'s own doc comment). `repeat` should be true for a multi-
/// hit/AOE technique (see `OneShotAnimation::repeat`'s own doc comment)
/// - the caller decides this from the item's own `TechniqueEffect`
/// (`battle::technique_effect`) before calling, since that's the only
/// place that already knows whether this specific use is single-hit or
/// not.
pub fn technique_animation_for(
    class: &str,
    technique: &str,
    repeat: bool,
) -> Option<OneShotAnimation> {
    let row = technique_animation_row(class, technique)?;
    let frames = (0..EXTRA_ANIM_COLS)
        .map(|col| row * EXTRA_ANIM_COLS + col)
        .collect();
    Some(OneShotAnimation {
        frames,
        frame_index: 0,
        elapsed_ms: 0.0,
        frame_duration_ms: TECHNIQUE_FRAME_DURATION_MS,
        repeat,
    })
}

/// Which row a (class, out-of-combat ability name) pair occupies on
/// `resources/character_effect.png` - the dungeon-view equivalent of
/// `technique_animation_row`, added 2026-09-11 for the 7 out-of-combat
/// class Abilities that had real PixelLab art sitting unused (technique
/// animations only ever covered in-battle Techniques). Same compound-key
/// shape as the technique sheet, for the same reason: `RangedStrike`
/// alone covers both Amazon's Throw Spear and Hunter's Shoot with
/// completely different art, so a lookup keyed by `ProvidesEffect`
/// variant alone would collide - keyed by the real `template.ron` item
/// name instead, same as every other per-name row function in this file.
/// Row 3 is this sheet's own forbidden row (32 / 9 == 3, `EXTRA_ANIM_COLS`
/// shared with the technique/death/victory/attack/defend sheets).
fn effect_animation_row(class: &str, ability: &str) -> Option<u16> {
    match (class, ability) {
        ("Rogue", "Stealth") => Some(0),
        ("Amazon", "Throw Spear") => Some(1),
        ("Amazon", "Trap") => Some(2),
        // Row 3 deliberately skipped - see this fn's own doc comment.
        ("Hunter", "Freeze Trap") => Some(4),
        ("Hunter", "Shoot") => Some(5),
        ("Mage", "Ice Armor") => Some(6),
        ("Mage", "Invisible Cloak") => Some(7),
        _ => None,
    }
}

/// Builds a fresh `OneShotAnimation` for `class` using `ability` (its
/// out-of-combat Effect item's real name) - `None` if that pair has no
/// row yet. Always plays once and holds its last frame (no `repeat` -
/// unlike a multi-hit technique, none of these 7 abilities have a
/// landing-over-time mechanic to keep pace with), same
/// `TECHNIQUE_FRAME_DURATION_MS` pace as every other one-shot animation
/// in this project. See `EffectAnimation` (this component holds the
/// result) and `systems::use_items` (where this gets called, the instant
/// one of these 7 effects actually applies).
pub fn effect_animation_for(class: &str, ability: &str) -> Option<OneShotAnimation> {
    let row = effect_animation_row(class, ability)?;
    let frames = (0..EXTRA_ANIM_COLS)
        .map(|col| row * EXTRA_ANIM_COLS + col)
        .collect();
    Some(OneShotAnimation {
        frames,
        frame_index: 0,
        elapsed_ms: 0.0,
        frame_duration_ms: TECHNIQUE_FRAME_DURATION_MS,
        repeat: false,
    })
}

/// A played-once animation override for a stationary dungeon-view entity
/// mid-out-of-combat-ability-use (Ice Armor, Invisible Cloak, Stealth,
/// Throw Spear, Trap, Freeze Trap, Shoot) - the dungeon-view equivalent
/// of `Battle::player_action_animation`. Added to `activate.used_by` in
/// `systems::use_items` the instant one of those 7 effects actually
/// applies (see `effect_animation_for`), ticked every real frame by
/// `systems::animation::tick_effect_animation`, and removed by that same
/// system via `CommandBuffer` once `OneShotAnimation::finished()` - at
/// which point `entity_render`'s `idle_glyph`/`idle_sheet` fall back to
/// this entity's ordinary `IdleAnimation` loop again, same as
/// `Battle::player_action_animation` falling back to
/// `character_battle_glyph` once cleared. A plain component rather than
/// a `Battle`-style resource field, since (unlike a Battle, which only
/// ever has one player) any number of dungeon-view entities could in
/// principle be mid-effect at once.
#[derive(Clone, Debug, PartialEq)]
pub struct EffectAnimation(pub OneShotAnimation);

