#![warn(clippy::pedantic)]

mod arena;
mod arena_state;
mod battle;
mod camera;
mod components;
mod keymap;
mod map;
mod map_builder;
mod render_helpers;
mod screens;
mod settings;
mod spawner;
mod stats;
mod systems;
mod turn_state;

mod prelude {
    pub use bracket_lib::prelude::*;
    pub use legion::systems::CommandBuffer;
    pub use legion::world::SubWorld;
    pub use legion::*;
    pub const SCREEN_WIDTH: i32 = 80;
    pub const SCREEN_HEIGHT: i32 = 50;
    pub const DISPLAY_WIDTH: i32 = SCREEN_WIDTH / 2;
    pub const DISPLAY_HEIGHT: i32 = SCREEN_HEIGHT / 2;
    /// Battle portrait console's coarse grid: same physical 1280x800
    /// window as the dungeon view, far fewer cells, so one glyph fills
    /// 256x160px instead of a 32px dungeon tile - the project's "blow up
    /// a single glyph" trick, reused by every console below that shares
    /// this grid.
    pub const BATTLE_PORTRAIT_COLS: i32 = 5;
    pub const BATTLE_PORTRAIT_ROWS: i32 = 5;
    /// Dungeon HUD console's grid (health bar, item lists, tooltips):
    /// same 1280x800 window, ~1.5x bigger cells than the battle screen's
    /// 8px text font (FINE_TEXT_CONSOLE).
    pub const HUD_COLS: i32 = 107;
    pub const HUD_ROWS: i32 = 67;

    // Console indices below are in BTermBuilder registration order (see
    // main()'s builder chain) - z-order IS registration order, so a
    // later-registered console paints over an earlier one wherever it
    // actually draws something (CLAUDE.md's own standing gotcha). Every
    // console below tagged "dungeon-view" has to stay registered BELOW
    // HUD_CONSOLE/ABILITY_BAR_CONSOLE or it paints over the HUD; anything
    // tagged "battle-only" has no such constraint, since the dungeon HUD
    // isn't showing during a battle - simple appends are fine for those.
    // Console 0 itself (the base map/entity layer, dungeonfont, WITH
    // background) has no named constant - referenced as the literal `0`
    // at its few call sites, same as every other console only ever
    // targeted from one place.

