use crate::prelude::*;
use std::collections::HashMap;
use std::fs;

// --- Rebindable actions -----------------------------------------------------
//
// Scoped to movement for now - the classic "arrows vs WASD" ask - rather
// than every key in the game. Escape/pause and the number-key item/
// technique selectors stay fixed: those are positional (1-9, or a
// hardware-convention Escape) rather than something players typically
// rebind, and leaving Escape permanently wired to pause means a bad
// rebind can never lock the pause menu away entirely. Adding a new
// rebindable action later is one more Action variant, one more line in
// Action::ALL/label, and swapping its matching hardcoded VirtualKeyCode
// check (see systems/player_input.rs) for a Keymap lookup - no other
// structural change.

/// A semantic action a key can trigger, independent of which physical key
/// is currently bound to it - see Keymap.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
}

impl Action {
    /// Every rebindable action, in the order the Options screen lists
    /// them - see screens/options.rs.
    pub const ALL: [Action; 4] = [
        Action::MoveUp,
        Action::MoveDown,
        Action::MoveLeft,
        Action::MoveRight,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Action::MoveUp => "Move Up",
            Action::MoveDown => "Move Down",
            Action::MoveLeft => "Move Left",
            Action::MoveRight => "Move Right",
        }
    }
}

// --- Key <-> name round-trip -------------------------------------------------
//
// Rebinding is deliberately limited to the arrow keys and A-Z - between
// them that covers every realistic movement scheme (arrows, WASD, IJKL,
// HJKL, ZQSD...) without needing a match arm for all ~150 VirtualKeyCode
// variants winit defines. The Options screen's capture step (see
// screens/options.rs) only ever calls rebind() with a key this list
// recognizes in the first place - key_from_name existing separately is
// what lets a saved keymap.ron round-trip through plain text instead of
// needing bracket-lib's own (feature-gated, not currently enabled)
// VirtualKeyCode serde support.
const REBINDABLE_KEYS: [VirtualKeyCode; 29] = [
    VirtualKeyCode::Up,
    VirtualKeyCode::Down,
    VirtualKeyCode::Left,
    VirtualKeyCode::Right,
    VirtualKeyCode::A,
    VirtualKeyCode::B,
    VirtualKeyCode::C,
    VirtualKeyCode::D,
    VirtualKeyCode::E,
    VirtualKeyCode::F,
    VirtualKeyCode::G,
    VirtualKeyCode::H,
    VirtualKeyCode::I,
    VirtualKeyCode::J,
    VirtualKeyCode::K,
    VirtualKeyCode::L,
    // M deliberately excluded - it's now hardcoded to open the Item Menu
    // (see systems/player_input.rs), the same reason Escape/Space are
    // never in this list either. Without this exclusion, a player could
    // rebind a movement Action to M and then be unable to ever trigger
    // it, since player_input checks for the Item Menu key first.
    VirtualKeyCode::N,
    VirtualKeyCode::O,
    VirtualKeyCode::P,
    VirtualKeyCode::Q,
    VirtualKeyCode::R,
    VirtualKeyCode::S,
    VirtualKeyCode::T,
    VirtualKeyCode::U,
    VirtualKeyCode::V,
    VirtualKeyCode::W,
    VirtualKeyCode::X,
    VirtualKeyCode::Y,
    VirtualKeyCode::Z,
];

/// True if `key` is one this game will accept as a new binding - see
/// REBINDABLE_KEYS. Used by the Options screen's capture step to ignore
/// (rather than crash or silently misbehave on) an out-of-scope keypress
/// like F1 or Numpad0 while awaiting a rebind.
pub fn is_rebindable_key(key: VirtualKeyCode) -> bool {
    REBINDABLE_KEYS.contains(&key)
}

/// A short display name for `key` - just its Debug representation, which
/// for every key in REBINDABLE_KEYS is already the obvious single-word
/// name ("Up", "W", "Q"...). Used both for on-screen display and as the
/// on-disk representation in keymap.ron.
fn key_name(key: VirtualKeyCode) -> String {
    format!("{:?}", key)
}

/// The reverse of key_name - only ever matches something in
/// REBINDABLE_KEYS, so an old/hand-edited keymap.ron naming a key this
/// game doesn't offer for rebinding just falls through to None, which
/// Keymap::load treats as "keep the default" for that entry.
fn key_from_name(name: &str) -> Option<VirtualKeyCode> {
    REBINDABLE_KEYS
        .iter()
        .copied()
        .find(|k| key_name(*k) == name)
}

// --- Keymap ------------------------------------------------------------

/// Which physical key currently triggers each rebindable Action.
#[derive(Clone, Debug, PartialEq)]
pub struct Keymap {
    bindings: HashMap<Action, VirtualKeyCode>,
}

