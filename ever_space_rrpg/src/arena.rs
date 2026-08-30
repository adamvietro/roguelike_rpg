/// Which top-level game mode the player picked at the new AdventureSelect
/// screen (see screens/title.rs). Kept as a plain `State` field (like
/// `options_awaiting`/`stats_selected_class`), not a resource - it's
/// screen-navigation state read only by class_select to decide which
/// start function to call (`State::start_game` vs `State::start_arena`),
/// not something any gameplay system needs to see.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdventureMode {
    DungeonCrawl,
    BattleArena,
}

/// Tracks progress through a Battle Arena run. Inserted as a resource
/// (`Some(ArenaRun{..})`) only when a Battle Arena run is active -
/// `start_game` (normal dungeon crawl) always inserts `None::<ArenaRun>`
/// instead. Other systems tell an arena run apart from an ordinary
/// dungeon crawl purely by checking this resource's presence, e.g.
/// end_turn's Exit-tile handling below.
///
/// This first slice only carries `level` (which shop/enemy tier the
/// player is on) - just enough to build the first testable piece, the
/// shop itself. Wave-count/boss-defeated tracking will be added here once
/// wave orchestration is built next.
#[derive(Clone, Copy, Debug)]
pub struct ArenaRun {
    /// Which of the 3 arena levels this is, 1..=3. Maps onto the
    /// existing dungeon template levels 0..=2 (see `template_level`) -
    /// arena level 1 uses the same Goblin/Goblin Chieftain pool as
    /// dungeon floor 0, arena level 2 uses dungeon floor 1's Orc/Orc
    /// Warlord pool, and so on. While the player is in the STARTING shop
    /// (before level 1's waves begin), this is already `1` - the shop is
    /// "preparing you for level 1", not a separate level 0.
    pub level: u8,
}

impl ArenaRun {
    /// Converts this run's 1..=3 arena level into the 0..=2 index
    /// `template.ron`'s `levels:` sets and spawn_level/spawn_boss expect.
    pub fn template_level(&self) -> usize {
        (self.level - 1) as usize
    }
}

/// Marker resource (`Option<ShoppingActive>`, same "None means not
/// active" convention as Option<Battle>/Option<ArenaRun>) - present only
/// while the player is browsing an arena shop. Its purpose is narrow and
/// deliberate: while Some, systems/movement.rs's normal walk-onto-it
/// auto-pickup is suppressed, and systems/player_input.rs's new buy key
/// (Enter) is enabled instead - so shop items are a deliberate choice,
/// not something the player sweeps up by walking past. Scoped
/// separately from ArenaRun itself (rather than just checking
/// ArenaRun.is_some()) because once wave combat exists, an ArenaRun will
/// still be active during a wave fight, where normal auto-pickup of
/// battle loot should keep working exactly like it does in a dungeon
/// crawl - only the shop screens want this suppressed.
#[derive(Clone, Copy, Debug)]
pub struct ShoppingActive;

/// How many of this item remain on the shop counter - the counter entity
/// itself is a lightweight display/bookkeeping marker (Point + Render +
/// Name + this), NOT a real usable Item (no Effect/Technique/Weapon
/// components) - buying one calls spawner::spawn_named_item_via_commands
/// to grant the player a real, fully-built copy, then decrements this.
/// Reaching 0 removes the counter entity entirely rather than leaving a
/// "0 remaining" marker behind - see player_input.rs's buy_nearby_item.
#[derive(Clone, Copy, Debug)]
pub struct ShopStock(pub i32);