    /// Console 1, dungeon-view: WITH-background, 1x1 grid stretched to
    /// fill the whole window - `resources/battle_backgrounds.png`'s full
    /// painted battle-arena scenes (one glyph per `MapTheme`, see
    /// `MapTheme::battle_background_row`). WITH background (not `_no_bg`)
    /// since these are fully opaque painted scenes with real near-black
    /// detail that a colorkey cutoff would otherwise eat.
    pub const BATTLE_BACKDROP_CONSOLE: usize = 1;
    /// Console 2, dungeon-view: no-bg, `map_tiles.png`, real per-tile
    /// floor/wall textures for any `MapTheme` with one (`MapTheme::
    /// tile_row`) - one shared console/sheet for every theme (a 4-row
    /// block each, see `docs/Map_Tile_Theme_Guide.md`), not one console
    /// per theme. Must sit below the dungeon-view entity layer too, not
    /// just the HUD - it's the floor those entities stand on.
    pub const MAP_TILE_CONSOLE: usize = 2;
    /// Console 3, battle-only: no-bg, `SCREEN_WIDTH*2 x SCREEN_HEIGHT*2`
    /// grid, `terminal8x8.png` - fine 8px text for the battle screen
    /// (message log, HP/ATB numbers, Actions box) plus Game Over/Victory/
    /// Pause.
    pub const FINE_TEXT_CONSOLE: usize = 3;
    /// Console 4, battle-only: no-bg, BATTLE_PORTRAIT_COLS x ROWS,
    /// dungeonfont - the original "big single glyph" console shared by
    /// battle creature portraits, the End screen's Amulet display, and
    /// Title screen icons.
    pub const BATTLE_PORTRAIT_CONSOLE: usize = 4;
    /// Console 5, dungeon-view: a fancy console (`set_fancy`), same grid/
    /// font as console 0, WITH an opaque background (unlike every other
    /// fancy console here, which uses a transparent one). Used by
    /// map_render (`systems/map_render.rs`) for `TileType::Exit`/
    /// `Counter`/`Water` - the tile types still on the old single-glyph
    /// dungeonfont rendering rather than a real per-theme texture (see
    /// `map_tile_glyph`'s own doc comment). Drawn through unconditionally,
    /// not just while the camera pans - a specific-glyph console
    /// discrepancy made console 0 render this content solid black at
    /// rest for reasons never fully identified (see `CLAUDE.md`'s
    /// standing gotchas and `docs/DEVLOG.md` for the full trace); this
    /// fancy path is the one confirmed to always work.
    pub const MAP_SCROLL_CONSOLE: usize = 5;
    /// Console 6, dungeon-view: a fancy console, same grid/font as
    /// MAP_TILE_CONSOLE - its "camera is panning" counterpart, for tiles
    /// from a theme with a real `tile_row`.
    pub const MAP_TILE_SCROLL_CONSOLE: usize = 6;
    /// Console 7, dungeon-view: no-bg, dungeonfont - entities with no
    /// real per-class/enemy idle-frame art (`IdleSpriteSheet::Dungeon`:
    /// the Shopkeeper, floor items, any not-yet-migrated enemy).
    pub const ENTITY_CONSOLE: usize = 7;
    /// Console 8, dungeon-view: a fancy console, transparent background,
    /// same grid as console 0 - ENTITY_CONSOLE's "camera is panning"
    /// counterpart. Every entity (not just one that's individually
    /// mid-glide) needs a fractional position derived from the camera's
    /// own pan on these frames, or a stationary one would stay snapped to
    /// its old integer cell while the map slides under it - see
    /// `components::camera_render_offset`.
    pub const ENTITY_SCROLL_CONSOLE: usize = 8;
    /// Console 9, dungeon-view: no-bg, `character_idle.png` (128x128px
    /// source cells - 4x dungeonfont's 32px, so the console's own GPU
    /// downscale does the final resize instead of a lossy PNG pre-shrink).
    /// Entities whose `IdleAnimation::sheet` is `CharacterIdle`, camera at
    /// rest and this entity not currently gliding.
    pub const CHARACTER_IDLE_CONSOLE: usize = 9;
    /// Console 10, dungeon-view: fancy, same grid/font as
    /// CHARACTER_IDLE_CONSOLE - its "camera is panning" counterpart.
    pub const CHARACTER_IDLE_SCROLL_CONSOLE: usize = 10;
    /// Console 11, dungeon-view: fancy, same grid/font as
    /// CHARACTER_IDLE_CONSOLE - the "camera at rest, this one entity is
    /// mid-glide" counterpart (the more common of the two glide cases).
    pub const CHARACTER_IDLE_GLIDE_CONSOLE: usize = 11;
    /// Console 12, dungeon-view: no-bg, `enemy_idle.png` - the enemy
    /// equivalent of CHARACTER_IDLE_CONSOLE (`IdleAnimation::sheet ==
    /// EnemyIdle`, see `components::idle_frames_for_enemy`). A dedicated
    /// sheet rather than more character_idle.png rows, since enemies are
    /// a different lookup domain (by name, not class).
    pub const ENEMY_IDLE_CONSOLE: usize = 12;
    /// Console 13, dungeon-view: fancy, same grid/font as
    /// ENEMY_IDLE_CONSOLE - its "camera is panning" counterpart.
    pub const ENEMY_IDLE_SCROLL_CONSOLE: usize = 13;
    /// Console 14, dungeon-view: fancy, same grid/font as
    /// ENEMY_IDLE_CONSOLE - the "camera at rest, this entity mid-glide"
    /// counterpart.
    pub const ENEMY_IDLE_GLIDE_CONSOLE: usize = 14;
    /// Console 15, dungeon-view: no-bg, `character_effect.png` - an
    /// out-of-combat Ability's played-once animation override (Ice
    /// Armor, Invisible Cloak, Stealth, Throw Spear, Trap, Freeze Trap,
    /// Shoot - see `components::EffectAnimation`/`effect_animation_for`).
    /// `entity_render.rs` checks for this before ever looking at
    /// `IdleAnimation` at all.
    pub const CHARACTER_EFFECT_CONSOLE: usize = 15;
    /// Console 16, dungeon-view: fancy, same grid/font as
    /// CHARACTER_EFFECT_CONSOLE - its "camera is panning" counterpart.
    pub const CHARACTER_EFFECT_SCROLL_CONSOLE: usize = 16;
    /// Console 17, dungeon-view: fancy, same grid/font as
    /// CHARACTER_EFFECT_CONSOLE - the "camera at rest, mid-glide"
    /// counterpart (rare in practice - an effect animation plays from a
    /// stationary "use item" action, not mid-move - kept for consistency
    /// with the other two dungeon-view trios above).
    pub const CHARACTER_EFFECT_GLIDE_CONSOLE: usize = 17;
    /// Console 18: the dungeon HUD - health bar, item lists, tooltips.
    /// See HUD_COLS/HUD_ROWS above.
    pub const HUD_CONSOLE: usize = 18;
    /// Console 19: big title/class-select text, and the large text used
    /// by the end/victory screens - DISPLAY_WIDTH x DISPLAY_HEIGHT grid
    /// on the small text font, landing at 32x32px cells (4x
    /// FINE_TEXT_CONSOLE's 8px text).
    pub const BIG_TEXT_CONSOLE: usize = 19;
    /// Console 20: a fancy console (supports rotation) - same
    /// DISPLAY_WIDTH x DISPLAY_HEIGHT grid and dungeonfont as console 0,
    /// so cells stay square for a clean (non-stretched) rotation. Used
    /// only by `draw_end_screen_fallen_portrait` for the GameOver
    /// screen's fallen hero.
    pub const END_SCREEN_FALLEN_CONSOLE: usize = 20;
    /// How much to blow up the fallen hero's glyph on the GameOver
    /// screen - `set_fancy`'s `scale` parameter (native 32x32px reads as
    /// tiny against the full 1280x800 window otherwise).
    pub const END_SCREEN_FALLEN_SCALE: f32 = 6.0;
    /// Same idea as END_SCREEN_FALLEN_SCALE, for `VictoryPose::WalkAway`
    /// (`screens/end.rs::draw_end_screen_portrait`) - drawn through
    /// CHARACTER_IDLE_GLIDE_CONSOLE (already fancy, otherwise idle on
    /// this screen) purely for `set_fancy`'s scale parameter. Smaller
    /// than END_SCREEN_FALLEN_SCALE on purpose - this pose reads as
    /// walking away/receding, not a big dead-center portrait.
    pub const VICTORY_WALK_AWAY_SCALE: f32 = 4.0;
    /// Console 21, dungeon-view: fancy, same DISPLAY_WIDTH x
    /// DISPLAY_HEIGHT grid as console 0. Used by `entity_render.rs` to
    /// draw any entity mid-tile-glide (`components::gliding_position`/
    /// `MovingAnimation`) at a sub-pixel position, for the case where the
    /// camera itself ISN'T panning (see ENTITY_SCROLL_CONSOLE above for
    /// the "camera also panning" case). A gliding entity is deliberately
    /// NOT also drawn on console 0/ENTITY_CONSOLE the same frame, so
    /// there's a clean handoff, not a double-draw. Relies on a genuinely
    /// transparent background (RGBA alpha 0) so a fancy console's
    /// normally-opaque background quad doesn't paint a visible box
    /// sliding over the map.
    pub const GLIDE_CONSOLE: usize = 21;
    /// Console 22, battle-only: a fancy console sharing BATTLE_PORTRAIT_
    /// COLS x ROWS's coarse grid with console 4 (same font, same window).
    /// Used by `draw_battle_arena` to give the "Attacking" portrait a
    /// small shake (`render_helpers::attack_wiggle_offset`) via sub-pixel
    /// positioning that a plain console's `set()` can't do. Same
    /// transparent-background trick as GLIDE_CONSOLE.
    pub const BATTLE_PORTRAIT_WIGGLE_CONSOLE: usize = 22;
    /// Console 23, battle-only: a plain console, same grid as
    /// BIG_TEXT_CONSOLE, used only for floating damage-number popups
    /// (`screens/battle.rs`). Needs its own console, registered after
    /// BATTLE_PORTRAIT_WIGGLE_CONSOLE, so a popup always paints above
    /// EVERY portrait regardless of which portrait console is in play -
    /// an idle (non-wiggling) enemy portrait also uses the wiggle
    /// console now (see `draw_portrait_fancy`), which would otherwise
    /// paint over its own damage number.
    pub const DAMAGE_POPUP_CONSOLE: usize = 23;
    /// Ability Bar's coarse icon grid columns - 1280/32 = 40px cells.
    /// Far more columns than the 9 slots ever used
    /// (`systems/player_input.rs::use_ability`'s 1-9 range); the extras
    /// just stay empty/transparent.
    pub const ABILITY_BAR_COLS: i32 = 32;
    /// Ability Bar rows - 800/20 = 40px cells, matching ABILITY_BAR_COLS
    /// for square icons. Only the bottom row is ever drawn into; the rest
    /// exist purely to keep that row thin instead of spanning the
    /// console's full height.
    pub const ABILITY_BAR_ROWS: i32 = 20;
    /// Ability Bar slots reachable by a hotkey - matches
    /// `systems/player_input.rs::use_ability`'s range (keys 1-9, then 0
    /// for the 10th).
    pub const ABILITY_BAR_MAX_SLOTS: usize = 10;
    /// Console 24, dungeon-view: the Ability Bar's icon strip along the
    /// bottom of the dungeon screen (`systems/hud.rs`) - plain
    /// (non-fancy), dungeonfont, registered last among the dungeon-view
    /// consoles so it paints above the tiles/entities/HUD beneath it. No
    /// background - only its bottom row of ABILITY_BAR_COLS cells is ever
    /// drawn into.
    pub const ABILITY_BAR_CONSOLE: usize = 24;
    /// Console 25, dungeon-view: plain, same DISPLAY_WIDTH x
    /// DISPLAY_HEIGHT/32px dungeonfont grid as console 0 - the
    /// player-status frame's buff badges (`systems/hud.rs`). Registered
    /// after ABILITY_BAR_CONSOLE so a badge paints over the portrait/bars
    /// beneath it. No background - only a few cells are ever used.
    pub const BUFF_BADGE_CONSOLE: usize = 25;
    /// Console 26, dungeon-view: same HUD_COLS x HUD_ROWS/terminal8x8
    /// grid as HUD_CONSOLE (reuses its pixel-ratio math), registered
    /// LAST of the dungeon-view consoles - the Ability/Item/Battle Bar's
    /// stack-count badges (`systems/hud.rs`, e.g. "x2" for two Freeze
    /// Traps). Has to be last: those bar icons are opaque sprite art on
    /// ABILITY_BAR_CONSOLE, so a badge drawn onto HUD_CONSOLE directly
    /// would sit below the icon in z-order and never show. No
    /// background.
    pub const ABILITY_BAR_BADGE_CONSOLE: usize = 26;
    /// Console 27, battle-only: plain, sharing console 4's coarse grid,
    /// `character_idle.png` - the Class Select screen's highlighted-class
    /// preview (`screens/title.rs::class_select`), playing that class's
    /// real idle loop in place of its static portrait.
    pub const CLASS_SELECT_IDLE_CONSOLE: usize = 27;
    /// Console 28, battle-only: plain, console 4's grid, `character_
    /// battle.png` at native 32x32px cells - the battle screen's own
    /// player-portrait idle loop (`screens/battle.rs::draw_battle_arena`,
    /// driven by `Battle::player_idle_frame`, not the dungeon-view
    /// `IdleAnimation` system). Enemies keep using console 4/dungeonfont.
    pub const CHARACTER_BATTLE_CONSOLE: usize = 28;
    /// Console 29, battle-only: fancy, same grid/font as
    /// CHARACTER_BATTLE_CONSOLE - its "Attacking" shake equivalent of
    /// BATTLE_PORTRAIT_WIGGLE_CONSOLE.
    pub const CHARACTER_BATTLE_WIGGLE_CONSOLE: usize = 29;
    /// Console 30, battle-only: plain, console 4's grid, `character_
    /// portrait.png` (one still pose per class, PixelLab's own
    /// `rotations/south.png`). Shared by Class Select's non-highlighted
    /// icon, the in-battle Victory screen's player portrait, and the
    /// run-ending Victory screen's hero icon.
    pub const CHARACTER_PORTRAIT_BIG_CONSOLE: usize = 30;
    /// Console 31, dungeon-view: plain, ABILITY_BAR_COLS x ROWS grid (so
    /// it lines up with the health frame's own icon position),
    /// `character_portrait.png` - the dungeon HUD's player-status
    /// portrait icon (`systems/hud.rs`).
    pub const CHARACTER_PORTRAIT_HUD_CONSOLE: usize = 31;
    /// Console 32, battle-only: fancy, same grid as
    /// END_SCREEN_FALLEN_CONSOLE, `character_portrait.png` at native
    /// 32x32px cells - the GameOver screen's fallen hero for any class
    /// with a row on this sheet (falls back to END_SCREEN_FALLEN_CONSOLE/
    /// dungeonfont otherwise).
    pub const END_SCREEN_FALLEN_PORTRAIT_CONSOLE: usize = 32;
    /// Console 33, battle-only: plain, console 4's grid, `enemy_
    /// battle.png` at native 32x32px cells - the per-enemy battle-idle
    /// loop (`draw_battle_arena`, driven by each `EnemyCombatant`'s own
    /// `battle_idle_frame`) for any enemy with a row on that sheet.
    pub const ENEMY_BATTLE_CONSOLE: usize = 33;
    /// Console 34, battle-only: fancy, same grid/font as
    /// ENEMY_BATTLE_CONSOLE - both an actively wiggling ("Attacking")
    /// enemy portrait and any non-wiggling multi-enemy one (both need
    /// the fractional positioning a plain console can't express).
    pub const ENEMY_BATTLE_WIGGLE_CONSOLE: usize = 34;
    /// Console 35, battle-only: plain, same grid as
    /// CHARACTER_PORTRAIT_BIG_CONSOLE, `character_death.png` - a real
    /// played-once death animation (`components::death_animation_for_
    /// class`/`OneShotAnimation`) for any class with a row there,
    /// replacing the old "rotate the static portrait 90 degrees" hack.
    pub const CHARACTER_DEATH_CONSOLE: usize = 35;
    /// Console 36, battle-only: FANCY, same grid/font as CHARACTER_
    /// DEATH_CONSOLE - `set_fancy`'s fractional positioning for the
    /// Defeat screen's own fallen-portrait placement
    /// (`components::DefeatBackground::fallen_portrait_position`), added
    /// 2026-09-13 once Swamp's own Defeat art proved the coarse 5x5
    /// `BATTLE_PORTRAIT` grid genuinely couldn't place the corpse on
    /// solid ground AND clear of the frame edge at the same time - every
    /// integer cell tried was one or the other, never both. Every other
    /// theme's own position still just widens its existing (col, row)
    /// into (col as f32, row as f32) - unaffected, exact same spot.
    pub const CHARACTER_DEATH_GLIDE_CONSOLE: usize = 36;
    /// Console 37, battle-only: plain, same grid, `character_
    /// victory.png` - a real played-once victory-pose animation
    /// (`components::victory_animation_for_class`), shared by the
    /// in-battle Victory screen and the run-ending Victory screen.
    pub const CHARACTER_VICTORY_CONSOLE: usize = 37;
    /// Console 38, battle-only: plain, same grid, `character_
    /// technique.png` - keyed by (class, technique) pairs rather than
    /// class alone (`components::technique_animation_for`/`Battle::
    /// player_technique_animation`). Replaces the ordinary battle-idle
    /// loop for exactly the duration of a technique's own display.
    pub const CHARACTER_TECHNIQUE_CONSOLE: usize = 38;
    /// Console 39, battle-only: plain, same grid, `character_
    /// attack.png` - the generic basic-Attack one-shot animation
    /// (`components::attack_animation_for_class`).
    pub const CHARACTER_ATTACK_CONSOLE: usize = 39;
    /// Console 40, battle-only: plain, same grid, `character_
    /// defend.png` - Defend's equivalent of CHARACTER_ATTACK_CONSOLE.
    pub const CHARACTER_DEFEND_CONSOLE: usize = 40;
    /// Console 41, battle-only: FANCY (unlike CHARACTER_ATTACK/DEFEND_
    /// CONSOLE), `enemy_attack.png` (`components::attack_animation_for_
    /// enemy`/`EnemyCombatant::attack_animation`) - needs `set_fancy`'s
    /// fractional positioning so a 2+ enemy fight's zigzag formation
    /// still lines up during an enemy's own Attack animation.
    pub const ENEMY_ATTACK_CONSOLE: usize = 41;
    /// Console 42, battle-only: FANCY (same reasoning as ENEMY_ATTACK_
    /// CONSOLE), `enemy_death.png` (`components::death_animation_for_
    /// enemy`/`EnemyCombatant::death_animation`).
    pub const ENEMY_DEATH_CONSOLE: usize = 42;
    /// Consoles 43/44/45, dungeon-view: the Shopkeeper's own plain/
    /// scroll/glide trio, same shape as CHARACTER_IDLE_CONSOLE/
    /// ENEMY_IDLE_CONSOLE/CHARACTER_EFFECT_CONSOLE's trios above (see
    /// `IdleSpriteSheet::Shopkeeper`'s own doc comment for why it needs a
    /// dedicated sheet). Appended at the end rather than inserted early -
    /// a single always-decorative NPC has nothing that requires the
    /// lower slot the earlier trios needed.
    pub const SHOPKEEPER_IDLE_CONSOLE: usize = 43;
    pub const SHOPKEEPER_IDLE_SCROLL_CONSOLE: usize = 44;
    pub const SHOPKEEPER_IDLE_GLIDE_CONSOLE: usize = 45;
    /// Console 46, dungeon/menu-view: fancy, same DISPLAY_WIDTH x
    /// DISPLAY_HEIGHT/32px grid as console 0, `ui_panels.png` - the real
    /// PixelLab 9-slice panel borders (`render_helpers::draw_pixel_box`,
    /// item 10 in docs/ideas.md), replacing `draw_ascii_box` one call site
    /// at a time. FANCY (not plain) so a box's real pixel position can
    /// land at a sub-cell offset via `set_fancy` - `draw_pixel_box` only
    /// ever draws whole 32px tiles (no per-tile stretching), but the
    /// WHOLE box's own top-left anchor still needs fractional placement,
    /// since HUD_CONSOLE's own ~12px cells essentially never land on a
    /// multiple of 32px. Appended at the very end rather than inserted
    /// earlier: `draw_pixel_box` only ever draws a HOLLOW border (same as
    /// draw_ascii_box - no filled interior yet, see its own doc comment),
    /// so it can never actually overlap whatever a box's own interior
    /// text/icons draw on a console registered earlier, regardless of
    /// z-order between the two.
    pub const UI_PANEL_CONSOLE: usize = 46;
    /// Console 47, dungeon/menu-view: plain (no_bg), same HUD_COLS x
    /// HUD_ROWS grid as HUD_CONSOLE, terminal8x8.png - text printed OVER
    /// a `draw_filled_pixel_box` fill needs to live on a console
    /// registered LATER than the fill's own (HUD_CONSOLE), not just drawn
    /// after it on the SAME console. Confirmed live 2026-09-14, traced to
    /// bracket-terminal's real source, not guessed: `SimpleConsole::set`
    /// REPLACES a cell's entire (glyph, fg, bg) tuple in its own `tiles`
    /// array - printing text over a cell the fill already painted doesn't
    /// layer on top of it, it OVERWRITES it outright, same array slot.
    /// Once overwritten, the CELL's glyph is now the TEXT's own letter
    /// shape - and a no_bg console's shader discards any pixel whose
    /// SOURCE TEXTURE is near-black, which is exactly what the "empty"
    /// space around a letter's own stroke looks like inside its 8x8
    /// glyph cell. Discarding on a no_bg console reveals whatever's on
    /// the console(s) below - previously the dungeon view, three-plus
    /// layers down, since the fill that USED to be at that exact cell no
    /// longer exists once text replaced it. Registering text on ITS OWN
    /// LATER console instead means the fill's own tiles (on HUD_CONSOLE)
    /// are never touched by a text draw at all - text's own discarded
    /// pixels now reveal the fill sitting untouched one console down,
    /// not the dungeon view several consoles further down still.
    pub const PANEL_TEXT_CONSOLE: usize = 47;
    /// Console 48, dungeon-view: plain (no_bg), same `ABILITY_BAR_COLS x
    /// ABILITY_BAR_ROWS`/dungeonfont grid as `ABILITY_BAR_CONSOLE` - a
    /// SECOND, LATER-registered instance of that exact same console
    /// config, used ONLY by the out-of-combat Item/Ability/Battle Bar's
    /// own icon portraits (`systems/hud.rs`). Needed 2026-09-14: once
    /// `draw_filled_pixel_box`'s solid fill moved onto `UI_PANEL_CONSOLE`
    /// (46) for pixel-perfect alignment with its own border (see that
    /// console's own doc comment), the fill became a single fully-opaque
    /// `set_fancy` quad with no discard at all - and since `UI_PANEL_
    /// CONSOLE` (46) is registered AFTER `ABILITY_BAR_CONSOLE` (24), that
    /// opaque fill started painting directly over every icon the bars
    /// draw there, hiding them completely (confirmed live). Rather than
    /// renumber the whole console list to move `ABILITY_BAR_CONSOLE`
    /// itself later (a much bigger, riskier mechanical change - every
    /// constant between its old and new position would need to shift),
    /// this appends a plain duplicate of its exact config at the very
    /// end instead, registered AFTER `UI_PANEL_CONSOLE`/`PANEL_TEXT_
    /// CONSOLE` - `systems/hud.rs`'s out-of-combat bar block draws its
    /// icon portraits here now instead of `ABILITY_BAR_CONSOLE`, which
    /// stays registered (and in the per-frame `cls()` sweep) purely for
    /// the mouse-position translation math (`ctx.set_active_console`)
    /// that's keyed to its grid dimensions - identical between the two
    /// consoles, so that math needs no change.
    pub const ABILITY_BAR_ICON_CONSOLE: usize = 48;
    /// Console 49, dungeon-view: plain (no_bg), same `HUD_COLS x
    /// HUD_ROWS`/terminal8x8 grid as `ABILITY_BAR_BADGE_CONSOLE` - the
    /// same "second later instance" fix as `ABILITY_BAR_ICON_CONSOLE`
    /// just above, for the out-of-combat bars' own stack-count badges
    /// (e.g. "x2" for two Freeze Traps), which sit ON TOP of those same
    /// icons and therefore need to move later in lockstep with them.
    pub const ABILITY_BAR_ICON_BADGE_CONSOLE: usize = 49;
    /// Console 50, battle-only: fancy, same DISPLAY_WIDTH x
    /// DISPLAY_HEIGHT/32px grid as UI_PANEL_CONSOLE, sourced from
    /// `battle_bar_frame.png` (item 10 in docs/ideas.md) - the player's
    /// real pixel-art HP/ATB bars (`render_helpers::draw_pixel_bar`),
    /// replacing the old `[####----]` ASCII-text bars
    /// (`battle::hp_bar_string`) for the player specifically (enemies
    /// keep the ASCII version - see docs/journal.md's 2026-09-14 entry).
    /// FANCY for the same reason UI_PANEL_CONSOLE is: `set_fancy`'s
    /// fractional positioning and non-uniform per-axis scale are what
    /// let one glyph become a precisely-sized colored fill quad, the
    /// same trick `render_helpers::draw_panel_fill` already uses.
    pub const BATTLE_BAR_CONSOLE: usize = 50;
    /// Console 51, battle-only: plain (no_bg), same HUD_COLS x HUD_ROWS
    /// grid as HUD_CONSOLE/PANEL_TEXT_CONSOLE, terminal8x8.png - the
    /// player's HP number, overlaid directly ON TOP of the health bar
    /// itself (direct request 2026-09-14). Registered LAST of all -
    /// even PANEL_TEXT_CONSOLE (47) is registered before BATTLE_BAR_
    /// CONSOLE (50), so text meant to sit ON TOP of a bar drawn there
    /// needs a console later than BOTH, not just later than the fill/
    /// border the way every other converted box's own text only needed.
    pub const BATTLE_BAR_TEXT_CONSOLE: usize = 51;

