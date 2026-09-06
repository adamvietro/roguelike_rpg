# Ever Space RRPG — Ideas & Backlog

Working list of what's still ahead, and a running record of what's already
shipped. Pull individual "Working" items into a real session when ready to
build them — nothing there is scoped or scheduled just by being listed.

---

# Working

## Near-term priorities

Roughly in the order they've come up:

1. ~~Player-status frame / Battle Arena shop overlap~~ — **done.** Rather
   than moving either element, replaced the old fixed top-left item LIST
   entirely with a single tooltip for whichever item is currently
   adjacent to the player (`components::shop_item_near`, the exact same
   lookup `buy_nearby_item` uses to decide what Enter actually buys, so
   the tooltip can never show something different from what you'd
   purchase). Anchored to the player's own on-screen position (converted
   map coordinate -> HUD_CONSOLE cell via `mouse_to_hud`, same conversion
   `tooltips.rs` already used for the mouse-hover tile tooltip) rather
   than a fixed screen spot, so it travels with the player along the
   counter instead of colliding with the frame above it.
2. **Overall balance pass** — starting gold, starting items, and player/
   enemy stats generally. Not scoped - a real look at the numbers across
   both Dungeon Crawl and Battle Arena, not a specific bug.
3. ~~An Item Bar, mirroring the Ability Bar/Battle Bar model~~ — **done.**
   A third icon bar, BLUE box, sitting immediately left of the Ability
   Bar (Item | gap | Ability | gap | Battle, all one row). Sits ALONGSIDE
   the Item Menu rather than replacing it — the Item Menu (`M`) is still
   there for browsing/descriptions, the bar is the fast path. Click
   (left mouse button) to use directly, rather than a new number-key
   range — 1-9/0 stay exclusive to class abilities, no new keybinds
   needed. This was also the first click-to-activate interaction
   anywhere in the game (see components::MouseLeftJustPressed) — a real
   physical-click edge detector, not bracket-lib's own `left_click`
   flag, which fires twice per click (see that struct's doc comment).
   Roster is `spawner::universal_item_names()` (currently just Healing
   Potion + Dungeon Map, will grow as more universal items are added),
   shown greyed-out when unowned like the other two bars.
4. ~~Player-status frame (class-portrait icon + health bar, top-left)~~ —
   **done.** Replaces the old plain full-width health bar. Icon reuses
   the player's existing `Render` component (same sprite as the dungeon-
   map glyph/battle portrait) on the Ability/Battle/Item Bar's console at
   its existing 40x40px size - no new console or art. Bar stays on
   HUD_CONSOLE's fine grid rather than the coarse icon console
   (`bar_horizontal` only fills whole cells, so fewer/bigger cells reads
   chunkier), one row tall specifically because text can't be centered
   between 2+ rows on this console API. See item 1 above for the one
   known collision this introduced (Battle Arena shop's item list).
5. ~~Pause screen: bigger text, cursor menu, rotating hints~~ — **done.**
   Header on the same big-font console Options/History already use;
   body on the medium HUD font instead of the old tiny 8px console.
   Resume/Options/Quit is now a real arrow-key + Enter menu
   (`screens/pause.rs`'s `PAUSE_MENU_ITEMS`), matching every other menu
   screen's convention - Escape/O/Q still work directly too, purely
   additive like Adventure Select/Class Select's own cursor rollout. The
   general how-to-play hint that used to sit permanently on the dungeon
   HUD moved here as a rotating "Hints" box (lower third of the screen,
   3 lines, cycling every 4s) instead - see `PAUSE_HINTS` for the list,
   add/remove freely.
6. ~~Mouse-click-to-use on the Ability Bar~~ — **done** (`player_input.rs`'s
   `use_ability_bar_click`). Turned out to need no new design at all -
   click hit-tests the bar's own column range, then calls the exact same
   `use_ability` the number keys already use, so clicking slot `n` and
   pressing the key for slot `n` queue the identical `ActivateItem`.
   Coexists with the hotkeys rather than replacing them, same as the Item
   Bar's click sits alongside the Item Menu. **Battle Bar still open** -
   it's reference-only outside a fight by design, so click support there
   still needs its own discussion about what it should even do; hovering
   already shows a description on it.
   - This closed a real bug, not just a feature gap: one of the Pause
     screen's rotating hints already claimed "Dungeon Abilities can be
     clicked, or used with their hotkey" - true now, but it wasn't when
     that hint was first added (copied verbatim from a hint list written
     assuming this already worked). Worth double-checking any hint text
     against actual behavior before trusting it as documentation.
7. **Player buffs shown on the portrait** — the top-left class-portrait
   icon (see item 4 above) should reflect an active buff while exploring
   the dungeon, so the player has an at-a-glance signal without opening a
   menu. Not scoped - needs a look at how `Buff`/`BuffKind` (currently a
   battle-only concept) would map onto something visible outside combat,
   and what happens with more than one active buff at once.
8. **Mouse-targeted ranged AOE outside battle** ("rain of fire" for
   ranged classes) — not started.
9. **Standing "fix issues with the battle system" bucket** — not a fixed
   list, just wherever ATB/multi-enemy/the cursor system turns up real
   bugs as they get more play.
10. **A shop for Potions/Maps in Dungeon Crawl mode** — pull them out of
    floor loot entirely, likely reusing a good chunk of the Battle Arena
    shop's pricing/stock/purchase code. More natural now that a real
    "universal item" pool already exists behind the Item Menu.
11. **Idle walk-in-place animation art** — the cycling infrastructure
    (`IdleAnimation` component) is built and genuinely cycling; every
    frame just points at the same placeholder glyph. Needs real distinct
    per-frame art, and maybe a move to a sprite sheet per class instead of
    cramming more cells into the one shared `dungeonfont.png`.
12. **Music & sound effects** — no crate picked yet (`rodio` is the
    leading candidate, since bracket-lib has no built-in audio support).
13. **Stack-count badge on the Ability Bar/Battle Bar icons** — e.g. a
    small "x2" for two Freeze Traps, simplified away to actually finish
    the bars in one session. Hovering already reveals ownership, just not
    the exact count at a glance.
14. **Visual confirmation pass on the Ability Bar/Battle Bar** — the box-
    overlap bug is fixed and tested, but the exact label/tooltip
    positioning was computed via pixel-ratio math without a full round of
    "here's a screenshot, nudge this" the way most of this project's
    visual work gets. Worth a dedicated look with a few different ability
    counts.
15. **README.md needs a manual pass** — controls changed materially over
    the last couple sessions (M for the Item Menu, number keys now mean
    abilities not potions/maps, Item Bar/Ability Bar clicking, Pause
    screen now a cursor menu) and the README wasn't touched.
16. Cleanup: `arena_advance_to_next_shop` duplicates a chunk of
    `start_arena`'s shop-building code — not urgent, just flagged.
17. Minor/cosmetic: `tooltips.rs` still reads the old integer camera
    offset during a glide, instead of the smooth fractional one.

## Content / world

- **Dungeon Shop** — replace most dungeon floor items with a shop at the
  end of each floor. Needs mobs to drop gold first (Battle Arena already
  has a gold economy to borrow patterns from). See item 10 above — the
  Item Menu's universal-item pool makes this a more natural fit now.
- **Chests** — findable in the dungeon, holding items; a way to keep some
  of the "find an item" feeling once floor-item drops move to the shop.
  Chests should be defended by enemies, not free loot.

## Future Class Ability Ideas (brainstorm only)

Nothing below is scoped, designed in detail, or scheduled — pull
individual items into a real session when ready to build them.

### Rogue
- **Vanish** — Flee and Stealth at the same time.
- **Riposte** — Counter attack for 2x damage.
- **Backstab** — Bonus damage specifically when the attack comes from
  Stealth (ties into the existing `Stealthed` component).
- **Smoke Bomb** — A flee that also blinds/slows whatever you're fleeing
  from.
- **Shiv** — A cheap, low-commitment quick hit (contrast to Flurry's
  bigger multi-hit).
- **Pickpocket** — Out-of-combat: lift an item off a nearby enemy without
  a fight.

### Barbarian
- **Rampage** — Damage scales up as your own HP drops (desperation-style).
- **Second Wind** — Self-heal technique.
- **Reckless Swing** — A big hit that costs you some HP as recoil.
- **Berserk (passive)** — Bonus damage below some HP threshold.

### Mage
- **Frost Bolt** — Chance to freeze the enemy briefly — an in-battle
  cousin to Hunter's Freeze Trap.
- **Chain Lightning** — A single-target multi-hit spell (Blizzard covers
  the AOE version already).
- **Mana Shield** — An absorb/block effect, parallel to Ice Armor but
  reactive.
- **Arcane Missile** — Guaranteed hit, ignores evasion.
- **Meteor** — Skip a turn to wind up, then one big guaranteed hit.
- **Drain Life** — Damage plus self-heal in one action.

### Hunter
- **Multi-shot** — A single-target multi-hit (Arrow Volley covers the
  AOE version already).
- **Trueshot** — Ignores some/all Defense — ranged cousin to Amazon's
  Pierce Thrust idea below.
- **Snare Shot** — Immobilize/reduce accuracy instead of a flat stun, so
  it reads differently from Stun mechanically.
- **Camouflage** — A lighter, out-of-combat stealth without full
  Invisibility.

### Amazon
- Animation for Throw Spear.
- **Pierce Thrust** — A spear jab that ignores some or all of the enemy's
  Defense, rewarding you for facing armored enemies.
- **Retreating Shot** — Deal damage and immediately guarantee your next
  Defend/Flee succeeds better — "hit and create distance" instead of
  trading blows.
- **Called Shot** — A slower wind-up attack (skip this turn) that
  guarantees a big hit next turn — a ranged cousin to Counter Attack.
- **Weakpoint Strike** — A variant on Pierce Thrust that trades accuracy
  for a Defense-ignoring hit.
- **Net Trap** — A second trap variant: instead of damage, it roots/slows
  the first enemy that steps on it for a few turns (crowd control rather
  than damage).
- **Scout (Eagle Eye)** — Temporarily increases your FOV radius, letting
  you spot enemies (and Throw Spear targets) from farther away.
- **Reposition/Vault** — A short instant dash a few tiles, useful for
  breaking line of sight or repositioning before a fight.
- **Momentum (passive)** — Small damage or evasion bonus while at full
  HP, encouraging hit-and-run play.
- **Keen Eyes (passive)** — Small bonus specifically to Throw Spear's
  targeting range or bonus damage.
- **Spear Wall** — Temporary Defense boost, same shape as Ice Armor but
  Amazon-flavored.

### Cross-class note

Several ideas above (Rampage, Berserk, Momentum, Keen Eyes) are
"always-on while a condition holds" passives — a genuinely new mechanical
category. Right now every effect in the game is either a one-time
consumable (Technique, used once in battle) or a one-time out-of-combat
use (Effect, used once from the item list). A true passive — always
active, no consumption, gated on an ongoing condition like "at full HP" —
doesn't fit either shape yet and would need its own system whenever the
first one of these actually gets built.

---

# Done

## Classes — all 5 real classes fully built
Barbarian, Rogue, Amazon, Hunter, Mage are all fully implemented (stats,
weapon tiers, starting kits, and in-battle techniques + out-of-combat
abilities). Only the hidden Debug class remains an intentional
placeholder/test tool.

## Combat system
- **ATB (Active Time Battle)**, **multi-enemy battles**, **AOE
  techniques**, damage/status systems, boss-per-level, and the
  `battle/` category-module refactor — all from prior sessions.
- **Multi-hit damage popup sequencing** — a `HitQueue` lands each hit in
  a multi-hit technique one at a time (~150ms apart) instead of all at
  once, so each gets its own visible popup instead of only the last one
  surviving to render.
- **Full arrow-key cursor navigation in the battle menu** — a real 2D
  cursor (`battle::MenuCursor`): Up/Down within a column, Left/Right
  switches columns and remembers the row you were on in each one.
  Locked/unowned techniques can be pointed at but not selected.
- **Cross-battle cursor memory** (opt-in via Options: "Remember Last
  Battle Action") — a fresh battle's cursor can start on whatever action
  was last chosen for that class, persisted across sessions.
- **True ATB cursor freedom** — cursor movement is unrestricted in every
  battle state under Active mode (pre-aim while gauges race or an
  enemy's result shows); still gated to PlayerMenu under Wait mode,
  where it costs nothing anyway.
- **Hold-Enter-to-fire-ASAP** under True ATB, including through the
  queuing window while an enemy's result is still showing.
- **A narrow "don't double-fire" guard** (`pending_enter_release`) so a
  held Enter that lands a killing blow, a fatal hit, or confirms
  Adventure Select can't also immediately dismiss/confirm the very next
  screen — with a real-time debounce so it survives this project's own
  WSLg/X11 key-repeat quirks.
- Victory/Game Over screens now dismiss on Enter specifically, not any
  key.

## Out-of-combat UI — Item Menu, Ability Bar, Battle Bar
- **Item Menu** (press M) — a real arrow-key-navigable list of universal
  consumables (Healing Potion, Dungeon Map, future items), replacing the
  old fixed "1=potion, 2=map" hotkeys. Shows the selected item's live
  description (wrapped, with a fallback for anything missing one), and a
  read-only reference panel of Battle Attacks, class Abilities (full
  roster, greyed if unowned), and Weapons. Opening/browsing is free;
  using an item costs a turn.
- **Number keys 1-9, 0 now trigger class Abilities directly**, by fixed
  roster position, instead of the old potion/map slots.
- **Ability Bar** — a row of icons along the bottom of the dungeon
  screen for out-of-combat class abilities, centered as a group, red box
  border, number labels matching the real hotkeys, greyed out if
  unowned, hover for a full description.
- **Battle Bar** — a second row next to the Ability Bar for in-battle
  Techniques (reference only, not directly usable outside a fight) —
  same icon size, green box, no number labels, same greyed-out-when-
  unowned and hover-for-description behavior.
- **The dungeon-exploration HUD is now minimal** — health bar, hint
  text, dungeon level/gold, shop stock while shopping, and the two icon
  bars. The old on-screen "Battle Attacks"/"Weapons" text panels are
  gone, consolidated into the Item Menu's reference panel instead.

## Battle screen / UI
- Bordered actions box, Battle Victory screen, floating damage numbers,
  persistent battle log, active-effect status lines, theme-aware
  backgrounds, attack wiggle, and the bracket-lib black-pixel-culling
  workaround — all from prior sessions.

## Battle Arena mode — feature-complete for a full playable loop
Adventure Select → starting shop → 3 levels of (5/5/3 waves + boss) each →
shop between levels → Victory. Real gold economy, separated stats
tracking, its own shop map/UI.

## Title / meta screens
- Title screen, Class Select, Adventure Select — all now have arrow-key
  cursor navigation (yellow highlight + `►` pointer), alongside the
  number/letter-key shortcuts that already worked.
- Pause screen, **Options screen** (now with a "Remember Last Battle
  Action" toggle alongside Battle Speed/ATB Mode, also arrow-navigable),
  **History/Stats screen** (also arrow-navigable).
- Victory and Game Over screens.

## Player / dungeon
- Auto-pickup, smooth camera scrolling, idle animation infrastructure,
  map-generation and Amulet-of-Yala bug fixes — all from prior sessions.

## Sprite art
- Full character portraits and all 18 ability icons across all 5
  classes, plus the Battle Arena Shopkeeper — see
  `Dungeon_Font_Glyph_to_Cell_Map.md`. No new art this session; the
  Ability Bar/Battle Bar reuse existing glyphs at a larger render size.

## Engineering / refactors
- `main.rs`/`battle` module splits, persistent settings files
  (`keymap.ron`, `battle_speed.ron`, `atb_mode.ron`, `menu_memory.ron`,
  `last_battle_action.ron`, `stats.ron`), `.cargo/config.toml` X11 fix,
  fortress-guaranteed weapon placement — all from prior sessions.
- **Item/ability data model split** — `components::usable_menu_items`
  (universal, no `class:` tag) vs. `usable_ability_items`/
  `battle_items_carried` (class-restricted), driven entirely by the
  `class:` field templates already had. Several helper functions
  (`group_items`, `ability_bar_slots`, `battle_bar_slots`) made generic
  over legion's `EntityStore` trait so they work from both `&World` and
  `&SubWorld` contexts.
- **A permanent regression test** (`systems/hud.rs`'s
  `hud_system_execution_tests`) that actually executes `hud_system()`
  through a real `Schedule` — kept in the shipped code deliberately,
  unlike this project's other throwaway tests, specifically to catch a
  whole class of legion component-access bug that's invisible to every
  compile check (see the project instructions doc's Section 2 for the
  full story of the crash this caught).

## Stats tracking
- Games played/won, enemies killed, deepest level reached, per-ability
  usage counts, and Battle Arena's own separated stats — all from prior
  sessions.