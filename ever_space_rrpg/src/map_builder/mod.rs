use crate::prelude::*;
mod empty;
mod rooms;
use rooms::RoomsArchitect;
mod automata;
use automata::CellularAutomataArchitect;
mod drunkard;
use drunkard::DrunkardsWalkArchitect;
mod prefab;
use prefab::{apply_chest, apply_prefab};
mod themes;
pub use themes::*;

trait MapArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator) -> MapBuilder;
}

/// What kind of scenery a theme's battle arena should get, beyond the
/// floor/wall tint every theme has. New themes pick whichever fits - a
/// theme isn't limited to these two forever, this just covers what's
/// implemented in battle_tick so far.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BattleScenery {
    /// A handful of large tree/foliage silhouettes scattered around the
    /// arena (outdoor themes).
    ScatteredTrees,
    /// Thick stone walls running down the left/right sides, so the arena
    /// reads as an enclosed room (indoor/dungeon themes).
    RoomWalls,
}

/// See `MapTheme::end_scene_theme`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndSceneTheme {
    Forest,
    Dungeon,
    Sewer,
}

/// A Debug-run's forced-theme choice (see `TurnState::ThemeSelect`) -
/// `Random` means "behave exactly like every other class", the normal
/// `MapBuilder::new` per-floor roll from `dungeon_theme_pool()`; the
/// other three force every floor of the run to that one theme instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeChoice {
    Random,
    Forest,
    Dungeon,
    Sewer,
}

impl ThemeChoice {
    pub const ALL: [ThemeChoice; 4] = [
        ThemeChoice::Random,
        ThemeChoice::Forest,
        ThemeChoice::Dungeon,
        ThemeChoice::Sewer,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ThemeChoice::Random => "Random",
            ThemeChoice::Forest => "Forest",
            ThemeChoice::Dungeon => "Dungeon",
            ThemeChoice::Sewer => "Sewer",
        }
    }

    /// The real `MapTheme` this choice forces - `None` for `Random`,
    /// which means "let `MapBuilder::new` roll one normally instead".
    pub fn theme(self) -> Option<Box<dyn MapTheme>> {
        match self {
            ThemeChoice::Random => None,
            ThemeChoice::Forest => Some(ForestTheme::new()),
            ThemeChoice::Dungeon => Some(DungeonTheme::new()),
            ThemeChoice::Sewer => Some(SewerTheme::new()),
        }
    }
}