    /// Every registered console, in the same order `main()`'s builder
    /// chain registers them - `State::tick`'s own per-frame `cls()` sweep
    /// iterates this instead of one hand-written `set_active_console`/
    /// `cls()` pair per console. A newly registered console still needs
    /// to be added here (see `CLAUDE.md`'s standing gotcha on this), but
    /// now there's exactly one place to add it instead of one place to
    /// register it AND a second, easy-to-forget place to remember to
    /// clear it.
    pub const ALL_CONSOLES: &[usize] = &[
        0,
        BATTLE_BACKDROP_CONSOLE,
        MAP_TILE_CONSOLE,
        FINE_TEXT_CONSOLE,
        BATTLE_PORTRAIT_CONSOLE,
        MAP_SCROLL_CONSOLE,
        MAP_TILE_SCROLL_CONSOLE,
        ENTITY_CONSOLE,
        ENTITY_SCROLL_CONSOLE,
        CHARACTER_IDLE_CONSOLE,
        CHARACTER_IDLE_SCROLL_CONSOLE,
        CHARACTER_IDLE_GLIDE_CONSOLE,
        ENEMY_IDLE_CONSOLE,
        ENEMY_IDLE_SCROLL_CONSOLE,
        ENEMY_IDLE_GLIDE_CONSOLE,
        CHARACTER_EFFECT_CONSOLE,
        CHARACTER_EFFECT_SCROLL_CONSOLE,
        CHARACTER_EFFECT_GLIDE_CONSOLE,
        HUD_CONSOLE,
        BIG_TEXT_CONSOLE,
        END_SCREEN_FALLEN_CONSOLE,
        GLIDE_CONSOLE,
        BATTLE_PORTRAIT_WIGGLE_CONSOLE,
        DAMAGE_POPUP_CONSOLE,
        ABILITY_BAR_CONSOLE,
        BUFF_BADGE_CONSOLE,
        ABILITY_BAR_BADGE_CONSOLE,
        CLASS_SELECT_IDLE_CONSOLE,
        CHARACTER_BATTLE_CONSOLE,
        CHARACTER_BATTLE_WIGGLE_CONSOLE,
        CHARACTER_PORTRAIT_BIG_CONSOLE,
        CHARACTER_PORTRAIT_HUD_CONSOLE,
        END_SCREEN_FALLEN_PORTRAIT_CONSOLE,
        ENEMY_BATTLE_CONSOLE,
        ENEMY_BATTLE_WIGGLE_CONSOLE,
        CHARACTER_DEATH_CONSOLE,
        CHARACTER_DEATH_GLIDE_CONSOLE,
        CHARACTER_VICTORY_CONSOLE,
        CHARACTER_TECHNIQUE_CONSOLE,
        CHARACTER_ATTACK_CONSOLE,
        CHARACTER_DEFEND_CONSOLE,
        ENEMY_ATTACK_CONSOLE,
        ENEMY_DEATH_CONSOLE,
        SHOPKEEPER_IDLE_CONSOLE,
        SHOPKEEPER_IDLE_SCROLL_CONSOLE,
        SHOPKEEPER_IDLE_GLIDE_CONSOLE,
        UI_PANEL_CONSOLE,
        PANEL_TEXT_CONSOLE,
        ABILITY_BAR_ICON_CONSOLE,
        ABILITY_BAR_ICON_BADGE_CONSOLE,
        BATTLE_BAR_CONSOLE,
        BATTLE_BAR_TEXT_CONSOLE,
    ];

    pub use crate::arena::*;
    pub use crate::battle::*;
    pub use crate::camera::*;
    pub use crate::components::*;
    pub use crate::keymap::*;
    pub use crate::map::*;
    pub use crate::map_builder::*;
    pub use crate::render_helpers::*;
    pub use crate::settings::*;
    pub use crate::spawner::*;
    pub use crate::stats::*;
    pub use crate::systems::*;
    pub use crate::turn_state::*;
}

use prelude::*;