/// Where rebound keys are persisted - see Keymap::load/save. Lives under
/// saves/ (already gitignored) alongside the run-history stats file, even
/// though this is really "options" rather than "save game" - both are
/// the same kind of small local config the player's install keeps
/// between sessions.
const KEYMAP_PATH: &str = "saves/keymap.ron";

impl Keymap {
    /// Arrow keys for movement - matches the game's behavior from before
    /// rebinding existed, so nobody's controls change unless they
    /// actually open Options.
    pub fn default_bindings() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert(Action::MoveUp, VirtualKeyCode::Up);
        bindings.insert(Action::MoveDown, VirtualKeyCode::Down);
        bindings.insert(Action::MoveLeft, VirtualKeyCode::Left);
        bindings.insert(Action::MoveRight, VirtualKeyCode::Right);
        Self { bindings }
    }

    /// Loads saved bindings from KEYMAP_PATH, falling back to
    /// default_bindings() for any action whose entry is missing, the
    /// file doesn't exist at all, or the file fails to parse as RON.
    /// Never panics on a missing/corrupt save file - worst case, the
    /// player just sees default controls again.
    pub fn load() -> Self {
        let mut keymap = Self::default_bindings();
        if let Ok(text) = fs::read_to_string(KEYMAP_PATH) {
            if let Ok(saved) = ron::de::from_str::<HashMap<String, String>>(&text) {
                for action in Action::ALL {
                    if let Some(key) = saved.get(action.label()).and_then(|n| key_from_name(n)) {
                        keymap.bindings.insert(action, key);
                    }
                }
            }
        }
        keymap
    }

    /// Writes the current bindings to KEYMAP_PATH as plain action-name ->
    /// key-name text, creating the saves/ directory first if needed.
    /// Silently does nothing on a write failure (e.g. a read-only
    /// filesystem) rather than panicking - losing a rebind on save is a
    /// minor inconvenience, not worth crashing the game over.
    pub fn save(&self) {
        let _ = fs::create_dir_all("saves");
        let named: HashMap<String, String> = Action::ALL
            .iter()
            .map(|a| (a.label().to_string(), key_name(self.key_for(*a))))
            .collect();
        if let Ok(text) = ron::ser::to_string_pretty(&named, ron::ser::PrettyConfig::default()) {
            let _ = fs::write(KEYMAP_PATH, text);
        }
    }

    /// The action currently bound to `key`, if any - used by
    /// systems/player_input.rs to turn a raw keypress into a semantic
    /// Action instead of matching a hardcoded VirtualKeyCode directly.
    pub fn action_for_key(&self, key: VirtualKeyCode) -> Option<Action> {
        self.bindings
            .iter()
            .find(|(_, &bound_key)| bound_key == key)
            .map(|(&action, _)| action)
    }

    /// The key currently bound to `action`. Falls back to that action's
    /// DEFAULT key if somehow missing from `bindings` (should never
    /// happen after the swap-based rebind below, but this used to
    /// index the HashMap directly with `[]`, which panicked outright the
    /// moment an entry went missing - e.g. the eviction bug rebind() had
    /// before it was rewritten to swap instead of evict. Falling back
    /// here instead of indexing keeps this function safe even if some
    /// future change reintroduces a gap, rather than trusting every
    /// caller of rebind() to never make one.
    pub fn key_for(&self, action: Action) -> VirtualKeyCode {
        self.bindings
            .get(&action)
            .copied()
            .unwrap_or_else(|| Self::default_bindings().bindings[&action])
    }

    /// Rebinds `action` to `key`. If `key` was already bound to a
    /// DIFFERENT action, that action is given `action`'s OLD key instead
    /// - a true swap, not an eviction. Every Action always keeps exactly
    /// one binding this way; nothing ever ends up with zero. (An earlier
    /// version of this function removed the other action's binding
    /// outright via `retain`, which left it with no entry in `bindings`
    /// at all - the very next `key_for`/`save` call for that action then
    /// panicked, since both used to index the HashMap directly. That's
    /// the crash rebinding onto an in-use key used to cause.) Does NOT
    /// persist to disk on its own - callers save() explicitly once
    /// they're done rebinding.
    pub fn rebind(&mut self, action: Action, key: VirtualKeyCode) {
        let previous_key = self.key_for(action);
        if let Some(other_action) = self.action_for_key(key) {
            if other_action != action {
                self.bindings.insert(other_action, previous_key);
            }
        }
        self.bindings.insert(action, key);
    }

    /// Restores every action to its default key (see default_bindings).
    /// Does NOT persist to disk on its own - same as rebind, callers
    /// save() explicitly afterward.
    pub fn reset_to_defaults(&mut self) {
        *self = Self::default_bindings();
    }
}