pub trait MapTheme: Sync + Send {
    fn tile_to_render(&self, tile_type: TileType) -> FontCharType;
    /// Base color for this theme's floor tiles - used both for the dungeon
    /// view (future use) and to tint the battle screen's arena background.
    fn floor_color(&self) -> RGB;
    /// Base color for this theme's wall tiles - used to frame the battle
    /// screen's arena background.
    fn wall_color(&self) -> RGB;
    /// Which battle arena scenery this theme uses.
    fn battle_scenery(&self) -> BattleScenery;
    /// This theme's starting row on the shared `resources/map_tiles.png`
    /// atlas - `None` for a theme still on the old single-glyph
    /// `tile_to_render` rendering (see docs/Map_Tile_Theme_Guide.md).
    /// Every migrated theme owns a fixed 4-row block there (floor/wall/
    /// themed-floor/special-wall, MAP_TILE_COLS columns each) - the same
    /// "one shared sheet, more rows" shape CHARACTER_IDLE_CONSOLE/
    /// ENEMY_IDLE_CONSOLE already use, just keyed by theme instead of by
    /// class/enemy name. Defaults to `None` so a brand new MapTheme impl
    /// doesn't have to know about this system until it actually gets
    /// real tile art.
    fn tile_row(&self) -> Option<u16> {
        None
    }
    /// How many distinct floor textures this theme's FLOOR pool has -
    /// defaults to every existing theme's shape (`MAP_TILE_COLS * 2`:
    /// row `+0` basic floor plus row `+2` themed floor, `MAP_TILE_COLS`
    /// each - the atlas's actual column width, a fixed property of the
    /// shared image, not something a theme can vary). Variants
    /// `0..MAP_TILE_COLS` render off row `+0`, the rest off row `+2` -
    /// see `components::map_tile_glyph`. A theme with fewer real floor
    /// variants than a full two rows can just return a smaller count;
    /// the unused cells at the end of row `+2` simply never get rolled.
    fn floor_variant_count(&self) -> u8 {
        MAP_TILE_COLS as u8 * 2
    }
    /// How many distinct wall textures this theme's WALL pool has -
    /// defaults to every existing theme's shape (row `+1` only,
    /// `MAP_TILE_COLS` variants). See `floor_variant_count`'s own doc
    /// comment for the same "fewer than a full row is fine" note - a
    /// theme can also return 1 (no accent variants at all;
    /// `MapBuilder::assign_tile_variants` skips wall-accent rolling
    /// entirely in that case rather than panicking on an empty range).
    fn wall_variant_count(&self) -> u8 {
        MAP_TILE_COLS as u8
    }
    /// This theme's dedicated cell for `TileType::Exit`, as raw
    /// (row, col) coordinates into `resources/map_tiles.png` - NOT
    /// relative to `tile_row()`'s floor/wall block, since Exit is a
    /// single rare tile with no variant pool of its own, not part of
    /// that per-theme 4-row shape. Defaults to `None`, which falls back
    /// to the old flat dungeonfont glyph (`tile_to_render`) - exactly
    /// the same "no real art yet" fallback `tile_row() == None` already
    /// gives Floor/Wall, just applied per-tile-type instead of for the
    /// whole theme at once.
    fn exit_tile(&self) -> Option<(u16, u16)> {
        None
    }
    /// This theme's dedicated cell for `TileType::Counter` - see
    /// `exit_tile`'s own doc comment, same shape and same reasoning.
    fn counter_tile(&self) -> Option<(u16, u16)> {
        None
    }
    /// This theme's row on the shared `resources/battle_backgrounds.png`
    /// sheet (one full 1280x800 painted arena scene per row, no columns -
    /// see `BATTLE_BACKDROP_CONSOLE`'s own doc comment in main.rs)  -
    /// `None` for a theme still on the old procedural tinted-glyph fill
    /// (`tile_to_render`/`floor_color`/`wall_color`/`battle_scenery` in
    /// `draw_battle_arena`). Defaults to `None` so a brand new `MapTheme`
    /// impl doesn't need real battle-background art to compile.
    fn battle_background_row(&self) -> Option<u16> {
        None
    }
    /// Which family of Victory/Game Over backdrops (see components::
    /// VictoryBackground/DefeatBackground) a Dungeon Crawl run through
    /// this theme should use - screens/end.rs reads this off the live
    /// `Box<dyn MapTheme>` resource the instant a run ends, so the End
    /// screen always matches whichever theme the player was actually
    /// exploring (a Forest run showing a sewer's Victory scene made no
    /// sense - see the 2026-09-13 replan in docs/journal.md). Battle
    /// Arena runs never call this at all (is_arena short-circuits to
    /// VictoryBackground::arena()/DefeatBackground::arena() instead).
    /// Defaults to `EndSceneTheme::Dungeon` so a brand new MapTheme impl
    /// still compiles/renders something reasonable before it has its own
    /// dedicated End-screen art - same reasoning as battle_background_
    /// row's own default.
    fn end_scene_theme(&self) -> EndSceneTheme {
        EndSceneTheme::Dungeon
    }
    /// (back_row, front_row) for `screens/battle.rs`'s multi-enemy
    /// zigzag formation (`enemy_portrait_position`) - how far up the
    /// screen (in BATTLE_PORTRAIT_COLS x BATTLE_PORTRAIT_ROWS grid units)
    /// this theme's own battle backdrop art lets enemies stand before
    /// they'd overlap its perimeter scenery. Defaults to (1.9, 2.3) -
    /// Forest's own hard ceiling, screenshot-verified against the real
    /// art (its tree/fence perimeter reaches noticeably lower into frame
    /// than Dungeon/Sewer's thin top wall does) - a safe, conservative
    /// default for any theme that hasn't had this checked yet, rather
    /// than one that could silently clip a future theme's own art.
    /// Override per-theme once its own headroom has actually been
    /// checked against real background art the same way, not guessed.
    fn enemy_formation_rows(&self) -> (f32, f32) {
        (1.9, 2.3)
    }
    /// Which placement style floor variant `variant` (1..floor_variant_
    /// count() - variant 0 is always the plain default, never patched or
    /// scattered) should use - see `VariantStyle`. Defaults to `Patch`
    /// for every variant, matching the original "blocks of leaves,
    /// blocks of moss" design. Override per-variant for anything that
    /// reads as a discrete point fixture rather than a spreadable ground
    /// cover - confirmed needed in real play (2026-09-08): Dungeon's
    /// "torchlight glow" patched as a whole region looked like a wall of
    /// torches, which no real dungeon would have. See
    /// docs/Map_Tile_Theme_Guide.md for the full row-9-12 breakdown of
    /// which cells need which style, theme by theme. Meaningless for any
    /// variant this theme's own `path_variants` also names - see that
    /// method's own doc comment for why those two are excluded from the
    /// ordinary patch/scatter pools entirely, regardless of what this
    /// would say for them.
    fn floor_variant_style(&self, _variant: u8) -> VariantStyle {
        VariantStyle::Patch
    }
    /// (vertical, horizontal) floor variants this theme wants stamped as
    /// ONE real connected line between `player_start` and `amulet_
    /// start`, instead of the ordinary random Patch/Scatter treatment
    /// every other floor variant gets - `None` (the default) for a
    /// theme with no such concept. `MapBuilder::assign_tile_variants`
    /// excludes both variants from the normal patch/scatter candidate
    /// pools when this is `Some`, so nothing else ever paints over (or
    /// duplicates) the line - this is the ONLY thing that places them.
    /// Both are real, distinct atlas cells (not the same texture
    /// reused) - `vertical` is the plain north-south trail, `horizontal`
    /// is that exact same trail pre-rotated 90 degrees and stored as its
    /// own cell (2026-09-13 - Forest repurposed its unused Path Fork
    /// cell for this, after several rounds of a LIVE `set_fancy`
    /// rotation producing real, unfixable rendering seams on this
    /// project's specific bracket-terminal setup; see docs/journal.md's
    /// full account). `MapBuilder::stamp_theme_path` decides which of
    /// the two each tile actually gets, purely from how that tile
    /// connects to its own path neighbors - not this method's concern.
    fn path_variants(&self) -> Option<(u8, u8)> {
        None
    }
    /// This theme's `TileType::Water` cells, as column offsets within
    /// row `tile_row() + 3` (the "special wall" row every migrated
    /// theme reserves - see docs/Map_Tile_Theme_Guide.md) - a theme can
    /// register more than one distinct look (Sewer has both a Standing
    /// Sewage Water and a separate Toxic Sludge Pool), each addressed by
    /// its position in this list (`Map::tile_variant` indexes into it,
    /// the same way it already indexes into the Floor/Wall pools).
    /// Defaults to empty - no theme places `TileType::Water` anywhere
    /// unless it opts in here AND in at least one of `fortress_moat_
    /// variant`/`chest_moat_variant`/`wall_water_patch_variant` below,
    /// which is what actually decides where it goes. See `TileType::
    /// Water`'s own doc comment for the blocking-but-see-through
    /// mechanic this exists for.
    fn water_variants(&self) -> Vec<u16> {
        Vec::new()
    }
    /// Index into `water_variants()` to use for whichever of the three
    /// `apply_prefab` room shapes (FORTRESS/TURRET/BUNKER) gets picked,
    /// instead of the usual plain `TileType::Wall` on its ring - `None`
    /// (the default) keeps every one of them plain Wall. `apply_prefab`
    /// only actually applies this some of the time (`PREFAB_MOAT_
    /// CHANCE_PCT`, 2026-09-13 - "have a chance to be surrounded by
    /// water", after Fortress-only-and-always turned out to read oddly
    /// once Turret/Bunker showed up looking untouched next to it), so
    /// even a theme with this set to `Some` won't water EVERY placement.
    /// A moat still fully blocks the same way a wall did (see
    /// `TileType::Water`), it just lets the player see the room's
    /// interior/guards through it instead of a solid brick face - not
    /// every theme has a liquid that fits this (Dungeon deliberately
    /// doesn't - a stone dungeon reads oddly with a moat).
    fn prefab_moat_variant(&self) -> Option<u8> {
        None
    }
    /// Same as `prefab_moat_variant`, for the CHEST_ROOM prefab's own
    /// wall ring instead - independent of it (a theme can use a
    /// different `water_variants()` index for each, or only implement
    /// one of the two), and NOT subject to `PREFAB_MOAT_CHANCE_PCT` -
    /// the chest room stays deterministic (always moated when `Some`),
    /// since only the Fortress/Turret/Bunker trio was asked to vary.
    fn chest_moat_variant(&self) -> Option<u8> {
        None
    }
    /// Index into `water_variants()` to use for a handful of small,
    /// isolated patches converted from ordinary `TileType::Wall` tiles
    /// elsewhere on the map (NOT tied to any prefab) - purely cosmetic,
    /// since Water is exactly as blocking as the Wall it replaces, just
    /// see-through. `None` (the default) places none.
    fn wall_water_patch_variant(&self) -> Option<u8> {
        None
    }
    /// Extra `TileType::Wall` variants beyond the normal `wall_variant_
    /// count()` pool, for a theme's solid "special wall" feature tiles
    /// (a stump, a rubble pile, a broken pillar - anything in row
    /// `tile_row() + 3` that ISN'T one of `water_variants()` above).
    /// These behave exactly like any other Wall tile (fully blocking AND
    /// opaque, unlike Water) - the only thing that sets them apart is
    /// how rarely `MapBuilder::assign_tile_variants` places them
    /// (deliberately much sparser than the normal per-tile wall accent,
    /// with a hard cap of at most one per 5x5 area so they read as a
    /// genuine rare feature rather than a repeated pattern - 2026-09-13,
    /// "they can be used sparingly... shouldn't have more than 1 in an
    /// area for 5x5"). Variant numbers continue right where the normal
    /// wall pool leaves off (`wall_variant_count()..wall_variant_count()
    /// + MAP_TILE_COLS`, mirroring how `floor_variant_count` already
    /// spans two rows) - see `components::map_tile_glyph`. Defaults to
    /// empty; a theme lists exactly the column offsets (within row
    /// `tile_row() + 3`) it wants in this pool, skipping whichever
    /// columns `water_variants()` already claims.
    fn wall_obstacle_variants(&self) -> Vec<u16> {
        Vec::new()
    }
}