struct State {
    ecs: World,
    resources: Resources,
    input_systems: Schedule,
    player_systems: Schedule,
    monster_systems: Schedule,
    pause_systems: Schedule,
    background_systems: Schedule,
    /// See build_title_background_movement_scheduler - the decorative
    /// background enemies' movement, run separately from
    /// background_systems (which redraws every frame) so it can be
    /// throttled by background_move_timer_ms instead of moving a full
    /// tile 30 times a second.
    background_movement_systems: Schedule,
    /// Accumulates real elapsed time (ms) while the title/class-select
    /// background is on screen; background_movement_systems only
    /// actually runs once this passes BACKGROUND_MOVE_INTERVAL_MS, then
    /// it resets to 0 - see title_screen/class_select.
    background_move_timer_ms: f32,
    /// Which Action the Options screen is currently waiting for a new
    /// key for, if any - None means the screen is just showing the list
    /// (see screens/options.rs). Kept as a plain State field rather than
    /// an ECS resource since it's transient screen-navigation state, not
    /// anything gameplay systems need to see, and State's other
    /// Resources::default() reset points (start_game/return_to_title)
    /// have no reason to touch it either way.
    options_awaiting: Option<Action>,
    /// Which screen Options should return to on Escape - TitleScreen or
    /// Paused, whichever one it was opened from (see title.rs's and
    /// pause.rs's 'O' handlers, which both set this right before
    /// entering TurnState::Options). Same "plain State field, not a
    /// resource" reasoning as options_awaiting above.
    options_return_to: TurnState,
    /// Which sub-view the History screen is currently showing - see
    /// StatsViewMode's own doc comment. Same "plain State field, not a
    /// resource" reasoning as options_awaiting.
    stats_view_mode: StatsViewMode,
    /// Which adventure type was picked at the new AdventureSelect screen
    /// - read by class_select to decide whether to call start_game
    /// (Dungeon Crawl) or start_arena (Battle Arena). Same "plain State
    /// field, not a resource" reasoning as options_awaiting - this is
    /// screen-navigation state, not anything a gameplay system reads.
    /// Defaults to DungeonCrawl so the Debug hidden shortcut in
    /// class_select (which can be reached without ever visiting
    /// AdventureSelect, if that ever changes) keeps its old behavior.
    adventure_mode: AdventureMode,
    /// Cursor row for Adventure Select's arrow-key navigation (0 or 1) -
    /// see screens/title.rs::adventure_select. Reset to 0 in
    /// return_to_title and whenever title_screen re-enters this screen.
    adventure_select_cursor: usize,
    /// Cursor row for Class Select's arrow-key navigation (0..CLASS_ROSTER.len())
    /// - see screens/title.rs::class_select. Reset the same way as
    /// adventure_select_cursor above.
    class_select_cursor: usize,
    /// Cursor row for Theme Select's arrow-key navigation (0..ThemeChoice
    /// ::ALL.len()) - see screens/title.rs::theme_select and
    /// TurnState::ThemeSelect. Reset the same way as
    /// adventure_select_cursor/class_select_cursor above.
    theme_select_cursor: usize,
    /// The theme Theme Select confirmed for the upcoming Debug run (see
    /// TurnState::ThemeSelect) - `ThemeChoice::Random` for every other
    /// class, since only that screen ever sets this to anything else.
    /// Consumed and reset back to `Random` by start_game the instant it's
    /// read, so a later ordinary (non-Debug) run can never inherit a
    /// stale forced theme.
    pending_theme_choice: ThemeChoice,
    /// Which idle-loop frame the currently-highlighted class's preview is
    /// showing, and how long it's been showing it - see
    /// screens/title.rs::class_select. Plain State fields rather than a
    /// real ECS IdleAnimation component, since a class-select roster
    /// entry isn't a real entity - same "menu timing lives on State"
    /// convention as background_move_timer_ms. Reset to (0, 0.0) whenever
    /// class_select_cursor changes, so switching the highlight always
    /// restarts the breathing cycle cleanly instead of continuing
    /// mid-cycle from whichever frame the previous class happened to be
    /// on.
    class_select_anim_frame: usize,
    class_select_anim_elapsed_ms: f32,
    /// Cursor row for the Options screen's browsing list (the 4 rebindable
    /// Actions plus Battle Speed/ATB Mode, 6 rows total) - see
    /// screens/options.rs. Reset to 0 every time Options is freshly
    /// entered (from the title screen or from Pause).
    options_cursor: usize,
    /// Cursor row for History's Overview class list - see
    /// screens/stats_view.rs. Reset to 0 whenever the History screen is
    /// freshly entered from the title screen (not when returning to
    /// Overview from a drill-down sub-view, so the cursor stays where it
    /// was on the class you just looked at).
    stats_view_cursor: usize,
    /// Cursor position for the Item Menu (press M) - see
    /// screens/item_menu.rs. Reuses battle::MenuCursor's exact 2-column
    /// + remembered-row-per-column shape (col 0 = the left side's Items
    /// + Equipped Items, concatenated into one list; col 1 = the right
    /// side's Battle Actions + Dungeon Actions, same treatment) rather
    /// than duplicating that logic for a second, unrelated menu - see
    /// MenuCursor's own doc comment for why nothing about it is actually
    /// battle-specific. Reset to (0, 0) in return_to_title; persists
    /// across individual menu opens within the same run otherwise (a
    /// small, harmless convenience - if a list has since shrunk,
    /// MenuCursor's own clamping keeps this in bounds regardless).
    item_menu_cursor: MenuCursor,
    /// Cursor row for the Paused screen's menu (Resume/Options/Quit) -
    /// see screens/pause.rs. Same "reset only in return_to_title, not on
    /// every fresh open" reasoning as item_menu_cursor - Escape reaches
    /// this screen from inside player_input's ECS system, which has no
    /// access to plain State fields to reset one on the way in.
    pause_cursor: usize,
    /// Real elapsed ms Enter has been continuously NOT the held key,
    /// while pending_enter_release is armed - see that field's own doc
    /// comment and ENTER_RELEASE_DEBOUNCE_MS. Reset to 0 the instant
    /// Enter is seen held again; pending_enter_release only actually
    /// clears once this crosses the debounce threshold.
    enter_not_held_ms: f32,
    /// True from the moment Enter drives a transition into a screen that
    /// Enter can ALSO dismiss/confirm (Battle Victory or Game Over,
    /// arrived at via a held Enter that fired the killing blow / took
    /// the fatal hit - see screens/battle.rs's finish_battle/
    /// dismiss_action_result; or Adventure Select -> Class Select, both
    /// Enter-confirmable - see screens/title.rs's adventure_select),
    /// until that same Enter press is actually released. While true,
    /// the destination screen's own Enter check is suppressed, so the
    /// exact key that caused the transition can't ALSO immediately fire
    /// the next screen's action. Cleared centrally in tick() only after
    /// Enter has been continuously absent for ENTER_RELEASE_DEBOUNCE_MS
    /// (see enter_not_held_ms) - NOT the instant a single frame shows it
    /// absent. That distinction matters: this project's own WSLg/X11
    /// keyboard stack can deliver a genuinely-held key's OS auto-repeat
    /// as alternating fake release+press events rather than one
    /// sustained press (classic X11 behavior without "detectable
    /// autorepeat" support) - a plain single-frame check was fooled by
    /// that flicker, clearing the guard on the fake release and letting
    /// the very next fake repeat re-arm dismissal immediately, defeating
    /// the whole point. A short real-world debounce survives that quirk
    /// while still feeling instant for an actual release-and-press.
    /// Deliberately narrow otherwise: only these specific hand-off
    /// points check it, and it never gates cursor movement, number-key
    /// selection, or the in-battle ActionResult screens' existing "any
    /// held key blows through fast" behavior, which stays exactly as it
    /// was.
    pending_enter_release: bool,
    /// Which entry of screens::pause::PAUSE_HINTS is currently showing on
    /// the Paused screen - see paused_tick. Advances on its own timer
    /// (pause_hint_timer_ms) rather than resetting whenever Paused is
    /// freshly entered, the same "just keeps ticking" reasoning
    /// background_move_timer_ms already uses for the title screen's
    /// decorative background - a loading-screen-style tip rotation isn't
    /// expected to restart from the top every time you check it.
    pause_hint_index: usize,
    /// Real elapsed ms the current pause_hint_index has been showing -
    /// see PAUSE_HINT_INTERVAL_MS. Only accumulates while paused_tick is
    /// actually running (i.e. only while TurnState::Paused), same as
    /// background_move_timer_ms only accumulating during the title
    /// screen's own background schedule.
    pause_hint_timer_ms: f32,
    /// Whether the left mouse button was down as of the PREVIOUS frame -
    /// compared against the current frame's real held/not-held state
    /// (`INPUT.lock().is_mouse_button_pressed(0)`) each tick() to derive
    /// MouseLeftJustPressed, the one-frame-per-physical-click signal the
    /// Item Bar's click-to-use handler reads. See MouseLeftJustPressed's
    /// own doc comment for why this compares real held-state across
    /// frames rather than using BTerm::left_click directly. Never reset
    /// on return_to_title - unlike pending_enter_release, this has no
    /// per-screen hand-off semantics, it just continuously tracks physical
    /// button state regardless of what screen is showing.
    mouse_left_was_down: bool,
    /// The run-ending Game Over screen's own death animation, if the
    /// player's class has one (see components::death_animation_for_class)
    /// - `None` either before Game Over triggers or for a class without
    /// real death art yet, in which case game_over() keeps the old
    /// rotated-glyph fallback. Built once, the instant Game Over is
    /// entered (see dismiss_action_result/screens/battle.rs), then ticked
    /// once per frame by game_over() itself - a OneShotAnimation holds on
    /// its last frame once finished, so this never needs to be cleared
    /// early, only reset (to a fresh one, or None) the next time a run
    /// actually ends in death.
    death_animation: Option<OneShotAnimation>,
    /// The run-ending Victory screen's own victory-pose animation, same
    /// shape/lifecycle as death_animation above (see
    /// components::victory_animation_for_class) - built once when Victory
    /// is entered, ticked by victory() itself. Deliberately separate from
    /// the IN-BATTLE Victory screen's own animation (BattleVictory::
    /// portrait_animation lives on that resource instead, since a run
    /// can include many in-battle victories but only ever reaches this
    /// specific run-ending screen once).
    victory_animation: Option<OneShotAnimation>,
    /// Which painted backdrop (and, via its `pose()`, which of the
    /// player's own animations) this run's Victory screen is showing -
    /// see components::VictoryBackground. Built once the instant Victory
    /// is entered (Arena's is fixed; Dungeon Crawl's is randomized - see
    /// VictoryBackground::random_dungeon), then left alone for the rest
    /// of that screen so it doesn't re-randomize every frame. Reset to
    /// `None` only by return_to_title, same lifecycle as death_animation/
    /// victory_animation above.
    victory_background: Option<VictoryBackground>,
    /// The WalkAway pose's own looping animation (see
    /// components::victory_walk_away_animation) - kept separate from
    /// victory_animation above since the two poses read from different
    /// sheets/consoles (character_victory.png vs. character_idle.png) and
    /// can't share one Option field. Only ever built when
    /// victory_background's pose() resolves to VictoryPose::WalkAway;
    /// same "built once, ticked every frame, reset by return_to_title"
    /// lifecycle otherwise.
    victory_walk_animation: Option<OneShotAnimation>,
    /// The ClimbAway pose's own played-once animation (see
    /// components::victory_climb_animation_for_class) - kept separate
    /// from victory_walk_animation above for the identical reason that
    /// one is separate from victory_animation: a different row block on
    /// character_victory.png, built/reset the same lifecycle way. Only
    /// ever built when victory_background's pose() resolves to
    /// VictoryPose::ClimbAway.
    victory_climb_animation: Option<OneShotAnimation>,
}

/// How long Enter must be continuously absent before pending_enter_release
/// actually clears - see that field's own doc comment for why this isn't
/// just "the first frame it's not held". Comfortably longer than any
/// single- or few-frame event flicker, comfortably shorter than any real
/// human release-then-press-again.
const ENTER_RELEASE_DEBOUNCE_MS: f32 = 150.0;

/// The actual debounce math behind pending_enter_release/
/// enter_not_held_ms - pulled out as a pure function (no BTerm/State
/// needed) so it's directly testable. Returns the updated (still_pending,
/// elapsed_not_held_ms). If `pending` is already false, both reset to
/// (false, 0.0) - nothing to debounce. Otherwise: seeing Enter held
/// resets the "not held" clock to 0 (still pending); not seeing it
/// accumulates real elapsed time, clearing only once that crosses
/// ENTER_RELEASE_DEBOUNCE_MS.
fn tick_enter_release_guard(
    pending: bool,
    enter_held: bool,
    not_held_ms: f32,
    frame_time_ms: f32,
) -> (bool, f32) {
    if !pending {
        return (false, 0.0);
    }
    if enter_held {
        (true, 0.0)
    } else {
        let elapsed = not_held_ms + frame_time_ms;
        (elapsed < ENTER_RELEASE_DEBOUNCE_MS, elapsed)
    }
}

impl State {
    fn new() -> Self {
        let mut resources = Resources::default();
        resources.insert(TurnState::TitleScreen);
        // random_move_system (part of background_movement_systems, see
        // build_title_background_movement_scheduler) reads this resource -
        // it must exist before the very first
        // background_movement_systems.execute() call, which happens from
        // title_screen()'s tick right after this constructor returns.
        // Previously nothing in the background schedules touched
        // Option<Battle> at all, so its absence here was harmless; it no
        // longer is.
        resources.insert(None::<Battle>);
        // Same reasoning as Option<Battle> just above -
        // movement_system now reads Option<ShoppingActive> too (see
        // systems/movement.rs's auto-pickup gate), so it must exist
        // before the first background_movement_systems.execute() call.
        resources.insert(None::<ShoppingActive>);
        // Same reasoning again - movement_system also reads
        // Option<ArenaRun> now (see the LOS-freeze fix in
        // systems/movement.rs). Missing this is exactly what caused a
        // startup panic: legion's resource fetch has no "missing means
        // None" fallback the way an Option value's own None does - the
        // RESOURCE SLOT ITSELF has to exist, or the fetch panics
        // outright, even though the type being fetched is an Option.
        resources.insert(None::<ArenaRun>);
        // Same reasoning again - movement_system also reads
        // Option<ChestLoot> now (see the chest-opening branch in
        // systems/movement.rs), so it too must exist before the first
        // background_movement_systems.execute() call. This is the exact
        // "forgot one reset point" panic the comment above already warns
        // about, hit for real once already for ShoppingActive/ArenaRun -
        // see CLAUDE.md's own standing gotcha about title-background
        // schedulers needing every #[resource] they touch inserted in
        // BOTH State::new() and State::return_to_title().
        resources.insert(None::<ChestLoot>);
        // player_input_system and hud_system (both part of
        // build_input_scheduler, run during TurnState::AwaitingInput) now
        // read Option<ShopMessage> - unlike Option<Battle>/ArenaRun/
        // ShoppingActive above, that scheduler never runs during the
        // title-background schedules, so this isn't strictly needed here
        // for THAT reason, but every full Resources::default() reset point
        // gets it anyway for the same defensive consistency (start_game/
        // start_arena/return_to_title all do too) - cheap insurance
        // against the exact "forgot one reset point" class of startup
        // panic this project has already hit once for a different
        // resource.
        resources.insert(None::<ShopMessage>);
        // Loaded once here and re-inserted after every Resources::default()
        // reset point below (start_game/return_to_title also wipe every
        // resource) - Keymap::load reads from disk each time, so a rebind
        // made in one run is still there after starting a fresh one or
        // returning to the title screen, without needing a separate
        // long-lived copy on State itself.
        resources.insert(Keymap::load());
        resources.insert(BattleSpeed::load());
        resources.insert(AtbMode::load());
        resources.insert(MenuMemory::load());
        resources.insert(LastBattleAction::load());
        resources.insert(Stats::load());
        let mut state = Self {
            ecs: World::default(),
            resources,
            input_systems: build_input_scheduler(),
            player_systems: build_player_scheduler(),
            monster_systems: build_monster_scheduler(),
            pause_systems: build_pause_scheduler(),
            background_systems: build_title_background_scheduler(),
            background_movement_systems: build_title_background_movement_scheduler(),
            background_move_timer_ms: 0.0,
            options_awaiting: None,
            options_return_to: TurnState::TitleScreen,
            stats_view_mode: StatsViewMode::Overview,
            adventure_mode: AdventureMode::DungeonCrawl,
            adventure_select_cursor: 0,
            class_select_cursor: 0,
            theme_select_cursor: 0,
            pending_theme_choice: ThemeChoice::Random,
            class_select_anim_frame: 0,
            class_select_anim_elapsed_ms: 0.0,
            options_cursor: 0,
            stats_view_cursor: 0,
            item_menu_cursor: MenuCursor::new(),
            pause_cursor: 0,
            enter_not_held_ms: 0.0,
            pending_enter_release: false,
            pause_hint_index: 0,
            pause_hint_timer_ms: 0.0,
            mouse_left_was_down: false,
            death_animation: None,
            victory_animation: None,
            victory_background: None,
            victory_walk_animation: None,
            victory_climb_animation: None,
        };
        state.spawn_title_background();
        state
    }

    /// Builds a fresh game world for a new run, with the player spawned as
    /// `class` - called once when leaving ClassSelect, and again any time
    /// the player returns to the title screen and picks a class to start
    /// over. Replaces the old reset_game_state, which always hardcoded
    /// "Barbarian" at spawn_player instead of taking a chosen class.
    fn start_game(&mut self, class: &str) {
        // Consumed and reset immediately - see pending_theme_choice's own
        // doc comment on State for why (a later ordinary run must never
        // inherit a stale forced theme from an earlier Debug one).
        let theme_choice = self.pending_theme_choice;
        self.pending_theme_choice = ThemeChoice::Random;

        self.ecs = World::default();
        self.resources = Resources::default();
        let mut rng = RandomNumberGenerator::new();
        let mut map_builder = MapBuilder::new(&mut rng, theme_choice.theme());
        let player = spawn_player(&mut self.ecs, map_builder.player_start, class);
        grant_starting_items(&mut self.ecs, player, class);
        // Dungeon Crawl now earns gold too (enemy kills, a guaranteed
        // per-floor chest) toward a shop between floors - see Gold's own
        // doc comment for why this starts at 0 rather than Arena's
        // ARENA_STARTING_GOLD.
        let mut commands = legion::systems::CommandBuffer::new(&self.ecs);
        commands.add_component(player, Gold(0));
        commands.flush(&mut self.ecs);
        let exit_idx = map_builder.map.point2d_to_index(map_builder.amulet_start);
        map_builder.map.tiles[exit_idx] = TileType::Exit;
        spawn_level(&mut self.ecs, &mut rng, 0, &map_builder.monster_spawns);
        spawn_prefab_enemies(&mut self.ecs, &mut rng, 0, &map_builder.prefab_enemy_spawns);
        spawn_prefab_weapon(
            &mut self.ecs,
            &mut rng,
            0,
            map_builder.prefab_weapon_spawn,
            class,
        );
        spawn_boss(&mut self.ecs, &mut rng, 0, map_builder.amulet_start);
        if let Some(chest_pt) = map_builder.prefab_chest_spawn {
            spawn_chest(&mut self.ecs, chest_pt);
            spawn_prefab_chest_guards(&mut self.ecs, 0, &map_builder.prefab_chest_guard_spawns);
        }
        self.resources.insert(map_builder.map);
        self.resources.insert(Camera::new(map_builder.player_start));
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(map_builder.theme);
        self.resources.insert(None::<Battle>);
        self.resources.insert(None::<BattleVictory>);
        self.resources.insert(Keymap::load());
        self.resources.insert(BattleSpeed::load());
        self.resources.insert(AtbMode::load());
        self.resources.insert(MenuMemory::load());
        self.resources.insert(LastBattleAction::load());
        // Always present (see systems/end_turn.rs's Exit-tile branch) -
        // None here means "this is an ordinary dungeon crawl", not
        // "unknown". start_arena is the only place this is ever Some.
        self.resources.insert(None::<ArenaRun>);
        self.resources.insert(None::<ShoppingActive>);
        self.resources.insert(None::<ShopMessage>);
        self.resources.insert(None::<ChestLoot>);
        // So advance_level's own MapBuilder::new call (each later floor)
        // can keep forcing the same theme this run started with - see
        // pending_theme_choice's own doc comment on State.
        self.resources.insert(theme_choice);

        // Counts as "this class was chosen" the instant a run actually
        // begins, regardless of how it later ends (won, lost, or
        // abandoned via quit-to-title) - see Stats::record_game_started.
        let mut stats = Stats::load();
        stats.record_game_started(class, AdventureMode::DungeonCrawl);
        self.resources.insert(stats);
    }

    /// Places this shop's already-decided stock (see
    /// arena::roll_arena_shop_items, which runs BEFORE the room is even
    /// built, so MapBuilder::new_arena_shop can size the counter to fit)
    /// onto `item_points` - one ShopStock counter marker per entry, in
    /// order, each carrying its own Price. Split out of start_arena so the
    /// next slice (a shop reached after a boss kill, not just the starting
    /// one) can call this same logic against a freshly-built shop map
    /// without duplicating it.
    fn spawn_arena_shop_items(&mut self, items: &[(String, i32, i32)], item_points: &[Point]) {
        for ((name, quantity, price), &pt) in items.iter().zip(item_points.iter()) {
            spawn_shop_stock_at(&mut self.ecs, name, pt, *quantity, *price);
        }
    }

    /// Marks a fixed rectangle of `map` as revealed and gives
    /// `player_entity` a frozen FieldOfView covering exactly that
    /// rectangle - no fog of war, no shadowcasting, never recomputed
    /// (see systems/movement.rs's `freeze_los` guard, which skips the
    /// normal dirty-on-move FieldOfView rebuild specifically while this
    /// is active). Shared by build_shop_room (a shop's own Counter tile
    /// would otherwise block sight past it - see MapBuilder::
    /// new_arena_shop) and arena_begin_wave (a wave map has no fog of
    /// war either, by design - the whole arena should read as visible
    /// the instant it loads).
    fn reveal_and_freeze_fov(
        &mut self,
        player_entity: Entity,
        map: &mut Map,
        reveal_x: i32,
        reveal_y: i32,
        reveal_w: i32,
        reveal_h: i32,
    ) {
        let mut full_fov = FieldOfView::new(8);
        for y in reveal_y..(reveal_y + reveal_h) {
            for x in reveal_x..(reveal_x + reveal_w) {
                map.revealed_tiles[map_idx(x, y)] = true;
                full_fov.visible_tiles.insert(Point::new(x, y));
            }
        }
        full_fov.is_dirty = false;
        let mut cb = CommandBuffer::new(&mut self.ecs);
        cb.add_component(player_entity, full_fov);
        cb.flush(&mut self.ecs);
    }