/// How a non-default floor variant gets placed on the map - see
/// `MapTheme::floor_variant_style` and `MapBuilder::assign_tile_variants`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VariantStyle {
    /// A contiguous, randomly-placed, randomly-sized region gets stamped
    /// with this variant - natural for anything that reads as a
    /// spreadable ground cover (a dirt patch, moss, a puddle, an algae
    /// bloom).
    Patch,
    /// Individual tiles scattered sparsely, one at a time, never as a
    /// contiguous block - natural for a discrete point fixture that
    /// would look absurd repeated in bulk right next to itself (a torch,
    /// a drain grate, a pile of bones, a pillar base).
    Scatter,
}

/// Every dungeon-crawl `MapTheme`, in one place - `MapBuilder::new` picks
/// one at random from exactly this list. Add a new theme here (plus its
/// own `MapTheme` impl in `themes.rs`) and it's automatically in the
/// random pool - no range/match bookkeeping needed anywhere else, unlike
/// the old `rng.range(0, 2)` + match this replaced (2026-09-08, when
/// Sewer became the third theme and hand-updating a hardcoded range for
/// every new one stopped being worth it).
fn dungeon_theme_pool() -> Vec<Box<dyn MapTheme>> {
    vec![DungeonTheme::new(), ForestTheme::new(), SewerTheme::new()]
}

/// Columns on `resources/map_tiles.png` - every theme's 4-row block uses
/// this many columns per row, matching the 16-cell (4x4) template in
/// docs/Map_Tile_Theme_Guide.md. A fixed property of the shared atlas
/// image itself, not per-theme - see `MapTheme::floor_variant_count`/
/// `wall_variant_count` for what IS per-theme now (how many of those
/// columns' worth of variants a theme actually uses).
pub const MAP_TILE_COLS: u16 = 4;
/// Percent chance (0-99) an individual Wall tile swaps from the theme's
/// default wall texture (variant 0) to a random accent variant - see
/// MapBuilder::assign_tile_variants. Deliberately a MINORITY so the
/// default stays the dominant wall texture.
const WALL_ACCENT_CHANCE_PCT: i32 = 30;
/// How many floor texture patches get stamped per generated map - see
/// MapBuilder::assign_tile_variants.
const FLOOR_PATCH_COUNT_MIN: i32 = 6;
const FLOOR_PATCH_COUNT_MAX: i32 = 12;
/// Radius (tiles) of one floor texture patch - see
/// MapBuilder::assign_tile_variants.
const FLOOR_PATCH_RADIUS_MIN: i32 = 2;
const FLOOR_PATCH_RADIUS_MAX: i32 = 4;
/// Percent chance (0-99) an individual Floor tile swaps to a random
/// Scatter-style variant (see `MapTheme::floor_variant_style`) - much
/// lower than WALL_ACCENT_CHANCE_PCT on purpose: these are meant to be
/// rare, discrete highlights (a torch, a grate) across a whole floor
/// area, not a dense accent the way wall rubble/rock can be along a
/// boundary.
const FLOOR_SCATTER_CHANCE_PCT: i32 = 6;
/// Percent chance (0-99) an individual Wall tile becomes a candidate for
/// one of its theme's `wall_obstacle_variants` (a stump, a rubble pile -
/// see that method's own doc comment) - deliberately far rarer than
/// WALL_ACCENT_CHANCE_PCT (2026-09-13, "I don't want a lot of them...
/// used sparingly"). A candidate still only actually places if
/// WALL_OBSTACLE_MIN_SPACING's own check passes too.
const WALL_OBSTACLE_CHANCE_PCT: i32 = 4;
/// How many tiles away (each direction) an existing obstacle variant
/// blocks a new one from placing - 2 means "at most one per 5x5 area"
/// (2026-09-13's own phrasing), since a 5-wide block centered on a
/// candidate spans -2..=2.
const WALL_OBSTACLE_MIN_SPACING: i32 = 2;
/// How many small isolated Wall-to-Water patches get stamped per
/// generated map, for a theme with a `wall_water_patch_variant` - see
/// MapBuilder::assign_tile_variants. Deliberately just 0-2, much fewer
/// than FLOOR_PATCH_COUNT_MIN/MAX - this is a rare cosmetic accent
/// ("a small patch," 2026-09-13), not a recurring floor-level feature.
const WALL_WATER_PATCH_COUNT_MIN: i32 = 0;
const WALL_WATER_PATCH_COUNT_MAX: i32 = 2;
/// Radius (tiles) of one Wall-to-Water patch - smaller than a floor
/// patch's own radius, matching "a small patch" rather than a floor-
/// sized region.
const WALL_WATER_PATCH_RADIUS: i32 = 1;
/// Percent chance (0-99) that whichever of the three `apply_prefab` room
/// shapes (FORTRESS/TURRET/BUNKER) gets picked this time actually gets
/// its theme's `prefab_moat_variant` applied, for a theme that has one
/// configured - see that method's own doc comment. Tune this if 50/50
/// reads as too common or too rare in practice.
pub(crate) const PREFAB_MOAT_CHANCE_PCT: i32 = 50;