    /// Builds a shop room around `player_entity` and stocks it with
    /// `items` - the mechanical core every "enter a shop" transition
    /// needs (start_arena's very first shop, arena_advance_to_next_shop's
    /// later ones, and Dungeon Crawl's own dungeon_shop_transition), split
    /// out after those three had all accumulated their own copy of it.
    /// Moves `player_entity` to the new map's player_start (works whether
    /// it already existed elsewhere or was just spawned at a throwaway
    /// position moments ago - either way this is what actually places
    /// it), reveals the map's reveal rectangle with no fog of war and a
    /// frozen FieldOfView (see MapBuilder::new_arena_shop's own doc
    /// comment for why - the shopkeeper sits behind an opaque Counter
    /// tile, and real shadowcasting would never let the player see past
    /// it), spawns the decorative Shopkeeper NPC, places the stock via
    /// spawn_arena_shop_items, sets the Exit tile, and inserts the map/
    /// Camera/theme resources.
    ///
    /// Deliberately does NOT touch TurnState/Battle/BattleVictory/
    /// ArenaRun/Gold/Stats - those differ too much between a fresh-world
    /// bootstrap (start_arena) and an in-run transition (the other two)
    /// for a shared helper to guess correctly; every caller still sets
    /// those itself right after calling this.
    fn build_shop_room(
        &mut self,
        player_entity: Entity,
        rng: &mut RandomNumberGenerator,
        items: &[(String, i32, i32)],
    ) {
        let (
            mut map_builder,
            item_points,
            shopkeeper_point,
            reveal_x,
            reveal_y,
            reveal_w,
            reveal_h,
        ) = MapBuilder::new_arena_shop(rng, items.len());

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

        // Shopkeeper - purely decorative for now (no dialogue/trade
        // logic, the items themselves are what's interactive). The
        // plain 'W' Render glyph below is now only a fallback (used if
        // idle_glyph/idle_sheet ever find no IdleAnimation, which
        // shouldn't happen here) - real PixelLab art (2026-09-13) plays
        // instead via the IdleAnimation component, its own dedicated
        // Idle_Selling loop on resources/shopkeeper_idle.png (see
        // components::idle_frames_for_shopkeeper). Built once here and
        // never rebuilt afterward - unlike a player/enemy's IdleAnimation,
        // this entity never moves, so nothing ever needs to rebuild
        // `frames` for a facing change.
        self.ecs.push((
            Name("Shopkeeper".to_string()),
            shopkeeper_point,
            Render {
                color: ColorPair::new(YELLOW, BLACK),
                glyph: to_cp437('W'),
            },
            idle_frames_for_shopkeeper(),
        ));

        self.spawn_arena_shop_items(items, &item_points);

        let exit_idx = map_builder.map.point2d_to_index(map_builder.amulet_start);
        map_builder.map.tiles[exit_idx] = TileType::Exit;

        self.resources.insert(map_builder.map);
        // Plain Camera::new - the standard 40x25 dungeon viewport, same
        // as every other map. A custom smaller camera was tried here
        // first, but it didn't address the actual problem (the map
        // itself still being 80x50 underneath) and complicated other
        // things unnecessarily - the reveal-rectangle approach above is
        // what actually makes this map read as small.
        self.resources.insert(Camera::new(map_builder.player_start));
        self.resources.insert(map_builder.theme);
    }








    /// Reached by stepping on a dungeon floor's own stairs tile in
    /// Dungeon Crawl mode (see TurnState::DungeonShopTransition /
    /// systems/end_turn.rs's 3-way Exit-tile split). Builds a small shop
    /// room - the exact same layout/reveal-rectangle trick as
    /// MapBuilder::new_arena_shop, just stocked with the fixed Healing
    /// Potion/Dungeon Map pair instead of a class-rolled weapon/ability
    /// list, since Dungeon Crawl's shop doesn't vary by class or level -
    /// while keeping the same player entity/inventory (see
    /// arena_rebuild_keep_player, despite the name not Arena-specific).
    /// One-shot, like advance_level: runs once, changes TurnState away
    /// from DungeonShopTransition so it doesn't repeat next frame.
    /// Leaving via this shop's own stairs tile routes back through
    /// end_turn's Exit-tile check, which - now that ShoppingActive is
    /// Some - resolves to TurnState::NextLevel and actually generates the
    /// next floor (see advance_level, which clears ShoppingActive/
    /// ShopMessage again on the way out, since it doesn't otherwise touch
    /// either).
    fn dungeon_shop_transition(&mut self) {
        let player_entity = self.arena_rebuild_keep_player();

        let items: Vec<(String, i32, i32)> = vec![
            (
                "Healing Potion".to_string(),
                DUNGEON_SHOP_POTION_STOCK,
                HEALING_POTION_PRICE,
            ),
            (
                "Dungeon Map".to_string(),
                DUNGEON_SHOP_MAP_STOCK,
                DUNGEON_MAP_PRICE,
            ),
        ];
        let mut rng = RandomNumberGenerator::new();
        self.build_shop_room(player_entity, &mut rng, &items);

        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(Some(ShoppingActive));
        self.resources.insert(None::<ShopMessage>);
    }


    /// Tears down the current run (if any) and returns to the title
    /// screen - called when the player dismisses the GameOver or Victory
    /// screen, instead of immediately starting a new run with whatever
    /// class they last had. ClassSelect (via start_game) is now the only
    /// place a run actually begins. spawn_title_background is defined in
    /// screens/title.rs (a descendant module), which is why it needed to
    /// be marked `pub` - see that file's comment on the same fn.
    ///
    /// Also the single place a run's outcome gets recorded into Stats -
    /// every run-ending path (Victory/GameOver's "press 1" handlers, and
    /// Pause's Q quit-early handler) calls this one function, so none of
    /// those three callers need to know anything about Stats themselves.
    fn return_to_title(&mut self) {
        // Read what play-history needs BEFORE wiping ecs/resources below -
        // both are gone the instant World::default()/Resources::default()
        // run. A run ended via Pause's early quit still has `outcome` at
        // whatever TurnState it was mid-run (AwaitingInput/Paused/etc,
        // never Victory) - that's correctly treated as "no win recorded"
        // below, while the deepest level reached still counts, since the
        // player genuinely got that far.
        let outcome = self.resources.get::<TurnState>().map(|t| *t);
        let player_info = <(&Player, &Class)>::query()
            .iter(&self.ecs)
            .next()
            .map(|(p, c)| (p.map_level, c.0.clone()));
        // Read before the wipe below for the same reason as outcome/
        // player_info above - both are gone the instant Resources::default()
        // runs. adventure_mode is a plain State field (not a resource), so
        // it survives the wipe on its own, but is read here anyway since
        // this fn resets it to DungeonCrawl a few lines down and the
        // Stats-recording code below needs the value as it was DURING the
        // run that's ending, not the post-reset default.
        let arena_run = self
            .resources
            .get::<Option<ArenaRun>>()
            .and_then(|run| *run);
        let mode = self.adventure_mode;

        self.ecs = World::default();
        self.resources = Resources::default();
        self.spawn_title_background();
        self.resources.insert(TurnState::TitleScreen);
        // Resources::default() above wipes Option<Battle> along with
        // everything else - background_movement_systems (random_move_system)
        // needs it present before the next
        // background_movement_systems.execute() call, same reasoning as
        // State::new().
        self.resources.insert(None::<Battle>);
        self.resources.insert(None::<ShoppingActive>);
        self.resources.insert(None::<ArenaRun>);
        // Same reasoning as Option<Battle>/ShoppingActive/ArenaRun above -
        // movement_system now reads Option<ChestLoot> too (see
        // State::new()'s own comment on this exact resource).
        self.resources.insert(None::<ChestLoot>);
        self.resources.insert(None::<ShopMessage>);
        self.resources.insert(Keymap::load());
        self.resources.insert(BattleSpeed::load());
        self.resources.insert(AtbMode::load());
        self.resources.insert(MenuMemory::load());
        self.resources.insert(LastBattleAction::load());
        self.adventure_mode = AdventureMode::DungeonCrawl;
        self.adventure_select_cursor = 0;
        self.class_select_cursor = 0;
        self.theme_select_cursor = 0;
        self.pending_theme_choice = ThemeChoice::Random;
        self.pending_enter_release = false;
        self.enter_not_held_ms = 0.0;
        self.item_menu_cursor = MenuCursor::new();
        self.pause_cursor = 0;
        self.pause_hint_index = 0;
        self.death_animation = None;
        self.victory_animation = None;
        self.victory_background = None;
        self.victory_walk_animation = None;
        self.victory_climb_animation = None;
        self.pause_hint_timer_ms = 0.0;

        let mut stats = Stats::load();
        if let Some((map_level, class)) = player_info {
            if outcome == Some(TurnState::Victory) {
                stats.record_win(&class, mode);
            }
            match mode {
                // Dungeon Crawl's own "how far did I get" - Player::map_level,
                // untouched by Arena code, so only meaningful here.
                AdventureMode::DungeonCrawl => {
                    stats.record_deepest_level(&class, map_level);
                }
                // Arena's own "how far did I get" - by wave, not level
                // alone, and "reached" rather than "cleared" (see
                // record_arena_progress). wave == 0 means the run ended
                // while still browsing a level's shop, before any wave of
                // THAT level was reached - that's already reflected by
                // whatever the previous level's last-reached wave was, so
                // it's deliberately not recorded as new progress here.
                AdventureMode::BattleArena => {
                    if let Some(run) = arena_run {
                        if run.wave >= 1 {
                            stats.record_arena_progress(&class, run.level, run.wave);
                        }
                    }
                }
            }
        }
        self.resources.insert(stats);

        self.background_move_timer_ms = 0.0;
    }

    fn advance_level(&mut self) {
        let (player_entity, _) = find_player(&self.ecs).unwrap();

        use std::collections::HashSet;
        let mut entities_to_keep = HashSet::new();
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

        <&mut FieldOfView>::query()
            .iter_mut(&mut self.ecs)
            .for_each(|fov| fov.is_dirty = true);

        let mut rng = RandomNumberGenerator::new();
        // Keeps forcing whatever theme this run started with (see
        // pending_theme_choice's own doc comment on State/start_game) -
        // ThemeChoice::Random for every non-Debug run, unchanged
        // behavior.
        let theme_choice = *self.resources.get::<ThemeChoice>().unwrap();
        let mut map_builder = MapBuilder::new(&mut rng, theme_choice.theme());
        let mut map_level = 0;
        <(&mut Player, &mut Point)>::query()
            .iter_mut(&mut self.ecs)
            .for_each(|(player, pos)| {
                player.map_level += 1;
                map_level = player.map_level;
                pos.x = map_builder.player_start.x;
                pos.y = map_builder.player_start.y;
            });
        if map_level == 2 {
            spawn_amulet_of_yala(&mut self.ecs, map_builder.amulet_start);
        } else {
            let exit_idx = map_builder.map.point2d_to_index(map_builder.amulet_start);
            map_builder.map.tiles[exit_idx] = TileType::Exit;
        }
        spawn_level(
            &mut self.ecs,
            &mut rng,
            map_level as usize,
            &map_builder.monster_spawns,
        );
        spawn_prefab_enemies(
            &mut self.ecs,
            &mut rng,
            map_level as usize,
            &map_builder.prefab_enemy_spawns,
        );
        let player_class = entity_class(&self.ecs, player_entity).unwrap_or_default();
        spawn_prefab_weapon(
            &mut self.ecs,
            &mut rng,
            map_level as usize,
            map_builder.prefab_weapon_spawn,
            &player_class,
        );
        spawn_boss(
            &mut self.ecs,
            &mut rng,
            map_level as usize,
            map_builder.amulet_start,
        );
        if let Some(chest_pt) = map_builder.prefab_chest_spawn {
            spawn_chest(&mut self.ecs, chest_pt);
            spawn_prefab_chest_guards(
                &mut self.ecs,
                map_level as usize,
                &map_builder.prefab_chest_guard_spawns,
            );
        }
        self.resources.insert(map_builder.map);
        self.resources.insert(Camera::new(map_builder.player_start));
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(map_builder.theme);
        // This is now also reached by leaving the between-floor shop
        // (see TurnState::DungeonShopTransition / systems/end_turn.rs's
        // 3-way Exit-tile split), which sets both of these - clear them
        // here or a freshly generated floor would silently inherit
        // ShoppingActive's auto-pickup suppression and frozen FOV
        // forever, since nothing else on this path ever resets them.
        self.resources.insert(None::<ShoppingActive>);
        self.resources.insert(None::<ShopMessage>);
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        for &console in ALL_CONSOLES {
            ctx.set_active_console(console);
            ctx.cls();
        }
        // See pending_enter_release's own doc comment on State for why
        // this is a debounced "continuously absent for
        // ENTER_RELEASE_DEBOUNCE_MS" check, not a plain "not held this
        // single frame" check - see tick_enter_release_guard.
        let (still_pending, not_held_ms) = tick_enter_release_guard(
            self.pending_enter_release,
            ctx.key == Some(VirtualKeyCode::Return),
            self.enter_not_held_ms,
            ctx.frame_time_ms,
        );
        self.pending_enter_release = still_pending;
        self.enter_not_held_ms = not_held_ms;
        self.resources.insert(ctx.key);
        self.resources.insert(FrameTime(ctx.frame_time_ms));
        ctx.set_active_console(0);
        self.resources.insert(Point::from_tuple(ctx.mouse_pos()));
        ctx.set_active_console(ABILITY_BAR_CONSOLE);
        self.resources
            .insert(AbilityBarMousePos(Point::from_tuple(ctx.mouse_pos())));
        ctx.set_active_console(0);
        // See MouseLeftJustPressed's own doc comment for why this reads
        // real held-state (INPUT's level-triggered query) and diffs it
        // against last frame, rather than trusting BTerm::left_click.
        let mouse_left_down = INPUT.lock().is_mouse_button_pressed(0);
        self.resources.insert(MouseLeftJustPressed(
            mouse_left_down && !self.mouse_left_was_down,
        ));
        self.mouse_left_was_down = mouse_left_down;
        let current_state = self.resources.get::<TurnState>().unwrap().clone();
        match current_state {
            TurnState::TitleScreen => {
                self.title_screen(ctx);
            }
            TurnState::AdventureSelect => {
                self.adventure_select(ctx);
            }
            TurnState::ClassSelect => {
                self.class_select(ctx);
            }
            TurnState::ThemeSelect => {
                self.theme_select(ctx);
            }
            TurnState::AwaitingInput => self
                .input_systems
                .execute(&mut self.ecs, &mut self.resources),
            TurnState::PlayerTurn => {
                self.player_systems
                    .execute(&mut self.ecs, &mut self.resources);
            }
            TurnState::MonsterTurn => self
                .monster_systems
                .execute(&mut self.ecs, &mut self.resources),
            TurnState::Paused => {
                self.paused_tick(ctx);
            }
            TurnState::ItemMenu => {
                self.item_menu_tick(ctx);
            }
            TurnState::Options => {
                self.options_tick(ctx);
            }
            TurnState::StatsView => {
                self.stats_view_tick(ctx);
            }
            TurnState::InBattle => {
                self.battle_tick(ctx);
            }
            TurnState::BattleVictory => {
                self.battle_victory_tick(ctx);
            }
            TurnState::ArenaTransition => {
                self.arena_transition_tick(ctx);
            }
            TurnState::ArenaWaveCleared => {
                self.arena_wave_cleared_tick();
            }
            TurnState::GameOver => {
                self.game_over(ctx);
            }
            TurnState::Victory => {
                self.victory(ctx);
            }
            TurnState::NextLevel => {
                self.advance_level();
            }
            TurnState::ChestOpened => {
                self.chest_loot_tick(ctx);
            }
            TurnState::DungeonShopTransition => {
                self.dungeon_shop_transition();
            }
        }
        render_draw_buffer(ctx).expect("Render error");
    }
}