const NUM_ROOMS: usize = 20;
pub struct MapBuilder {
    pub map: Map,
    pub rooms: Vec<Rect>,
    pub monster_spawns: Vec<Point>,
    pub player_start: Point,
    pub amulet_start: Point,
    pub theme: Box<dyn MapTheme>,
    /// Guaranteed enemy spawn points from a placed prefab's 'M'
    /// markers (see map_builder::prefab). Spawned directly via
    /// spawn_prefab_enemies rather than mixed into the general
    /// monster_spawns lottery, so a prefab's guards are always actually
    /// monsters - previously they were just added to monster_spawns,
    /// where the normal weighted pick could just as easily hand that
    /// point an item instead.
    pub prefab_enemy_spawns: Vec<Point>,
    /// Guaranteed weapon spawn point from a placed prefab's 'S' marker,
    /// if a prefab placed successfully this level (see
    /// map_builder::prefab - placement can fail, so this may be None).
    /// Shared by Swords and Staffs (see spawn_prefab_weapon) - a level
    /// gets one or the other here, not both. Neither family spawns in the
    /// general ambient pool at all - see Template::prefab_only.
    pub prefab_weapon_spawn: Option<Point>,
    /// Guaranteed loot-chest spawn point from the chest room's 'C'
    /// marker, if it placed successfully this level (see
    /// map_builder::prefab::apply_chest - placement can fail, same
    /// best-effort attempt loop as apply_prefab). Unlike the
    /// Fortress/Turret/Bunker roll above, the chest room is always
    /// attempted, not one-of-three-random - see MapBuilder::new.
    pub prefab_chest_spawn: Option<Point>,
    /// Guard spawn points from the chest room's 'M' markers (1 or 2,
    /// decided at placement time) - see
    /// spawner::spawn_prefab_chest_guards, which always picks this
    /// dungeon level's single toughest non-boss enemy for these, unlike
    /// prefab_enemy_spawns' weighted pool above.
    pub prefab_chest_guard_spawns: Vec<Point>,
}

impl MapBuilder {
    /// An empty, all-default `MapBuilder` for an architect to build on -
    /// every architect's own `new` used to repeat this exact struct
    /// literal (including the same throwaway `DungeonTheme` placeholder
    /// for `theme`, immediately overwritten by `MapBuilder::new` once the
    /// architect returns), one copy per architect to keep in sync by
    /// hand. `theme` here is never actually read as `DungeonTheme` by
    /// anything - it only exists so the field has a value before the
    /// real theme gets assigned.
    fn blank() -> Self {
        Self {
            map: Map::new(),
            rooms: Vec::new(),
            monster_spawns: Vec::new(),
            player_start: Point::zero(),
            amulet_start: Point::zero(),
            theme: themes::DungeonTheme::new(),
            prefab_enemy_spawns: Vec::new(),
            prefab_weapon_spawn: None,
            prefab_chest_spawn: None,
            prefab_chest_guard_spawns: Vec::new(),
        }
    }

    /// `forced_theme` - see `ThemeChoice::theme` - overrides the normal
    /// random per-floor pick when `Some` (a Debug-run's `ThemeSelect`
    /// choice, see `TurnState::ThemeSelect`); every other caller (a
    /// non-Debug class, the title screen's own decorative background)
    /// passes `None` for the unchanged random-roll behavior. Applied
    /// BEFORE `apply_prefab`/`apply_chest` below (2026-09-13 - moved
    /// earlier than its original spot, right before
    /// `assign_tile_variants`, so those two can read the real theme's
    /// own `prefab_moat_variant`/`chest_moat_variant` instead of the
    /// still-unset placeholder) and, transitively, well before
    /// `assign_tile_variants` too - that call reads `mb.theme`'s own
    /// `floor_variant_style`, whose variant-index meaning differs per
    /// theme, so the theme has to be real before ANY of these three run.
    pub fn new(rng: &mut RandomNumberGenerator, forced_theme: Option<Box<dyn MapTheme>>) -> Self {
        let mut architect: Box<dyn MapArchitect> = match rng.range(0, 3) {
            0 => Box::new(DrunkardsWalkArchitect {}),
            1 => Box::new(RoomsArchitect {}),
            _ => Box::new(CellularAutomataArchitect {}),
        };
        let mut mb = architect.new(rng);

        mb.theme = match forced_theme {
            Some(theme) => theme,
            None => {
                let mut pool = dungeon_theme_pool();
                let pick = rng.random_slice_index(&pool).unwrap();
                pool.swap_remove(pick)
            }
        };

        apply_prefab(&mut mb, rng);
        apply_chest(&mut mb, rng);
        mb.assign_tile_variants(rng);

        mb
    }

    /// Assigns a texture variant to every Floor/Wall tile on the map,
    /// once, and stores it in `map.tile_variant` - see that field's own
    /// doc comment for why this is done once at generation time rather
    /// than picked fresh on every draw. A no-op if the active theme has
    /// no real tile art yet (`theme.tile_row()` is `None`) -
    /// `tile_variant` just stays at its default 0 for every tile, which
    /// is harmless since nothing reads it in that case anyway (see
    /// components::tile_render_at). Must be called AFTER every tile is
    /// in its final TileType for this map/theme - anything that changes
    /// a tile's TileType afterward (there isn't any today) would leave
    /// that tile with a stale variant rolled for its old type.
    ///
    /// Deliberately NOT a uniform per-tile random pick across the whole
    /// variant pool - confirmed too noisy in practice (floor and wall
    /// became hard to tell apart at a glance, since a per-tile roll has
    /// no larger-scale pattern for the eye to key off). Instead: every
    /// tile starts on its theme's plain default look (variant 0 - tile
    /// #1/#5 in the theme's own 16-cell template, see
    /// docs/Map_Tile_Theme_Guide.md, which is why those two specifically
    /// have to be the theme's plain/unremarkable texture), then two
    /// different rules layer real variety on top of that base:
    fn assign_tile_variants(&mut self, rng: &mut RandomNumberGenerator) {
        if self.theme.tile_row().is_none() {
            return;
        }
        for variant in self.map.tile_variant.iter_mut() {
            *variant = 0;
        }

        // Wall accents: a scattered MINORITY of individual wall tiles
        // swap to a different wall variant - deliberately per-tile, not
        // a patch, since a wall is usually only one tile thick and has
        // no width for a "block" to read differently from a scatter
        // anyway. Trees (or whatever variant 0 is) stay the dominant
        // wall texture; rock/rubble/thicket read as occasional accents.
        // Skipped entirely for a theme with only one wall variant -
        // `rng.range(1, 1)` would be an empty range and panic, and
        // there'd be nothing non-default to accent to anyway.
        let wall_variant_count = self.theme.wall_variant_count();
        if wall_variant_count > 1 {
            for idx in 0..self.map.tiles.len() {
                if self.map.tiles[idx] == TileType::Wall
                    && rng.range(0, 100) < WALL_ACCENT_CHANCE_PCT
                {
                    self.map.tile_variant[idx] = rng.range(1, wall_variant_count);
                }
            }
        }

        // Non-default floor variants split into two placement styles per
        // MapTheme::floor_variant_style - see VariantStyle's own doc
        // comment for why one style doesn't fit every kind of variant
        // (confirmed in real play, 2026-09-08: Dungeon's "torchlight
        // glow" patched as a whole region looked like a wall of torches).
        let floor_variant_count = self.theme.floor_variant_count();
        let path_variants = self.theme.path_variants();
        let is_path_variant = |v: u8| {
            path_variants.map_or(false, |(vertical, horizontal)| v == vertical || v == horizontal)
        };
        let patch_variants: Vec<u8> = (1..floor_variant_count)
            .filter(|&v| !is_path_variant(v) && self.theme.floor_variant_style(v) == VariantStyle::Patch)
            .collect();
        let scatter_variants: Vec<u8> = (1..floor_variant_count)
            .filter(|&v| !is_path_variant(v) && self.theme.floor_variant_style(v) == VariantStyle::Scatter)
            .collect();

        // Floor patches: a handful of contiguous, randomly-placed,
        // randomly-sized blobs, each stamped with a single Patch-style
        // floor variant - "blocks of leaves, blocks of moss" rather than
        // noise. The opposite shape from the wall accents above: floors
        // are big open areas where a whole natural-looking REGION of one
        // texture reads right, the way real terrain has patches of
        // different ground cover rather than pixel-scattered variety.
        let floor_tiles: Vec<usize> = self
            .map
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| **t == TileType::Floor)
            .map(|(i, _)| i)
            .collect();
        if floor_tiles.is_empty() {
            return;
        }
        if !patch_variants.is_empty() {
            let patch_count = rng.range(FLOOR_PATCH_COUNT_MIN, FLOOR_PATCH_COUNT_MAX + 1);
            for _ in 0..patch_count {
                let seed_idx = floor_tiles[rng.random_slice_index(&floor_tiles).unwrap()];
                let seed = self.map.index_to_point2d(seed_idx);
                let variant = patch_variants[rng.random_slice_index(&patch_variants).unwrap()];
                let radius = rng.range(FLOOR_PATCH_RADIUS_MIN, FLOOR_PATCH_RADIUS_MAX + 1);
                for y in (seed.y - radius)..=(seed.y + radius) {
                    for x in (seed.x - radius)..=(seed.x + radius) {
                        let pt = Point::new(x, y);
                        if !self.map.in_bounds(pt) {
                            continue;
                        }
                        let dx = (x - seed.x) as f32;
                        let dy = (y - seed.y) as f32;
                        if (dx * dx + dy * dy).sqrt() > radius as f32 {
                            continue;
                        }
                        let idx = map_idx(x, y);
                        if self.map.tiles[idx] == TileType::Floor {
                            self.map.tile_variant[idx] = variant;
                        }
                    }
                }
            }
        }