fn main() -> BError {
    let context = BTermBuilder::new()
        .with_title("Ever Space RRPG")
        // Raised from 30 to 60: with real elapsed-time-based animation
        // (MovingAnimation/FrameTime, not frame counts - see
        // systems/animation.rs), every timed effect in this project
        // already scales correctly at any frame rate. The tile-glide
        // specifically only gets ~4-5 frames total to play out at 30fps
        // for its default MOVE_ANIM_DURATION_MS, which reads as jumpy
        // rather than smooth - doubling the frame budget is the direct
        // fix for that, with no knock-on effect on anything's actual
        // speed (battle flashes, popups, background wandering, etc. all
        // still take exactly as long in real time as before).
        .with_fps_cap(60.0)
        .with_dimensions(DISPLAY_WIDTH, DISPLAY_HEIGHT)
        .with_tile_dimensions(32, 32)
        .with_resource_path("resources/")
        .with_font("dungeonfont.png", 32, 32)
        .with_font("terminal8x8.png", 8, 8)
        .with_font("character_idle.png", 128, 128)
        .with_font("character_battle.png", 32, 32)
        .with_font("character_portrait.png", 32, 32)
        .with_font("enemy_idle.png", 128, 128)
        .with_font("enemy_battle.png", 32, 32)
        .with_font("character_death.png", 32, 32)
        .with_font("character_victory.png", 32, 32)
        .with_font("character_technique.png", 32, 32)
        .with_font("character_attack.png", 32, 32)
        .with_font("character_defend.png", 32, 32)
        .with_font("enemy_attack.png", 32, 32)
        .with_font("enemy_death.png", 32, 32)
        .with_font("shopkeeper_idle.png", 128, 128)
        .with_font("character_effect.png", 32, 32)
        .with_font("map_tiles.png", 32, 32)
        .with_font("battle_backgrounds.png", 1280, 800)
        .with_font("ui_panels.png", 32, 32)
        .with_font("battle_bar_frame.png", 32, 32)
        .with_simple_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        // Console 1 (BATTLE_BACKDROP_CONSOLE): see its own doc comment
        // above for the full reasoning. Registered right after console 0
        // and before MAP_TILE_CONSOLE - the lowest slot in the whole
        // chain apart from console 0 itself, since it has to sit below
        // everything the battle screen draws, not just below the HUD.
        .with_simple_console(1, 1, "battle_backgrounds.png")
        // Console 2 (MAP_TILE_CONSOLE): a plain console, same grid as
        // console 0, sourced from map_tiles.png - see its own doc
        // comment above for the full reasoning, including the real
        // black-screen regression that put it in this specific slot
        // (right after console 0/BATTLE_BACKDROP_CONSOLE) instead of
        // later in the chain.
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "map_tiles.png")
        .with_simple_console_no_bg(SCREEN_WIDTH * 2, SCREEN_HEIGHT * 2, "terminal8x8.png")
        .with_simple_console_no_bg(
            BATTLE_PORTRAIT_COLS,
            BATTLE_PORTRAIT_ROWS,
            "dungeonfont.png",
        )
        // Console 4 (MAP_SCROLL_CONSOLE): a "fancy console" - supports
        // DrawBatch::set_fancy, unlike every plain "simple console" above.
        // Same DISPLAY_WIDTH x DISPLAY_HEIGHT grid/32x32px cells and
        // dungeonfont as console 0, WITH a background (unlike every other
        // fancy console below) so it can fully replace console 0 for a
        // frame without anything showing through around the edges of a
        // tile. Deliberately registered here, right after console 3 and
        // before HUD_CONSOLE, so it stays below the HUD in z-order the
        // same way console 0 always has - see MAP_SCROLL_CONSOLE's own
        // doc comment in the prelude module above for why that ordering
        // matters. Used only by map_render, only on frames where the
        // camera itself is panning.
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        // Console 5 (MAP_TILE_SCROLL_CONSOLE): a fancy console, same
        // grid/font as console 1 - see its own doc comment above.
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "map_tiles.png")
        // Console 6 (ENTITY_CONSOLE): a plain console, same grid as
        // console 0/1, dungeonfont - see its own doc comment above (was
        // the unnamed literal "console 1" before 2026-09-08's regression
        // fix moved it here to make room for MAP_TILE_CONSOLE).
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        // Console 7 (ENTITY_SCROLL_CONSOLE): a second new "fancy
        // console", same grid as console 4 above, transparent background
        // (the same RGBA-alpha-0 trick GLIDE_CONSOLE uses below). Used
        // only by entity_render, on the same frames as MAP_SCROLL_CONSOLE
        // - see its own doc comment in the prelude module above.
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        // Console 8 (CHARACTER_IDLE_CONSOLE): a plain console, same grid
        // as console 0/1 but sourced from the second sheet
        // (character_idle.png) - see its own doc comment above for the
        // full reasoning. No background (transparent everywhere except
        // wherever an entity with real idle-frame art is actually drawn).
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "character_idle.png")
        // Console 9 (CHARACTER_IDLE_SCROLL_CONSOLE): a fancy console,
        // same grid/font as console 8 - see its own doc comment above.
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "character_idle.png")
        // Console 10 (CHARACTER_IDLE_GLIDE_CONSOLE): a second fancy
        // console, same grid/font as console 8 - see its own doc comment
        // above.
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "character_idle.png")
        // Console 11 (ENEMY_IDLE_CONSOLE): a plain console, same grid as
        // console 0/1/8 but sourced from a fourth sheet
        // (enemy_idle.png) - see its own doc comment above. No background
        // (transparent everywhere except wherever an enemy with real
        // idle-frame art is actually drawn).
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "enemy_idle.png")
        // Console 12 (ENEMY_IDLE_SCROLL_CONSOLE): a fancy console, same
        // grid/font as console 11 - see its own doc comment above.
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "enemy_idle.png")
        // Console 13 (ENEMY_IDLE_GLIDE_CONSOLE): a second fancy console,
        // same grid/font as console 11 - see its own doc comment above.
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "enemy_idle.png")
        // Console 14 (CHARACTER_EFFECT_CONSOLE): a plain console, same
        // grid as console 0/1/8/11 but sourced from a fifth sheet
        // (character_effect.png) - see its own doc comment above. No
        // background (transparent everywhere except wherever an entity's
        // EffectAnimation is actually playing).
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "character_effect.png")
        // Console 15 (CHARACTER_EFFECT_SCROLL_CONSOLE): a fancy console,
        // same grid/font as console 14 - see its own doc comment above.
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "character_effect.png")
        // Console 16 (CHARACTER_EFFECT_GLIDE_CONSOLE): a second fancy
        // console, same grid/font as console 14 - see its own doc comment
        // above.
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "character_effect.png")
        .with_simple_console_no_bg(HUD_COLS, HUD_ROWS, "terminal8x8.png")
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "terminal8x8.png")
        // Console 14 (END_SCREEN_FALLEN_CONSOLE): a "fancy console" -
        // supports DrawBatch::set_fancy (sub-pixel position + rotation +
        // scale), unlike every "simple console" above. Same square
        // 32x32px cells as console 0/1 (dungeonfont at native size on the
        // DISPLAY_WIDTH x DISPLAY_HEIGHT grid), so a 90-degree rotation
        // doesn't stretch/squash the glyph. Used only by
        // draw_end_screen_fallen_portrait for the GameOver screen.
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        // Console 15 (GLIDE_CONSOLE): a second "fancy console", same grid
        // and cell size as console 14 above. Used by entity_render to draw
        // any entity mid-tile-glide at a sub-pixel position with a
        // genuinely transparent background, instead of console 1's
        // integer-snapped grid - see the GLIDE_CONSOLE doc comment.
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        // Console 16 (BATTLE_PORTRAIT_WIGGLE_CONSOLE): a third "fancy
        // console", sharing console 3's coarse BATTLE_PORTRAIT_COLS x
        // BATTLE_PORTRAIT_ROWS grid (so cells are automatically just as
        // big, no separate scale needed). Used by draw_battle_arena to
        // give the "Attacking" portrait a small sub-pixel shake - see the
        // BATTLE_PORTRAIT_WIGGLE_CONSOLE doc comment.
        .with_fancy_console(
            BATTLE_PORTRAIT_COLS,
            BATTLE_PORTRAIT_ROWS,
            "dungeonfont.png",
        )
        // Console 17 (DAMAGE_POPUP_CONSOLE): a plain console, identical
        // grid/font/cell-size to BIG_TEXT_CONSOLE above, registered last
        // so it renders above every other console including the fancy
        // portrait-wiggle one - see DAMAGE_POPUP_CONSOLE's own doc
        // comment in the prelude module for why a separate console (not
        // just moving the draw calls) was the actual fix needed.
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "terminal8x8.png")
        // Console 18 (ABILITY_BAR_CONSOLE): a plain console, registered
        // last so it renders above everything else - the dungeon view,
        // the HUD, and every icon console above. See
        // ABILITY_BAR_CONSOLE's own doc comment for the grid shape.
        .with_simple_console_no_bg(ABILITY_BAR_COLS, ABILITY_BAR_ROWS, "dungeonfont.png")
        // Console 19 (BUFF_BADGE_CONSOLE): a plain console, registered
        // last of all so a badge always paints over the portrait/bars
        // beneath it. See BUFF_BADGE_CONSOLE's own doc comment for why
        // this grid (32px cells) rather than reusing ABILITY_BAR_CONSOLE.
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        // Console 20 (ABILITY_BAR_BADGE_CONSOLE): a plain console,
        // registered last of all so a stack-count badge always paints
        // over the bar icons beneath it. See its own doc comment for why
        // this needs to be separate from HUD_CONSOLE despite sharing its
        // exact grid/font.
        .with_simple_console_no_bg(HUD_COLS, HUD_ROWS, "terminal8x8.png")
        // Console 21 (CLASS_SELECT_IDLE_CONSOLE): a plain console, same
        // coarse grid as console 3, sourced from character_idle.png - see
        // its own doc comment above. Registered last of all.
        .with_simple_console_no_bg(
            BATTLE_PORTRAIT_COLS,
            BATTLE_PORTRAIT_ROWS,
            "character_idle.png",
        )
        // Console 24 (CHARACTER_BATTLE_CONSOLE): a plain console, same
        // coarse grid as console 3, sourced from character_battle.png -
        // see its own doc comment above.
        .with_simple_console_no_bg(
            BATTLE_PORTRAIT_COLS,
            BATTLE_PORTRAIT_ROWS,
            "character_battle.png",
        )
        // Console 25 (CHARACTER_BATTLE_WIGGLE_CONSOLE): a fancy console,
        // same grid/font as console 24 - see its own doc comment above.
        .with_fancy_console(
            BATTLE_PORTRAIT_COLS,
            BATTLE_PORTRAIT_ROWS,
            "character_battle.png",
        )
        // Console 26 (CHARACTER_PORTRAIT_BIG_CONSOLE): a plain
        // console, same coarse grid as console 3, sourced from
        // character_portrait.png - see its own doc comment above.
        .with_simple_console_no_bg(
            BATTLE_PORTRAIT_COLS,
            BATTLE_PORTRAIT_ROWS,
            "character_portrait.png",
        )
        // Console 27 (CHARACTER_PORTRAIT_HUD_CONSOLE): a plain console,
        // same grid as ABILITY_BAR_CONSOLE, sourced from
        // character_portrait.png - see its own doc comment above.
        .with_simple_console_no_bg(ABILITY_BAR_COLS, ABILITY_BAR_ROWS, "character_portrait.png")
        // Console 28 (END_SCREEN_FALLEN_PORTRAIT_CONSOLE): a fancy
        // console, same grid/cell-size as END_SCREEN_FALLEN_CONSOLE, but
        // sourced from character_portrait.png - see its own doc comment
        // above.
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "character_portrait.png")
        // Console 29 (ENEMY_BATTLE_CONSOLE): a plain console, same coarse
        // grid as console 3, sourced from enemy_battle.png - see its own
        // doc comment above.
        .with_simple_console_no_bg(
            BATTLE_PORTRAIT_COLS,
            BATTLE_PORTRAIT_ROWS,
            "enemy_battle.png",
        )
        // Console 30 (ENEMY_BATTLE_WIGGLE_CONSOLE): a fancy console, same
        // grid/font as console 29 - see its own doc comment above.
        .with_fancy_console(
            BATTLE_PORTRAIT_COLS,
            BATTLE_PORTRAIT_ROWS,
            "enemy_battle.png",
        )
        // Console 31 (CHARACTER_DEATH_CONSOLE): a plain console, same
        // coarse grid as console 3, sourced from character_death.png -
        // see its own doc comment above.
        .with_simple_console_no_bg(
            BATTLE_PORTRAIT_COLS,
            BATTLE_PORTRAIT_ROWS,
            "character_death.png",
        )
        // Console (CHARACTER_DEATH_GLIDE_CONSOLE): a fancy console, same
        // grid/font as the plain one just above - see its own doc
        // comment above.
        .with_fancy_console(
            BATTLE_PORTRAIT_COLS,
            BATTLE_PORTRAIT_ROWS,
            "character_death.png",
        )
        // Console 32 (CHARACTER_VICTORY_CONSOLE): a plain console, same
        // coarse grid as console 3, sourced from character_victory.png -
        // see its own doc comment above.
        .with_simple_console_no_bg(
            BATTLE_PORTRAIT_COLS,
            BATTLE_PORTRAIT_ROWS,
            "character_victory.png",
        )
        // Console 33 (CHARACTER_TECHNIQUE_CONSOLE): a plain console, same
        // coarse grid as console 3, sourced from character_technique.png -
        // see its own doc comment above.
        .with_simple_console_no_bg(
            BATTLE_PORTRAIT_COLS,
            BATTLE_PORTRAIT_ROWS,
            "character_technique.png",
        )
        // Console 34 (CHARACTER_ATTACK_CONSOLE): a plain console, same
        // coarse grid as console 3, sourced from character_attack.png -
        // see its own doc comment above.
        .with_simple_console_no_bg(
            BATTLE_PORTRAIT_COLS,
            BATTLE_PORTRAIT_ROWS,
            "character_attack.png",
        )
        // Console 35 (CHARACTER_DEFEND_CONSOLE): a plain console, same
        // coarse grid as console 3, sourced from character_defend.png -
        // see its own doc comment above.
        .with_simple_console_no_bg(
            BATTLE_PORTRAIT_COLS,
            BATTLE_PORTRAIT_ROWS,
            "character_defend.png",
        )
        // Console 36 (ENEMY_ATTACK_CONSOLE): a FANCY console (needs
        // fractional set_fancy positioning, unlike the two plain consoles
        // just above - see its own doc comment above), same coarse grid,
        // sourced from enemy_attack.png.
        .with_fancy_console(
            BATTLE_PORTRAIT_COLS,
            BATTLE_PORTRAIT_ROWS,
            "enemy_attack.png",
        )
        // Console 37 (ENEMY_DEATH_CONSOLE): a FANCY console, same coarse
        // grid, sourced from enemy_death.png - see its own doc comment
        // above.
        .with_fancy_console(
            BATTLE_PORTRAIT_COLS,
            BATTLE_PORTRAIT_ROWS,
            "enemy_death.png",
        )
        // Consoles 42/43/44 (SHOPKEEPER_IDLE_CONSOLE/_SCROLL_/_GLIDE_):
        // same plain/fancy/fancy trio shape as character_idle.png's own
        // three consoles above, sourced from shopkeeper_idle.png - see
        // that const's own doc comment in the prelude module.
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "shopkeeper_idle.png")
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "shopkeeper_idle.png")
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "shopkeeper_idle.png")
        // Console 46 (UI_PANEL_CONSOLE): fancy, console 0's own grid,
        // sourced from ui_panels.png - see its own doc comment above.
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "ui_panels.png")
        // Console 47 (PANEL_TEXT_CONSOLE): plain (no_bg), same grid as
        // HUD_CONSOLE, terminal8x8.png - see its own doc comment above.
        .with_simple_console_no_bg(HUD_COLS, HUD_ROWS, "terminal8x8.png")
        // Console 48 (ABILITY_BAR_ICON_CONSOLE): plain (no_bg), same grid
        // as ABILITY_BAR_CONSOLE, dungeonfont.png - see its own doc
        // comment above.
        .with_simple_console_no_bg(ABILITY_BAR_COLS, ABILITY_BAR_ROWS, "dungeonfont.png")
        // Console 49 (ABILITY_BAR_ICON_BADGE_CONSOLE): plain (no_bg),
        // same grid as ABILITY_BAR_BADGE_CONSOLE, terminal8x8.png - see
        // its own doc comment above.
        .with_simple_console_no_bg(HUD_COLS, HUD_ROWS, "terminal8x8.png")
        // Console 50 (BATTLE_BAR_CONSOLE): fancy, console 0's own grid,
        // sourced from battle_bar_frame.png - see its own doc comment
        // above.
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "battle_bar_frame.png")
        // Console 51 (BATTLE_BAR_TEXT_CONSOLE): plain (no_bg), same grid
        // as HUD_CONSOLE, terminal8x8.png - see its own doc comment
        // above.
        .with_simple_console_no_bg(HUD_COLS, HUD_ROWS, "terminal8x8.png")
        .with_vsync(false)
        .build()?;

    main_loop(context, State::new())
}