        // Floor scatter: individual, sparse tiles for Scatter-style
        // variants (a torch, a grate, a bone pile) - same shape as the
        // wall accents above, just floor-side and much rarer
        // (FLOOR_SCATTER_CHANCE_PCT), since these are discrete highlights
        // across a whole floor area rather than a dense boundary accent.
        // Runs after patches, so a scattered accent can still land on top
        // of a patch (a torch on a mossy patch of floor is perfectly
        // plausible).
        if !scatter_variants.is_empty() {
            for &idx in &floor_tiles {
                if rng.range(0, 100) < FLOOR_SCATTER_CHANCE_PCT {
                    self.map.tile_variant[idx] =
                        scatter_variants[rng.random_slice_index(&scatter_variants).unwrap()];
                }
            }
        }

        // Sparse solid-obstacle accent and isolated Wall-to-Water
        // patches both run after the ordinary wall accent above, so an
        // obstacle/patch can win out over (rather than get silently
        // overwritten by) a generic accent roll on the same tile - see
        // each method's own doc comment.
        self.stamp_wall_obstacles(rng);
        self.stamp_wall_water_patches(rng);

        // Theme path, if this theme wants one - runs LAST, after every
        // other overlay above, so nothing else ever paints over it (a
        // patch/scatter accent landing on top of the path would break
        // its visual continuity). See `stamp_theme_path`'s own doc
        // comment for the actual placement algorithm.
        if path_variants.is_some() {
            self.stamp_theme_path(rng);
        }
    }

    /// Sparsely places this theme's `wall_obstacle_variants` (a stump, a
    /// rubble pile, a broken pillar - see that method's own doc comment)
    /// across existing Wall tiles - a no-op if the theme lists none.
    /// Each Wall tile independently rolls `WALL_OBSTACLE_CHANCE_PCT`
    /// (much rarer than the ordinary per-tile wall accent above), then -
    /// only if that roll succeeds - checks every tile within
    /// `WALL_OBSTACLE_MIN_SPACING` for an obstacle already placed there;
    /// if one exists, this candidate is skipped entirely rather than
    /// placing anyway, which is what actually enforces "at most one per
    /// 5x5 area" (2026-09-13) - a plain independent per-tile chance,
    /// however low, would still eventually cluster two by pure chance
    /// somewhere on a large enough map.
    fn stamp_wall_obstacles(&mut self, rng: &mut RandomNumberGenerator) {
        let obstacle_cols = self.theme.wall_obstacle_variants();
        if obstacle_cols.is_empty() {
            return;
        }
        let wall_variant_count = self.theme.wall_variant_count();
        let obstacle_variants: Vec<u8> = obstacle_cols
            .iter()
            .map(|&col| wall_variant_count + col as u8)
            .collect();
        let wall_tiles: Vec<usize> = self
            .map
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| **t == TileType::Wall)
            .map(|(i, _)| i)
            .collect();
        for idx in wall_tiles {
            if rng.range(0, 100) >= WALL_OBSTACLE_CHANCE_PCT {
                continue;
            }
            let pt = self.map.index_to_point2d(idx);
            let mut too_close = false;
            for dy in -WALL_OBSTACLE_MIN_SPACING..=WALL_OBSTACLE_MIN_SPACING {
                for dx in -WALL_OBSTACLE_MIN_SPACING..=WALL_OBSTACLE_MIN_SPACING {
                    let check = Point::new(pt.x + dx, pt.y + dy);
                    if !self.map.in_bounds(check) {
                        continue;
                    }
                    let check_idx = map_idx(check.x, check.y);
                    if self.map.tiles[check_idx] == TileType::Wall
                        && obstacle_variants.contains(&self.map.tile_variant[check_idx])
                    {
                        too_close = true;
                    }
                }
            }
            if !too_close {
                self.map.tile_variant[idx] =
                    obstacle_variants[rng.random_slice_index(&obstacle_variants).unwrap()];
            }
        }
    }

    /// Stamps a small number of small, isolated Wall-to-Water patches -
    /// see `MapTheme::wall_water_patch_variant`'s own doc comment - a
    /// no-op if the theme has none. Deliberately the SAME contiguous-
    /// blob shape `assign_tile_variants`'s floor patches already use,
    /// just against Wall tiles, with a smaller radius and far fewer of
    /// them (`WALL_WATER_PATCH_RADIUS`/`_COUNT_MIN`/`_MAX`) - "a small
    /// patch," not a floor-sized region. Purely cosmetic: Water is
    /// exactly as blocking as the Wall it replaces (see `TileType::
    /// Water`), just not opaque, so this can never open an unintended
    /// shortcut.
    fn stamp_wall_water_patches(&mut self, rng: &mut RandomNumberGenerator) {
        let Some(variant) = self.theme.wall_water_patch_variant() else {
            return;
        };
        let wall_tiles: Vec<usize> = self
            .map
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| **t == TileType::Wall)
            .map(|(i, _)| i)
            .collect();
        if wall_tiles.is_empty() {
            return;
        }
        let patch_count = rng.range(WALL_WATER_PATCH_COUNT_MIN, WALL_WATER_PATCH_COUNT_MAX + 1);
        for _ in 0..patch_count {
            let seed_idx = wall_tiles[rng.random_slice_index(&wall_tiles).unwrap()];
            let seed = self.map.index_to_point2d(seed_idx);
            let radius = WALL_WATER_PATCH_RADIUS;
            for y in (seed.y - radius)..=(seed.y + radius) {
                for x in (seed.x - radius)..=(seed.x + radius) {
                    let pt = Point::new(x, y);
                    if !self.map.in_bounds(pt) {
                        continue;
                    }
                    let dx = (x - seed.x) as f32;
                    let dy = (y - seed.y) as f32;
                    if (dx * dx + dy * dy).sqrt() > radius as f32 {
                        continue;
                    }
                    let idx = map_idx(x, y);
                    if self.map.tiles[idx] == TileType::Wall {
                        self.map.tiles[idx] = TileType::Water;
                        self.map.tile_variant[idx] = variant;
                    }
                }
            }
        }
    }

    /// Stamps ONE real connected line of this theme's `path_variants`
    /// between `player_start` and `amulet_start`, across Floor tiles
    /// only - a no-op if the theme has none (`path_variants() ==
    /// None`). Deliberately walks the real shortest route
    /// (`Map::bfs_distance_field`, greedily stepping to whichever
    /// neighbor has the smallest distance-to-target) rather than a
    /// straight line or an undirected random walk - either of those can
    /// cross a Wall tile the map's own layout never actually connects
    /// through, which would leave the path with gaps wherever it did.
    /// Ties (more than one neighbor sharing the smallest distance, which
    /// happens constantly in an open room) are broken at random, which
    /// is what keeps the path from reading as a mechanically straight
    /// line everywhere except forced corridors - a corridor only has
    /// one route anyway, so it stays straight there regardless.
    ///
    /// Each tile gets whichever of the two variants actually matches how
    /// it connects to its own neighbors (`horizontal` if it has a path
    /// neighbor to its left/right but not above/below, `vertical`
    /// otherwise, including a corner that connects both ways - no single
    /// orientation is more correct there, so it just keeps the texture's
    /// own default look) - decided HERE, once, at generation time, and
    /// baked directly into `tile_variant` as two genuinely different
    /// pre-rotated textures. Not a runtime rotation (2026-09-13,
    /// replacing several rounds of exactly that): bracket-terminal's
    /// NEAREST-filtered, zero-padding atlas sampling made a live
    /// `set_fancy` rotation produce real, unfixable seams specifically
    /// on this project's rendering setup - see docs/journal.md's own
    /// blow-by-blow. A second real atlas cell, pre-rotated once as a
    /// normal image edit and stored in `resources/map_tiles.png`
    /// directly (Forest's Path Fork cell, repurposed - nothing used it
    /// for real branching anyway), sidesteps that whole class of bug: a
    /// horizontal path tile is just an ordinary static glyph like any
    /// other tile, not a special render-time case at all.
    fn stamp_theme_path(&mut self, rng: &mut RandomNumberGenerator) {
        let Some((vertical_variant, horizontal_variant)) = self.theme.path_variants() else {
            return;
        };
        let start = self.player_start;
        let end = self.amulet_start;
        let field = self.map.bfs_distance_field(start);

        let mut path_tiles: Vec<usize> = Vec::new();
        let mut pos = end;
        loop {
            let idx = map_idx(pos.x, pos.y);
            if self.map.tiles[idx] == TileType::Floor {
                path_tiles.push(idx);
            }
            if pos == start {
                break;
            }
            let current_dist = field[idx];
            let mut best_candidates: Vec<Point> = Vec::new();
            let mut best_dist = current_dist;
            for delta in [
                Point::new(0, -1),
                Point::new(0, 1),
                Point::new(-1, 0),
                Point::new(1, 0),
            ] {
                let neighbor = pos + delta;
                if !self.map.in_bounds(neighbor) {
                    continue;
                }
                let neighbor_dist = field[map_idx(neighbor.x, neighbor.y)];
                if neighbor_dist < best_dist {
                    best_dist = neighbor_dist;
                    best_candidates.clear();
                    best_candidates.push(neighbor);
                } else if neighbor_dist == best_dist && neighbor_dist < current_dist {
                    best_candidates.push(neighbor);
                }
            }
            if best_candidates.is_empty() {
                // Shouldn't happen - start/end are always mutually
                // reachable Floor tiles - but never loop forever if it
                // somehow does.
                break;
            }
            pos = best_candidates[rng.random_slice_index(&best_candidates).unwrap()];
        }

        // Every tile is provisionally vertical (the walked-line default,
        // and the texture's own base orientation) until this second pass
        // reclassifies the ones that actually read as horizontal - has
        // to run as its own pass, after every path tile is already
        // known, since a tile's own classification depends on whether
        // ITS neighbors are ALSO path tiles.
        for &idx in &path_tiles {
            self.map.tile_variant[idx] = vertical_variant;
        }
        for &idx in &path_tiles {
            let pt = self.map.index_to_point2d(idx);
            let is_path = |p: Point| {
                self.map.in_bounds(p) && path_tiles.contains(&map_idx(p.x, p.y))
            };
            let horizontal = is_path(pt + Point::new(-1, 0)) || is_path(pt + Point::new(1, 0));
            let vertical = is_path(pt + Point::new(0, -1)) || is_path(pt + Point::new(0, 1));
            if horizontal && !vertical {
                self.map.tile_variant[idx] = horizontal_variant;
            }
        }
    }

    fn fill(&mut self, tile: TileType) {
        self.map.tiles.iter_mut().for_each(|t| *t = tile);
    }

    /// Builds the Battle Arena's shop map: a small hand-built walled room
    /// (not one of the three random MapArchitect layouts - a shop is a
    /// fixed, deliberate layout, not something worth randomizing) with an
    /// items row along the top, the player starting in the middle, and a
    /// stairs point (returned via the usual `amulet_start` field, same
    /// convention every other level already uses for "the special point
    /// that becomes a TileType::Exit tile") along the bottom.
    ///
    /// Returns the MapBuilder, exactly `item_count` floor points for
    /// items (left to right, centered on the counter), one point for the
    /// shopkeeper NPC, and a reveal rectangle (x, y, width, height) -
    /// State::start_arena only marks tiles inside that rectangle as
    /// revealed/visible, leaving the rest of the underlying 80x50 grid
    /// (a fixed size every Map uses, not something one MapBuilder can
    /// resize on its own) completely unrevealed. Since nothing renders
    /// for a tile that's neither visible nor ever revealed, this is what
    /// actually makes the map itself read as small - a camera trick
    /// alone couldn't do this, because the camera has no way to tell
    /// "the edge of this small room" apart from "more world offscreen."
    ///
    /// The shop's own interior is a fixed 10 (wide) x 6 (tall) - not
    /// scaled by item count - split top to bottom into: row 0 the
    /// shopkeeper (decorative, one tile, centered) - genuinely behind
    /// the counter, safe to see because start_arena's reveal covers this
    /// whole area rather than relying on real shadowcasting, which a
    /// Counter tile would otherwise block sight past just like a Wall
    /// does. Row 1 the counter itself (TileType::Counter - impassable,
    /// items sit on it, centered within the 10-wide row). Rows 2-5, four
    /// rows of open walkable floor - the player starts at the top of
    /// this (row 2, directly below the row-1 item at the same column) and
    /// the stairs sit at the bottom (row 5).
    pub fn new_arena_shop(
        rng: &mut RandomNumberGenerator,
        item_count: usize,
    ) -> (Self, Vec<Point>, Point, i32, i32, i32, i32) {
        const INTERIOR_W: i32 = 10;
        const INTERIOR_H: i32 = 6;
        const ROOM_W: i32 = INTERIOR_W + 2;
        const ROOM_H: i32 = INTERIOR_H + 2;

        // The reveal rectangle - "the overall map" as the player will
        // actually experience it, centered on the same point as the
        // room itself. ~40x20 comfortably frames the small 12x8 room
        // with a visible ring of backdrop, without being anywhere close
        // to the full 80x50 grid every dungeon floor uses.
        const REVEAL_W: i32 = 40;
        const REVEAL_H: i32 = 20;

        let room_x0 = (SCREEN_WIDTH - ROOM_W) / 2;
        let room_y0 = (SCREEN_HEIGHT - ROOM_H) / 2;
        let reveal_x0 = (SCREEN_WIDTH - REVEAL_W) / 2;
        let reveal_y0 = (SCREEN_HEIGHT - REVEAL_H) / 2;

        let mut mb = Self {
            map: Map::new(),
            rooms: Vec::new(),
            monster_spawns: Vec::new(),
            player_start: Point::zero(),
            amulet_start: Point::zero(),
            // Forest, not dungeon brick, for the shop's backdrop -
            // walking bark/mossy fill reads as deliberate scenery
            // framing a small clearing, since the reveal boundary above
            // means the player will actually see the edge of it rather
            // than endless brick.
            theme: ForestTheme::new(),
            prefab_enemy_spawns: Vec::new(),
            prefab_weapon_spawn: None,
            prefab_chest_spawn: None,
            prefab_chest_guard_spawns: Vec::new(),
        };
        mb.fill(TileType::Wall);
        for y in (room_y0 + 1)..(room_y0 + ROOM_H - 1) {
            for x in (room_x0 + 1)..(room_x0 + ROOM_W - 1) {
                let idx = map_idx(x, y);
                mb.map.tiles[idx] = TileType::Floor;
            }
        }

        let interior_x = |col: i32| room_x0 + 1 + col;
        let interior_y = |row: i32| room_y0 + 1 + row;

        // The counter row - TileType::Counter, not Floor: impassable
        // (can_enter_tile only allows Floor/Exit) so the player can
        // never step onto it, and rendered as a distinct warm bar (see
        // components::tile_render_at) rather than plain floor or wall
        // brick.
        for col in 0..INTERIOR_W {
            let idx = map_idx(interior_x(col), interior_y(1));
            mb.map.tiles[idx] = TileType::Counter;
        }

        let shopkeeper_point = Point::new(interior_x(INTERIOR_W / 2), interior_y(0));
        mb.player_start = Point::new(interior_x(INTERIOR_W / 2), interior_y(2));
        mb.amulet_start = Point::new(interior_x(INTERIOR_W / 2), interior_y(5));

        // Items are centered within the fixed 10-wide counter rather
        // than always starting at column 0 - a shop selling only 3-4
        // things sits in the middle of the counter instead of bunched
        // against the left wall. item_count is expected to be at most
        // 7 (1 weapon + 1 potion stack + up to 5 ability stacks), which
        // always fits inside 10 with room to spare.
        let items_start_col = ((INTERIOR_W - item_count as i32) / 2).max(0);
        let item_points: Vec<Point> = (0..item_count as i32)
            .map(|i| Point::new(interior_x(items_start_col + i), interior_y(1)))
            .collect();

        mb.assign_tile_variants(rng);

        (
            mb,
            item_points,
            shopkeeper_point,
            reveal_x0,
            reveal_y0,
            REVEAL_W,
            REVEAL_H,
        )
    }

    /// Builds the Battle Arena's combat map for one wave: a single open
    /// circular clearing (no interior walls to block sight or movement -
    /// the player is meant to see and reach every enemy) with the player
    /// starting at the center and `enemy_count` spawn points scattered
    /// around a thin ring near the circle's edge, so enemies visibly
    /// appear at the edge of the clearing rather than already being on
    /// top of the player. Also returns a boss spawn point (the reachable
    /// tile farthest from the player's start, the same
    /// "find_most_distant" convention every dungeon floor already uses
    /// for its own boss/exit point) - this is only actually used once
    /// wave 3 clears (see State::arena_spawn_boss_on_current_map), since
    /// the boss appears on this same map rather than a freshly built one.
    ///
    /// Returns the MapBuilder, up to `enemy_count` spawn points (fewer
    /// if the edge ring somehow has less room than that - it doesn't, at
    /// this circle's size, for any of the 5/5/3 wave counts), the boss
    /// spawn point, and a reveal rectangle (x, y, width, height) - see
    /// new_arena_shop's doc comment for why a reveal rectangle rather
    /// than the whole 80x50 map is what actually keeps this map's
    /// footprint small on screen.
    pub fn new_arena_wave(
        rng: &mut RandomNumberGenerator,
        enemy_count: usize,
    ) -> (Self, Vec<Point>, Point, i32, i32, i32, i32) {
        const RADIUS: i32 = 10;
        const REVEAL_W: i32 = 34;
        const REVEAL_H: i32 = 26;

        let center = Point::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2);
        let reveal_x0 = (SCREEN_WIDTH - REVEAL_W) / 2;
        let reveal_y0 = (SCREEN_HEIGHT - REVEAL_H) / 2;

        let mut mb = Self {
            map: Map::new(),
            rooms: Vec::new(),
            monster_spawns: Vec::new(),
            player_start: Point::zero(),
            amulet_start: Point::zero(),
            theme: ForestTheme::new(),
            prefab_enemy_spawns: Vec::new(),
            prefab_weapon_spawn: None,
            prefab_chest_spawn: None,
            prefab_chest_guard_spawns: Vec::new(),
        };
        mb.fill(TileType::Wall);
        // A circular clearing, not a rectangular room - there's no
        // separate wall ring to draw here: the surrounding Wall/forest
        // fill (already everywhere from the fill above) forms the
        // boundary on its own, wherever a tile falls outside RADIUS of
        // center.
        for y in (center.y - RADIUS - 1)..=(center.y + RADIUS + 1) {
            for x in (center.x - RADIUS - 1)..=(center.x + RADIUS + 1) {
                let dx = (x - center.x) as f32;
                let dy = (y - center.y) as f32;
                if (dx * dx + dy * dy).sqrt() <= RADIUS as f32 {
                    mb.map.tiles[map_idx(x, y)] = TileType::Floor;
                }
            }
        }

        mb.player_start = center;

        // A thin ring of floor tiles near the circle's own edge - the
        // "perimeter" enemies spawn on, so they visibly appear at the
        // edge of the clearing rather than already being on top of the
        // player. Same distance check as the fill above, just looking
        // for tiles close to RADIUS rather than anywhere inside it.
        let mut perimeter: Vec<Point> = Vec::new();
        for y in (center.y - RADIUS)..=(center.y + RADIUS) {
            for x in (center.x - RADIUS)..=(center.x + RADIUS) {
                let dx = (x - center.x) as f32;
                let dy = (y - center.y) as f32;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist <= RADIUS as f32 && dist >= RADIUS as f32 - 1.5 {
                    perimeter.push(Point::new(x, y));
                }
            }
        }

        // Distinct random points, same pick-and-remove pattern
        // spawn_monsters (below) already uses - capped at however many
        // perimeter tiles actually exist, though at this circle's size
        // (~60 edge tiles) that cap never actually bites for any of the
        // 5/5/3 wave counts.
        let mut candidates = perimeter;
        let spawn_count = enemy_count.min(candidates.len());
        let mut enemy_spawns = Vec::with_capacity(spawn_count);
        for _ in 0..spawn_count {
            let idx = rng.random_slice_index(&candidates).unwrap();
            enemy_spawns.push(candidates[idx]);
            candidates.remove(idx);
        }

        let boss_spawn = mb.find_most_distant();
        mb.assign_tile_variants(rng);

        (
            mb,
            enemy_spawns,
            boss_spawn,
            reveal_x0,
            reveal_y0,
            REVEAL_W,
            REVEAL_H,
        )
    }

    fn find_most_distant(&self) -> Point {
        let dijkstra_map = DijkstraMap::new(
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            &vec![self.map.point2d_to_index(self.player_start)],
            &self.map,
            1024.0,
        );

        const UNREACHABLE: &f32 = &f32::MAX;
        self.map.index_to_point2d(
            dijkstra_map
                .map
                .iter()
                .enumerate()
                .filter(|(_, dist)| *dist < UNREACHABLE)
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .unwrap()
                .0,
        )
    }

    fn build_random_rooms(&mut self, rng: &mut RandomNumberGenerator) {
        while self.rooms.len() < NUM_ROOMS {
            let room = Rect::with_size(
                rng.range(1, SCREEN_WIDTH - 10),
                rng.range(1, SCREEN_HEIGHT - 10),
                rng.range(2, 10),
                rng.range(2, 10),
            );
            let mut overlap = false;
            for r in self.rooms.iter() {
                if r.intersect(&room) {
                    overlap = true;
                }
            }
            if !overlap {
                room.for_each(|p| {
                    if p.x > 0 && p.x < SCREEN_WIDTH && p.y > 0 && p.y < SCREEN_HEIGHT {
                        let idx = map_idx(p.x, p.y);
                        self.map.tiles[idx] = TileType::Floor;
                    }
                });

                self.rooms.push(room)
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        use std::cmp::{max, min};
        for x in min(x1, x2)..=max(x1, x2) {
            if let Some(idx) = self.map.try_idx(Point::new(x, y)) {
                self.map.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        use std::cmp::{max, min};
        for y in min(y1, y2)..=max(y1, y2) {
            if let Some(idx) = self.map.try_idx(Point::new(x, y)) {
                self.map.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn build_corridors(&mut self, rng: &mut RandomNumberGenerator) {
        let mut rooms = self.rooms.clone();
        rooms.sort_by(|a, b| a.center().x.cmp(&b.center().x));

        for (i, room) in rooms.iter().enumerate().skip(1) {
            let prev = rooms[i - 1].center();
            let new = room.center();

            if rng.range(0, 2) == 1 {
                self.apply_horizontal_tunnel(prev.x, new.x, prev.y);
                self.apply_vertical_tunnel(prev.y, new.y, new.x);
            } else {
                self.apply_vertical_tunnel(prev.y, new.y, prev.x);
                self.apply_horizontal_tunnel(prev.x, new.x, new.y);
            }
        }
    }

    fn spawn_monsters(&self, start: &Point, rng: &mut RandomNumberGenerator) -> Vec<Point> {
        const NUM_MONSTERS: usize = 50;
        let mut spawnable_tiles: Vec<Point> = self
            .map
            .tiles
            .iter()
            .enumerate()
            .filter(|(idx, t)| {
                **t == TileType::Floor
                    && DistanceAlg::Pythagoras.distance2d(*start, self.map.index_to_point2d(*idx))
                        > 10.0
            })
            .map(|(idx, _)| self.map.index_to_point2d(idx))
            .collect();

        let mut spawns = Vec::new();
        // Capped at however many candidate tiles actually exist, rather
        // than always looping a fixed NUM_MONSTERS times. A small or
        // tightly-packed cave can easily have fewer than 50 tiles more
        // than 10 units from the player's start - previously this kept
        // looping anyway, and random_slice_index on an empty slice
        // returns None, which the unconditional .unwrap() below turned
        // into an outright panic once spawnable_tiles ran out. Spawning
        // fewer monsters on a small map is the correct behavior, not a
        // bug to route around - CellularAutomataArchitect's connectivity
        // fix (see automata.rs's cull_unreachable_areas) made this
        // reachable a lot more often, by shrinking the reachable floor
        // area on some maps, but the underlying fragility was already
        // here regardless of map size.
        let spawn_count = NUM_MONSTERS.min(spawnable_tiles.len());
        for _ in 0..spawn_count {
            let target_index = rng.random_slice_index(&spawnable_tiles).unwrap();
            spawns.push(spawnable_tiles[target_index].clone());
            spawnable_tiles.remove(target_index);
        }
        spawns
    }
}
