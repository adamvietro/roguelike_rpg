# Ever Space RRPG — Ideas & Backlog

Working list of what's still ahead, and a running record of what's already
shipped. Pull individual "Working" items into a real session when ready to
build them — nothing there is scoped or scheduled just by being listed.

**Standing rule: every still-open item lives in the numbered "Near-term
priorities" list below, no exceptions.** When adding something new, either
give it its own next number, or fold it as a sub-bullet into an existing
number if it genuinely fits that number's category (the way individual
animation tasks all live under one "Animation work" number, or individual
refactor ideas all live under one "Refactoring opportunities" number).
Never leave a new item as a bare bullet in some other section with no
number of its own - that's exactly what made items easy to lose track of
before this list got reorganized (2026-09-11). Once a numbered item (or a
specific sub-bullet within one) is actually finished, move its writeup to
the matching spot in "Done" below rather than leaving a "(fixed)" note
in place here - the numbered list should always reflect what's honestly
still left to do, not a mix of open and closed work.

---

# Working

## Near-term priorities

Roughly in the order they've come up:

1. **Overall balance pass** — starting gold, starting items, and player/
   enemy stats generally, across both Dungeon Crawl and Battle Arena.
   Dungeon Crawl's starting-kit sustain already got a first, data-driven
   pass (see "Dungeon Crawl economy" in Done below). Battle Arena's own
   pass is **paused, picking back up later** - a real `arena_class_
   survivability_report` simulation now exists (`cargo test --release
   arena_class_survivability_report -- --ignored --nocapture`) and, after
   three real bugs found and fixed along the way (see docs/journal.md and
   CLAUDE.md's standing gotchas - a genuine `bracket-pathfinding` library
   defect among them), produces trustworthy data: 0/10 wins for every
   class, with Level 3 (especially its boss and late waves) a wall for
   everyone, Mage dying earliest/most often at Level 2's boss
   specifically. Paused specifically because **the bot's own shopping/
   battle policy is too limited to trust these exact numbers as a verdict
   yet** - it buys one weapon tier and spends the rest on potions, never
   buys or uses class abilities at all, and never repositions when
   fleeing (see the "known bot limitation" bullet just below, inherited
   from the Dungeon Crawl bot and not yet revisited for Arena either) -
   before drawing real balance conclusions from Level 3's wall, the bot
   itself needs to actually use more of what a real player would.
   - **Known bot limitation, not yet fixed**: the simulation's bot
     "flee below 25% HP" rule doesn't reposition, and its pathing always
     recomputes the literal shortest route to the same target - if the
     enemy it just fled is still sitting on that route (usually true),
     the very next action walks right back into it and the fight repeats
     in a loop. This is inflating "timed out" results in the latest run
     without necessarily meaning anything real about reachability - needs
     a real fix (e.g. avoid re-pathing onto the same enemy for a turn or
     two after fleeing it) before those specific numbers can be trusted.
2. **Mouse-targeted ranged AOE outside battle** ("rain of fire" for
   ranged classes) — not started.
3. **Standing "fix issues with the battle system" bucket** — not a fixed
   list, just wherever ATB/multi-enemy/the cursor system turns up real
   bugs as they get more play.
4. **Battle screen redesign — now that classes have real animated art**
   (added 2026-09-08, explicit ask: "we can do soooo much better now").
   Not scoped yet - needs a real design conversation before code, per
   CLAUDE.md's convention for anything this size. The background piece
   already shipped (see "Battle arena backgrounds" in Done below) and
   the enemy-animation pieces are folded into "Animation work" below;
   what's left on the table:
   - **A hit-impact effect timed to `HitQueue`'s own per-hit landing**
     (`battle::damage::tick_hit_queue`) - screen shake, a flash, eventually
     a sound (see the Music & sound effects item below) - right now a
     landed hit only shows the floating damage number, no impact feedback
     synced to the moment it actually connects.
   - **Visual projectiles for ranged techniques** (Arrow Volley, Javelin
     Volley, Blizzard) that travel from caster to target instead of only
     animating the caster in place.
5. **Animation work** — what already shipped (idle/walk art for all 6
   classes and 8 enemies, the played-once Death/Victory/Technique
   framework, the title-screen fix, 2026-09-11's Attack/Defend/full-
   technique-roster/enemy-attack batch, out-of-combat effect animations,
   real directional movement, and 2026-09-13's full refresh - Victory
   Stairs climb pose for all 6 classes, boss Death animations, Goblin's
   walk fix, the Shopkeeper's first-ever animation) lives in "Character &
   enemy animation" in Done below - full row-mapping reference in
   `docs/Dungeon_Font_Glyph_to_Cell_Map.md`, session-by-session history
   in `docs/journal.md`, condensed standing rules in `CLAUDE.md`. Still
   open:
   - **Redo the enemy battle sprites** (`enemy_battle.png`, 9 rows) - a
     quality/style call, not a bug; the user isn't happy with how they
     look and wants another pass.
   - **8-way static `rotations`** - real 4-way movement facing now exists
     (see "Character & enemy animation" in Done), but this game only ever
     derives a cardinal `Direction` from a move, never a true 8-way
     diagonal one (there's no diagonal movement to derive it from) -
     `rotations`' 4 diagonal poses (`north-east`/`south-east`/
     `south-west`/`north-west`) still sit completely unused. Not a
     priority right now (2026-09-13) - revisit if a real use for it
     comes up.
6. **Music & sound effects** — no crate picked yet (`rodio` is the
   leading candidate, since bracket-lib has no built-in audio support).
   The `HitQueue` per-hit timing (`battle::damage::tick_hit_queue`,
   ~150ms apart) is a ready-made hook point for a per-hit sound once a
   crate is picked - see the Battle screen redesign item above too.
7. **More class abilities** — pull a few real ones out of the "Future
    Class Ability Ideas" brainstorm list below and actually build them.
    Each class only has a handful of real abilities/techniques right now
    (see spawner::class_effect_names/class_technique_names); the
    brainstorm list has several per class already sketched (Shiv/
    Pickpocket for Rogue, Mana Shield/Arcane Missile for Mage, Trueshot/
    Snare Shot for Hunter, and more) that just need someone to pick a
    few, design the actual mechanics/numbers, and wire them into
    `template.ron` the same way the existing ones already work. Good
    candidate for a session that isn't UI/layout work for once.
    - Worth knowing going in: a few of the brainstormed ones (Barbarian's
      Rampage/Berserk, Amazon's Momentum/Keen Eyes) are "always-on while
      a condition holds" passives - a genuinely new mechanical category.
      Every effect today is a one-time consumable (Technique) or one-time
      out-of-combat use (Effect); the first true passive needs its own
      system, not just a new template entry. Pick a non-passive one first
      if the goal is a quick, contained win.
8. **Content / world**
    - **More winnable item variety** — right now a chest/shop can only
      ever contain Gold, a Dungeon Map, or a Healing Potion. Not scoped -
      could be equipment, trinkets, or anything else worth finding.
    - **Quest system** — not scoped at all yet: no design conversation
      has happened on objectives, tracking/UI, rewards, or whatever NPC/
      dialogue hook would hand them out. Worth a real design discussion
      (per CLAUDE.md's convention for architectural-sized changes) before
      any code gets written.
9. **Ability Bar/other HUD panels should go transparent when the player
    is underneath them** (added 2026-09-11) — a side effect of the camera
    changes: the player can now end up positioned under the Ability
    Bar/similar fixed UI panels, which currently just draw solid on top
    of them. Needs a design pass (which panels, "transparent" vs. "hide
    entirely," how to detect the player's screen-space position is
    actually under a given panel's cells) before touching code.
10. **Real PixelLab-generated UI art, replacing every hand-drawn ASCII
    box border** (added 2026-09-13) — every box border in the game is
    currently the same plain `-`/`|`/`+` rectangle (`render_helpers::
    draw_ascii_box`, `ever_space_rrpg/src/render_helpers.rs:232-255`),
    reused everywhere via that one shared helper:
    - Item Menu screen (`screens/item_menu.rs`): Items/Battle Actions/
      Equipped Items/Dungeon Actions list boxes, the Stats panel, the
      shared description panel.
    - Battle screen (`screens/battle.rs`): the battle log box, the
      in-combat Battle Actions box.
    - Pause screen (`screens/pause.rs`): the Hints box.
    - Dungeon HUD (`systems/hud.rs`): the shop-item tooltip, and the
      Item Bar/Ability Bar/Battle Bar frames.
    - Separately, HP/ATB gauges use a plain `[####----]` text string
      (`battle::hp_bar_string`, `battle/mod.rs:1225`), not this box
      helper - a real health-bar graphic would replace that string
      entirely rather than reuse draw_ascii_box.
    - No hand-drawn border exists yet on end.rs/options.rs/stats_view.rs/
      title.rs/chest.rs - plain text only, no boxed frames.

    PixelLab's API (confirmed via its real OpenAPI spec, not just
    marketing copy - `https://api.pixellab.ai/v2/openapi.json`) has a
    dedicated UI-generation path, not just characters:
    - `POST /generate-ui-v2` - single element from a text description
      ("wooden inventory slot with metal corners"), optional
      `color_palette`/`concept_image`/`seed`, 16px up to ~512x512.
      Async: returns a `background_job_id`, poll `GET /background-
      jobs/{id}` until `completed`.
    - `POST /create-ui-asset` - a whole panel/window rather than one
      piece - either a default full-canvas rounded-rect, an explicit
      `pieces` layout (rects/circles/polygons with real coordinates), or
      a named `elements` list (`button`, `icon_button`, `toolbar`, `tab`,
      `panel`, `window`, `health_bar`, `avatar`, `triangle`/`pentagon`/
      `hexagon`/`octagon`) that gets auto-positioned. Returns a
      `ui_asset_id` + `background_job_id`; poll `GET /ui-assets/{id}`
      for `image_url` once `status` is `completed`.
    - Also `POST /generate-font-pro` for a fully custom pixel font, if a
      matching custom font (not just terminal8x8.png) ever becomes worth
      it alongside the new panel art.
    - Base `https://api.pixellab.ai/v2`, Bearer token auth (from the
      user's own pixellab.ai account page), Python SDK available
      (`pip install pixellab`). Cost is per-call/credit-based - the
      schema's own example shows ~$0.02 for a `generate-ui-v2` call.

    Unlike the character-sheet workflow so far (generate on the
    PixelLab web UI, zip, hand the zip to Claude), this is a real
    polling REST API - once the user provides an API token (kept as a
    local env var, never committed), the whole generate -> poll ->
    download -> composite-into-a-real-sheet pipeline could be scripted
    directly instead of a manual round trip. Not started - no token
    provided yet, nothing generated.
11. **Rename the game to "Five Blades Deep"** (decided 2026-09-13) - "Ever
    Space" collides with a real existing game and never fit this
    project's fantasy dungeon-crawler genre anyway. Checked clear of
    existing games/trademarks before deciding (see docs/journal.md's
    2026-09-13 entry for the other candidates checked and why they lost
    out - several were real, sometimes uncomfortably close, collisions).
    Deliberately not touched yet - deferred to its own dedicated pass
    rather than tangled into other in-progress work. Everywhere the old
    name still needs to change once that pass happens:
    - The Rust crate/package name (`ever_space_rrpg` in `Cargo.toml`) -
      renaming this changes the built binary's own name/path, so this is
      the one piece worth doing carefully/first, not as an afterthought.
    - The in-game window title text (currently "EVER SPACE RRPG" on the
      title screen).
    - Every doc's own header/title: this file, `CLAUDE.md`, `DEVLOG.md`,
      `README.md`, `CHANGELOG.md`.
    - The local repo folder name (currently `roguelike_rpg`) - the user
      would need to be aware their own working-directory path changes.
    - The GitHub repo's own name - external, on GitHub's side, not
      something achievable from inside this repo; the user would do this
      themselves whenever ready.
    - The devlog blog's "My Roguelike" umbrella tag (id 166, see
      `docs/journal.md`'s "also a source for blog posts" convention) -
      worth deciding whether this tag gets renamed too or stays as-is
      (blog tags are shared across the user's other projects too, not
      exclusively this game's naming decision to make alone).
12. **A winding river of Sewer's Standing Sewage Water, with a bridge
    (the Rusted Metal Grating Floor tile) crossing it** (added
    2026-09-13, deliberately deferred out of that day's water-feature
    pass) - a real linear placement algorithm threading a connected
    water path across the map (similar in spirit to Forest's own dirt-
    path line, `MapTheme::path_variants` - see "Map tile themes" in Done
    below - but for an IMPASSABLE feature that needs at least one
    guaranteed walkable crossing point rather than a tile the player
    just walks along). Not started - the moat/isolated-patch/sparse-
    obstacle placement system that shipped the same day intentionally
    stopped short of this one, since a river's own crossing-point
    guarantee (never leaving the map disconnected) is a meaningfully
    harder problem than either of those.

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

## Dungeon Crawl economy — gold, a guaranteed chest, and a shop between floors

- **Gold + the `shop_only` loot flag.** `Gold` (components.rs) was
  Battle Arena-exclusive - Dungeon Crawl players now get `Gold(0)` at
  `start_game` too, and every existing gold codepath (traps, ranged
  strikes, `buy_nearby_item`, battle kills) already gated purely on the
  component's PRESENCE, so those started paying out for free, no
  per-mode branching needed. Healing Potion/Dungeon Map pulled out of
  the ambient floor-loot pool via a new `Template::shop_only` flag (same
  pattern as `prefab_only`/`boss_only`) - still grantable by name (a
  chest, the shop).
- **A guaranteed per-floor loot chest.** A new, always-attempted (not
  random-one-of-three like Fortress/Turret/Bunker) prefab room
  (`map_builder/prefab.rs`'s `apply_chest`), guarded by 1-2 copies of
  that dungeon level's single toughest non-boss enemy
  (`spawner::spawn_prefab_chest_guards` - Goblin/Orc/Ogre/Ettin's own
  natural per-level ordering, not a random pick). Walking onto it grants
  30-50 gold, a Dungeon Map, and 1-3 Healing Potions, then shows a
  full-screen `TurnState::ChestOpened` overlay (`screens/chest.rs`)
  styled like Paused - reuses `pause_systems` outright (just
  `map_render`, so sprites drawn on the map a frame ago simply aren't
  redrawn). New `c` glyph drawn from a user-supplied reference image,
  flood-filled to true transparency from its outer edge so interior
  highlights survived. The wall template originally fully enclosed the
  room with no door at all (found via a user screenshot, "I can't get
  in") - fixed by opening one on the guards' row specifically, so
  reaching the chest requires passing them, not walking past them
  (verified with a real flood-fill across 200 generated floors, both
  times).
- **A shop between dungeon floors.** A floor's own stairs now lead into
  a shop room first (`TurnState::DungeonShopTransition`) -
  `systems/end_turn.rs`'s Exit-tile check is a 3-way split now: Arena's
  own `ArenaTransition`, this, or - once `ShoppingActive` is already
  Some, meaning you're standing on the SHOP's own stairs - `NextLevel`
  again, which is what actually generates the next floor.
  `State::dungeon_shop_transition` reuses `MapBuilder::new_arena_shop`/
  `arena_rebuild_keep_player`/`spawn_arena_shop_items`/`buy_nearby_item`
  completely unmodified, stocked with a fixed Healing Potion (x5, same
  `HEALING_POTION_PRICE`) + Dungeon Map (x2, new `DUNGEON_MAP_PRICE`)
  pair instead of Arena's class-rolled list. `advance_level` (now also
  reached by leaving the shop) clears `ShoppingActive`/`ShopMessage` on
  its way out, or a freshly generated floor would silently inherit the
  shop's auto-pickup suppression and frozen FOV forever. The HUD's
  top-right corner now also shows Gold while `shopping.is_some()`, not
  just during a whole Arena run. A separate real bug turned up here
  too: `buy_nearby_item`'s player lookup had no `Player` filter, so it
  could silently grab the Shopkeeper NPC's or a `ShopStock` counter's
  `Point` instead (both exist in the same scene) - never visibly broke
  the Arena shop (apparently by luck of legion's iteration order), but
  broke the new Dungeon Crawl shop outright. Found by the user
  ("I can't buy the potion"), reproduced with a real test both before
  and after the one-line `.filter(component::<Player>())` fix.
- **A first, data-driven starting-kit balance pass.** A new headless
  class-survivability simulation (a naive-then-smarter bot plays several
  runs per class through the real game logic - schedulers, movement,
  combat resolution, chest/shop interaction) found only 3/50 runs
  reaching the first shop once ambient floor Potions/Maps were removed,
  with Mage dying 10/10. Every class's starting kit now carries 3
  Healing Potions instead of 1 (Barbarian gets a kit at all now,
  previously none), and Mage's Speed went 6 -> 7 - deaths dropped to
  2/50 across all five classes in the re-run. The simulation itself is
  now a permanent tool, not a one-off: `#[ignore]`d so it doesn't run in
  the normal `cargo test` (real time even in release) - rerun by hand
  after any balance change with `cargo test --release
  class_survivability_report -- --ignored --nocapture` (see
  `screens/battle.rs`; pointer in CLAUDE.md). See "Overall balance pass"
  above for the one known open issue with the bot itself.

## Out-of-combat UI — Item Menu, Ability Bar, Battle Bar

- **Item Menu** (press M) — a real arrow-key-navigable list of universal
  consumables (Healing Potion, Dungeon Map, future items), replacing the
  old fixed "1=potion, 2=map" hotkeys. Shows the selected item's live
  description (wrapped, with a fallback for anything missing one), and a
  read-only reference panel of Battle Attacks, class Abilities (full
  roster, greyed if unowned), and Weapons. Opening/browsing is free;
  using an item costs a turn. Later redesigned into a full 5-box
  character dashboard (Items, Equipped Items, Stats, Battle Actions,
  Dungeon Actions) plus a shared description panel - cursor reuses
  `battle::MenuCursor` as-is. Stats box is mode-aware (Arena Level/Wave
  during a Battle Arena run, Dungeon Level otherwise). Also added
  descriptions to all 15 weapon templates (previously blank).
- **Number keys 1-9, 0 now trigger class Abilities directly**, by fixed
  roster position, instead of the old potion/map slots.
- **Ability Bar** — a row of icons along the bottom of the dungeon
  screen for out-of-combat class abilities, centered as a group, red box
  border, number labels matching the real hotkeys, greyed out if
  unowned, hover for a full description. Click-to-use added later
  (`use_ability_bar_click`) - reuses the exact same `use_ability` the
  number keys already call, no new design needed.
- **Item Bar** — a third icon bar (blue box) immediately left of the
  Ability Bar, for universal consumables (`spawner::universal_item_names`).
  Sits alongside the Item Menu rather than replacing it. Click (left
  mouse button) to use directly - the first click-to-activate
  interaction in the game (`components::MouseLeftJustPressed`, a real
  physical-click edge detector, since bracket-lib's own `left_click`
  fires twice per click).
- **Battle Bar** — a second row next to the Ability Bar for in-battle
  Techniques. Dungeon-HUD-only: it never renders during the actual
  battle screen (`TurnState::InBattle` routes to a completely separate
  `battle_tick`/`screens/battle.rs` menu, keyboard-only - number keys or
  cursor+Enter against an in-panel Actions list, no icon bar at all).
  Outside a fight it's pure reference (same icon size, green box, no
  number labels, greyed-out-when-unowned, hover-for-description) - there
  is no in-battle action bar for it to mirror, so click support was
  never actually a real option to begin with (closed, not deferred).
- **The dungeon-exploration HUD is now minimal** — health bar, hint
  text, dungeon level/gold, shop stock while shopping, and the three
  icon bars. The old on-screen "Battle Attacks"/"Weapons" text panels
  are gone, consolidated into the Item Menu's reference panel instead.
- **Player-status frame** — replaced the old plain full-width health bar
  with a top-left class-portrait icon (the player's existing `Render`,
  no new art) plus a compact one-row health bar (HUD_CONSOLE has no
  sub-cell text positioning, so a multi-row bar could never center its
  "current / max" overlay). Gained small buff badges
  (`BUFF_BADGE_CONSOLE`, a new 32px-cell console) for
  Invisible/Stealthed/IceArmored shortly after - real ability icon art,
  not placeholders, queued left to right if more than one is active.
  Introducing this frame collided with the Battle Arena shop's old fixed
  top-left item list - replaced with a single tooltip for whichever item
  is adjacent to the player (`components::shop_item_near`, the same
  lookup `buy_nearby_item` uses, so it can never show something
  different from what Enter would actually buy), anchored to the
  player's own on-screen position so it travels with them.
- **Visual confirmation pass** — checked the whole out-of-combat bar
  layout (label positioning, box borders, spacing) against a real
  screenshot with real ability counts. Read cleanly; no changes needed.
- **Stack-count badge** — a small "x2"-style badge (matching the shop's
  own existing quantity convention) on the Item/Ability/Battle Bar icons
  whenever a stack holds more than one copy, opposite the hotkey number's
  corner so the two are never confused. Needed a new console
  (`ABILITY_BAR_BADGE_CONSOLE`, registered after the bars themselves) -
  these bar icons are opaque full-bleed sprite art, so a badge drawn
  directly on HUD_CONSOLE would just get painted over and never show.
  Confirmed via screenshot. Not fully sold this earns its keep long-term
  (the Item Menu already shows exact counts) - easy to revert if not.

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

- **Theme Select screen (2026-09-13)** — a new `TurnState::ThemeSelect`,
  Debug-class-only and Dungeon-Crawl-only, reached from Class Select's
  hidden 'D' shortcut instead of starting the run immediately. Lets the
  user force every floor of the upcoming run onto one `MapTheme`
  (Forest/Dungeon/Sewer, or Random for the normal per-floor roll) - a
  testing convenience so a specific theme's Victory/Defeat art (see
  "Defeat and Victory screens" above) can actually be reached without
  repeatedly restarting runs. Same arrow-key + Enter centered-menu shape
  every other title-flow screen already uses. The override is applied
  inside `MapBuilder::new` itself (a new `forced_theme` parameter),
  BEFORE tile-variant assignment runs - applying it after would have left
  variants assigned under the wrong theme's own variant-style semantics.
  Arena is unaffected (its Victory/Defeat backgrounds don't depend on
  the dungeon theme at all).
- Title screen, Class Select, Adventure Select — all now have arrow-key
  cursor navigation (yellow highlight + `►` pointer), alongside the
  number/letter-key shortcuts that already worked.
- **Pause screen overhaul** — moved off the old tiny 8px console onto
  the same BIG_TEXT_CONSOLE/HUD_CONSOLE split every other menu uses. A
  real arrow-key + Enter menu (Resume/Options/Quit, matching every other
  menu screen's convention) - Escape/O/Q still work directly too,
  purely additive. The dungeon HUD's old permanent "how to play" hint
  moved here as a rotating "Hints" box in the lower third of the screen
  (3 lines, cycling every 4s, freely add/remove from `PAUSE_HINTS`).
- **Options screen** (now with a "Remember Last Battle Action" toggle
  alongside Battle Speed/ATB Mode, also arrow-navigable), **History/
  Stats screen** (also arrow-navigable).
- Victory and Game Over screens.

## Player / dungeon

- Auto-pickup, smooth camera scrolling, idle animation infrastructure,
  map-generation and Amulet-of-Yala bug fixes — all from prior sessions.
- **Camera now clamps to the map's own bounds** (2026-09-11,
  `title-screen-upgrades` branch) — `Camera` previously had no bounds
  awareness at all: a fixed `DISPLAY_WIDTH x DISPLAY_HEIGHT` window
  centered exactly on a target point, with black space showing wherever
  that window extended past the map's real `SCREEN_WIDTH x
  SCREEN_HEIGHT` edges. Most visible on the title screen (its one-shot
  camera placement centers on whatever random point a map architect
  picked as `player_start`, rarely the map's actual center), but the
  identical bug affected real dungeon-crawl play near any map edge too.
  Fixed centrally with a new shared `Camera::clamped_top_left`, used by
  both `Camera::new` and `on_player_move`, per the user's own call to
  fix both places with one shared fix rather than patching the title
  screen alone. The player's on-screen position is no longer always
  dead-center as a result - it shifts off-center near an edge instead of
  ever showing black past the map's real bounds. `components::
  camera_render_offset` (the glide's sub-pixel smoothing) needed a
  matching update, since its old math assumed an unclamped camera that
  always centers exactly on the target - now interpolates between the
  clamped camera position at both ends of a glide instead. Verified with
  an exhaustive test (every possible position on the map, not just
  samples) that the window can never extend past bounds.

## Battle arena backgrounds — real painted scenes for Forest/Dungeon/Sewer

Replaced the original theme-tinted-floor-and-border implementation (a
flat tinted CP437 glyph fill plus a vignette and one of two generic
scenery overlays, unchanged since before the game had any real pixel
art) with one full painted scene per theme
(`resources/battle_backgrounds.png`, 1280x800 per theme) - a JRPG-
battle-backdrop style deliberately picked over more `map_tiles.png`-
style tiling, since tiling would need ever-more themed tile variety to
avoid looking stale. Driven by a new `MapTheme::battle_background_row`
(mirrors `tile_row`'s exact `Option<u16>` shape/fallback); a theme
without real art yet still falls back to the old procedural fill
unchanged. Confirmed live in a real fight, all three themes.

- **Getting the art right took real iteration.** Sourced externally
  (PixelLab is built for character sprites, not full painted scenes) -
  Dungeon and Sewer worked on the first attempt; Forest took four
  rounds (breaking a diffusion model's default mirror-symmetry, removing
  hidden creature figures the prompt explicitly excluded, then getting
  it to read as an enclosed "stage" rather than an open woodland
  clearing). Two small hidden creatures that slipped through in the
  final version were patched out locally with a feathered clone-stamp
  instead of spending another generation on them.
- **Needed a genuinely new console, `BATTLE_BACKDROP_CONSOLE`** - has to
  sit below the battle screen's own text and creature portraits, which
  were still bare literals (`2`/`3`) since the very start, never touched
  by any of this project's four previous "insert a console early, renumber
  everything after" moves (all of which only ever needed to go below the
  HUD). Promoted both to named constants (`FINE_TEXT_CONSOLE`/
  `BATTLE_PORTRAIT_CONSOLE`) as part of the insertion; every console
  index in `main.rs` shifted by exactly 1 - a fully mechanical,
  grep-verified change.
- **Two real bugs found and fixed.** A launch crash from a new shape of
  the glyph-32 gotcha (a single-column "one glyph = one full image"
  sheet needs 33+ total cells just to contain index 32, which a 6x6 grid
  solves far more sanely than a 1x33 one would - see CLAUDE.md's
  standing gotchas for the full writeup). And - caught only by the
  user's own live screenshot - near-black pixels in the art (shadows,
  mortar lines, canopy gaps) rendering as stark white cracks, since
  bracket-terminal's WITH-bg console shader falls back to the background
  color for any texture pixel whose RGB is all <=0.1 (~25/255) or whose
  alpha isn't fully opaque - the exact same rule already documented for
  a `_no_bg` console, which this session's own design assumptions had
  gotten wrong. Fixed by flooring every channel to >=30 across all three
  images (imperceptible visually) and switching the fallback color from
  white to black as defense-in-depth. Full write-up in `docs/
  journal.md`'s 2026-09-11 entries.

## Battle arena enemy positions - a zigzag formation, per-theme rows, and a debug shortcut to test it

`enemy_portrait_position`'s old fixed coarse-grid coordinates (tuned back
when the arena background was a flat procedural fill with nothing near
the edges) broke against the real painted backgrounds - confirmed by a
real 2-enemy screenshot showing the rightmost column sitting flush
against the frame's own right edge, and an upper tier reaching into
Forest's tree/fence perimeter. Took four rounds of real live feedback to
land, each verified against the actual theme art (crop each theme's cell
out of `resources/battle_backgrounds.png`, overlay the candidate grid,
check for overlap) rather than guessed blind:

- **Single row, evenly spread** - fixed the edge-clipping and fence-
  overlap, but read as visually flat/robotic once seen live ("I don't
  like the line of enemies").
- **Shallow zigzag** between a back row and a front row, alternating by
  index parity (a 3-enemy fight reads as a wedge, 2/4-enemy as a
  diagonal/zigzag) - fixed the "flat line" complaint. Needed the
  Actions box's own position (`box_y_base`) pushed down to match, since
  the front row puts an enemy's text lower than the old single row did.
- **Shifted right** (`LEFT`/`RIGHT` from 1.4/3.8 to 1.9/3.9) after live
  feedback the formation still read as too close to the player/too
  central - close to the practical ceiling before re-clipping the frame
  edge.
- **Per-theme row values** (`MapTheme::enemy_formation_rows`, same
  `Option`-free default/override shape as `tile_row`/
  `battle_background_row`) after "we need to move them up" - Forest's
  fence genuinely caps how high the back row can go (screenshot-
  verified: 1.7 re-clipped it), but Dungeon/Sewer's much thinner top
  wall/pipe band has real headroom (tested as high as 0.9 before
  clipping Dungeon's window sill/torch/crate; landed on 1.5/1.9 as clean
  on both, with Sewer having room to spare beyond that). Forest keeps
  the original 1.9/2.3 as the trait's own conservative default.
- **Debug "Battle 4" cheat** - a new Debug-class item (glyph `9`,
  `ProvidesEffect::DebugBattle4`) that spawns 4 fresh Goblins next to the
  user and starts a real Battle against them instantly, added
  specifically because "trying to run an instance and finding 4 enemies
  and then trying to round them up" made this whole formation hard to
  test in normal play. New `Templates::spawn_named_enemy_via_commands`
  (mirrors the existing `spawn_named_item_via_commands`) spawns one
  exact named enemy via `CommandBuffer` rather than a random weighted
  pick - the same "only a SubWorld + CommandBuffer available" shape.

## Character & enemy animation

All 5 playable classes (Barbarian, Rogue, Amazon, Hunter, Mage) plus the
hidden "Debug" dev/test class have real PixelLab.ai idle/walk art across
every sheet, and all 8 enemies have their own dedicated idle/battle
sheets. Full row-mapping reference and every confirmed gotcha live in
`docs/Dungeon_Font_Glyph_to_Cell_Map.md`; full session-by-session history
in `docs/journal.md`; condensed standing rules in `CLAUDE.md`. See
"Animation work" in Working above for everything still outstanding now
that a full new animation batch is being assembled.

- `resources/character_idle.png` (6 cols x 8 rows) drives the dungeon/
  Battle Arena/Class-Select/title-screen walk-in-place loop, real Walk/
  south frames for all 6 classes. Row 5 permanently blank (glyph-32
  collision on this sheet's column count); one free row (7) left before
  a resize is needed.
- `resources/character_battle.png` (8 cols x 8 rows) drives the
  battle-screen portrait animation, real Fight_Stance_Idle/east frames
  for all 6 classes. Player-only - enemies use their own dedicated
  sheets below. Row 4 permanently blank (same collision, different row
  for this sheet's column count); one free row (7) left.
- `resources/character_portrait.png` (6 cols x 8 rows, column 0 only)
  drives every still-icon site (Class Select's non-highlighted row, the
  dungeon HUD portrait, both in-battle and run-ending Victory screens,
  the Game Over screen's rotated fallen pose), real rotations/south.png
  for all 6 classes. Every old dungeonfont-glyph fallback in these
  lookup paths is dead code in practice - no class currently falls
  through to it - but kept as the fallback for any future class added
  without art yet. Two free rows (6, 7) left.
- **Played-once Death/Victory/technique animations (2026-09-08).** A new
  `OneShotAnimation` type (components.rs) - plays through its frames once
  and holds the last one (or loops, for a multi-hit/AOE technique) - on
  three more sheets, each `EXTRA_ANIM_COLS` (9) wide:
  `resources/character_death.png` and `resources/character_victory.png`
  (all 6 classes have a row, replacing the old rotated-glyph Game Over
  pose and static Victory portrait) and `resources/character_
  technique.png` (keyed by COMPOUND `(class, technique name)` since a
  class can end up with several - currently Rogue/Flurry, Hunter/Arrow
  Volley, Barbarian/Whirlwind, Amazon/Javelin Volley, Mage/Blizzard;
  Debug has none, since its "techniques" are cheat items). A multi-hit/
  AOE technique's animation loops for as long as its `HitQueue` is still
  landing damage instead of freezing on frame 1; every technique
  animation runs much faster than the idle loop's own pace
  (`TECHNIQUE_FRAME_DURATION_MS`, 80ms vs. 350ms) and never gets the
  "Attacking" flash's usual wiggle-shake layered on top of it. Full
  technical detail in `docs/DEVLOG.md` and `docs/
  Dungeon_Font_Glyph_to_Cell_Map.md`'s "One-shot animation sheets"
  section.
- **Enemy art (2026-09-08): all 8 enemies done.** Enemies get their own
  dedicated sheets (`resources/enemy_idle.png`, `resources/
  enemy_battle.png`, both 9 rows) rather than more rows on the character
  sheets above - see the design conversation and full row-mapping in
  `docs/Dungeon_Font_Glyph_to_Cell_Map.md`'s "Enemy sheets" section.
  Done: Goblin, Orc, Ogre, Ettin, Goblin Chieftain, Orc Warlord, Ogre
  Warlord, Ettin Overlord. Verified end-to-end with real screenshots
  (Goblin and Orc both checked live) in the dungeon walk loop and in
  real fights - clean, no bleed, no leftover placeholder art.
  - **"Ogre Warlord" is a brand-new enemy**, not a reused name - a
    second possible boss for BOTH Level 1 (alongside Orc Warlord) and
    Level 2 (alongside Ettin Overlord), per a design conversation
    (`Templates::spawn_boss` already supported multiple `boss_only`
    templates per level with zero code changes needed). Placeholder
    stats (hp 13/dmg 3/speed 4) deliberately sit between its two fellow
    bosses - not yet playtested live for balance.
  - **Amazon's Walk redo (2026-09-08) also fixed a real defect** - held
    back rather than shipped the same day it was first generated, same
    call later made for Orc Warlord below; the regenerated batch came
    back clean, confirmed by screenshot.
  - **Orc Warlord's redo (2026-09-08) fixed a real defect** - its first
    batch's Walk/Fight_Stance_Idle both came back as a genuine PixelLab
    generation defect (a thin off-model sliver, not a full character);
    the regenerated batch came back clean, confirmed by screenshot, and
    now occupies row 6 on both sheets. Its Walk/south came back with 8
    frames, more than the idle sheet's own 6-column ceiling
    (`MAX_IDLE_FRAMES`) - 6 of the 8 were evenly sampled rather than
    truncated, so the cycle doesn't skip its back half.
- **Title-screen enemies now actually walk in place (2026-09-11,
  `title-screen-upgrades` branch).** They already had real idle-
  animation frames wired up - the bug was `tick_idle_animation_system`
  only running once every 400ms (the enemy-wander throttle) and only
  getting that one triggering frame's real elapsed time each time, not
  the ~400ms that had actually passed, so a frame took ~9 real seconds
  to advance. Fixed by moving animation ticking into the schedule that
  already runs every real frame, decoupling it from the movement
  throttle. Verified with real screenshots 80ms apart showing a
  stationary enemy visibly cycle several poses before its next scheduled
  step.
- **Attack/Defend animations, every remaining technique, enemy attack
  animations, Idle_Battle_Stance refresh (2026-09-11, `new-animation-
  batch` branch).** A much larger PixelLab export per class (all 5
  playable classes + Debug + all 8 enemies) covering Attack, Defend,
  Death, Victory, Idle_Battle_Stance, 4-directional Walk, 8-way
  rotations, and a named animation for nearly every real battle
  Technique. Shipped this pass ("ready now" - see "Animation work"
  above for what's still deferred): two brand-new sheets,
  `resources/character_attack.png`/`resources/character_defend.png`
  (own consoles, `CHARACTER_ATTACK_CONSOLE`/`CHARACTER_DEFEND_CONSOLE`),
  replacing the old "just keep showing Idle_Battle_Stance" behavior for
  a plain Attack/Defend; `resources/character_technique.png` widened
  from 8 to 20 rows to fit a real animation for every remaining
  technique (18 rows populated, up from 5); a new enemy-side sheet,
  `resources/enemy_attack.png` (`ENEMY_ATTACK_CONSOLE`, registered
  fancy so a 2+ enemy fight's zigzag formation stays correct during an
  enemy's own attack) - enemies previously had no one-shot-animation
  concept at all; and a full `character_battle.png` Idle_Battle_Stance
  refresh for every class, including Amazon (whose export used a
  differently-named folder, `Idle_Battle_Animation`). `Battle::
  player_technique_animation` renamed to `player_action_animation`,
  now shared by Attack/Defend/Technique (mutually exclusive per turn),
  with a new sibling `player_action_kind` field recording which of the
  three separate sheets/consoles a glyph index resolves against. Full
  row-mapping detail in `docs/Dungeon_Font_Glyph_to_Cell_Map.md`'s
  "2026-09-11's full animation batch" section. A mid-session follow-up
  "v2" zip filled 3 gaps found during the initial inventory (Barbarian's
  Counter Attack, added as technique row 18; Amazon's missing
  Idle_Battle_Stance; Hunter's Shoot, an out-of-combat Effect held for
  the deferred work) and renamed Amazon's `Spear_Volley`/`War_Cry`/
  `Spear_Throw` art folders to match their real `template.ron` names
  (`Javelin_Volley`/`Battle_Cry`/`Throw_Spear`).
- **Out-of-combat effect animations for all 7 class Abilities (2026-09-11,
  `new-animation-batch` branch).** Ice Armor, Invisible Cloak, Stealth,
  Throw Spear, Trap, Freeze Trap, and Shoot now play a real one-shot
  animation in the DUNGEON VIEW - the first animation system in this
  project that isn't the battle screen. New sheet
  `resources/character_effect.png` (9 cols, 8 rows, one row per (class,
  ability) pair - keyed by the real item name, not by `ProvidesEffect`
  variant, since `RangedStrike` alone covers both Amazon's Throw Spear
  and Hunter's Shoot with different art). New component `EffectAnimation`
  gets attached the instant one of these 7 effects applies
  (`systems/use_items.rs`) and ticks/clears itself
  (`systems::animation::tick_effect_animation`); `entity_render.rs`'s
  `idle_glyph`/`idle_sheet` - the two functions every dungeon-view render
  path already funnels through - check for one before ever looking at the
  ordinary `IdleAnimation` loop, so no render path needed touching
  individually. New `CHARACTER_EFFECT_CONSOLE`/`_SCROLL_`/`_GLIDE_` trio
  in `main.rs`, inserted (not appended - a real mid-chain shift, every
  console from `HUD_CONSOLE` on moved up by 3) at the same z-order tier as
  `CHARACTER_IDLE_CONSOLE`'s own trio, so it stays below the HUD/Ability
  Bar. Verified with a permanent legion-access regression test (see
  CLAUDE.md) rather than a remove-after-verifying one, matching
  `hud_system_execution_tests`'s own precedent. Full detail in
  `docs/Dungeon_Font_Glyph_to_Cell_Map.md`'s "Out-of-combat effect
  animations" section. Still open: the movement-animation work (real
  walk-cycle synced to actual movement, 4-directional facing) - see
  "Animation work" above.
- **Pixel-health floor fixed and applied retroactively to every sheet
  built this session (2026-09-11).** The user flagged transparent pixels
  in Hunter's animations, which turned out to be two separate real
  findings from a full scan of all 1549 source frames (every class +
  enemy, every animation, every direction, both original and v2 zips):
  (1) source alpha was confirmed genuinely clean everywhere (strictly
  binary 0/255, no anti-aliasing) - NOT the issue; (2) 6-12% of every
  character's OPAQUE pixels were near-black, and the build script's
  `center_crop_32` had never actually applied the near-black floor
  CLAUDE.md's own PixelLab gotcha already called for, despite being used
  for every sheet built this session - a real gap, not a one-off. Fixed
  by adding `floor_pixels` (floors near-black opaque pixels to
  RGB>=30/channel, and defensively forces transparent pixels' RGB to
  true black) into the one function every frame already passes through,
  then rebuilding and re-verifying (a full re-scan of all 10 rebuilt
  sheets confirmed zero near-black-opaque and zero non-black-transparent
  pixels) and re-copying all 10 into `resources/`: `character_battle.png`
  /`_death`/`_victory`/`_technique`/`_attack`/`_defend`/`_effect.png`,
  `enemy_battle`/`_attack`/`_idle.png`. Also separately caught and fixed
  by the same defensive floor: Ettin Overlord specifically had ~4700
  transparent pixels with leftover non-black RGB (the OTHER real risk
  the near-black gotcha describes - a plain console's shader is a pure
  RGB colorkey that never reads alpha, so this would have rendered as a
  solid wrongly-opaque block on that enemy specifically). A third check
  (enclosed transparent "holes" not connected to the frame border) found
  real holes in ~39% of frames but mostly legitimate negative space, not
  defects - deliberately NOT auto-fixed, see the open item above.
- **Real directional movement animation, and the "becomes static while
  moving" bug fixed (2026-09-11, `new-animation-batch` branch).** The
  player and enemies used to freeze their pose for the entire ~220ms of
  every glide between tiles - `MovingAnimation` only ever tweened
  position, there was never a real per-frame walk cycle under it, and
  `tick_idle_animation` deliberately paused during a glide on the
  (wrong) assumption that movement already had its own animation. Fixed
  by removing that pause - `IdleAnimation` now advances continuously
  whether an entity is standing still or mid-glide - and by finally
  wiring up the 4-directional Walk art (`north`/`south`/`east`/`west`)
  that's been sitting unused in every zip since the very first batch: a
  new `Direction` enum (4-way only - no diagonal movement exists to
  derive an 8-way facing from), computed once per committed move in
  `systems/movement.rs` via `Direction::from_move`, which rebuilds the
  mover's `IdleAnimation.frames` in place for the new facing (preserving
  `frame_index`/`elapsed_ms`, so a walk cycle doesn't restart mid-stride
  on a turn). `resources/character_idle.png`/`enemy_idle.png` both
  widened from 1 row per class/enemy to 4 (25/33 rows respectively,
  same forbidden-row-exception shape as every other sheet this project
  has widened). Runs for the player, every real dungeon enemy, AND the
  title-background's decorative wandering enemies alike - no separate
  code path needed since `entity_render.rs`'s `idle_glyph`/`idle_sheet`
  already just read whatever's currently in `IdleAnimation.frames`.
  Verified with a permanent legion-access regression test
  (`systems::movement::facing_access_tests`). Full detail in
  `docs/Dungeon_Font_Glyph_to_Cell_Map.md`'s "Real directional movement
  animation" section. Out-of-combat effect animations stay south-only/
  undirected, unaffected by this - a stationary ability-use pose doesn't
  need facing.
- **`Breathing_Idle` closed, not deferred (2026-09-13).** Was originally
  set aside as unused per standing instruction (didn't read well as a
  battle-portrait loop) - now confirmed not needed at all, since
  `Idle_Battle_Stance` already covers that same real animated loop.
- **Enclosed-transparent-region "hole" review closed, not deferred
  (2026-09-13).** 2026-09-11's flood-fill scan found real holes in ~39%
  of frames but a spot check judged most legitimate negative space
  rather than defects - decided that's good enough as shipped, no
  further human pass needed.
- **Full animation refresh batch (2026-09-13, `enemy-death-victory-
  backgrounds` branch): Victory Stairs climb pose, boss Death
  animations, Goblin's walk fix, Shopkeeper's first-ever animation.**
  12 zips (all 6 player classes, the 4 bosses, Goblin, Shopkeeper), each
  with the usual per-frame canvas-size drift (40x40/44x44/48x48 despite
  metadata.json's declared 32x32) needing the standard center-crop-32 +
  near-black-floor pipeline, and several PixelLab auto-generated-caption
  folder names needing remapping (3 of 4 boss Death folders, 4 of 6
  player climb-pose folders, confirmed by actually opening frames and
  checking the motion rather than trusting the caption text alone).
  - **Victory Stairs / `VictoryPose::ClimbAway`** - a real north-east-
    facing "climbing away" animation for all 6 classes, added as a new
    row block (8-13) on `character_victory.png` rather than a separate
    sheet - the same "grow the sheet" convention `character_idle.png`
    used for 4-directional Walk. Wired into the `ClimbAway` pose slot
    that had been falling back to the ordinary face-camera animation
    since the Victory/Defeat backgrounds work shipped.
  - **Boss Death animations** - new `enemy_death.png` sheet (south-west
    facing, matching `enemy_battle.png`'s own orientation), one row per
    boss (Orc Warlord, Ogre Warlord, Ettin Overlord, Goblin Chieftain).
    Wiring this into `record_enemy_kill` (screens/battle.rs) turned into
    a real design decision: keeping a "dying" enemy inside `battle.
    enemies` until its animation finished would have meant auditing
    every ATB-gauge-fill/targeting loop that iterates it (several call
    sites, plus the headless class-survivability simulation's own copy
    of the battle loop) for a "still dying, skip it" guard. Instead, the
    animation is a **new independent `Battle::dying_effects` list** -
    rewards/removal/the fight-over check all still happen exactly when
    they did before, and the death animation plays as a pure decorative
    overlay ticked/drawn every `battle_tick` frame, dropped once
    finished. The one real consequence: if the LAST enemy in a fight has
    a death animation, the Victory screen still appears on the old
    timing and the overlay simply gets cut short by that transition,
    same as every other in-flight battle effect (flash, wiggle, popup)
    already does - confirmed safe by rerunning both the normal test
    suite and the headless `class_survivability_report`/
    `arena_class_survivability_report` simulations (still 0/10 full-run
    wins, unchanged - see "Overall balance pass" above, this didn't
    touch balance).
  - **Goblin's walk redo** - fixed the spear-reads-as-a-helmet look from
    the original art; new 4-directional Walk rows on `enemy_idle.png`,
    same 9-sampled-to-6-frames convention as Orc Warlord's own earlier
    redo. Nothing else about Goblin touched.
  - **Shopkeeper's first-ever animation** - new dedicated
    `shopkeeper_idle.png` sheet/console trio (`SHOPKEEPER_IDLE_CONSOLE`/
    `_SCROLL_`/`_GLIDE_`, same plain/fancy/fancy shape as
    `CHARACTER_IDLE_CONSOLE`'s own trio) plays `Idle_Selling` as its
    permanent standing loop wherever it's drawn (dungeon shop + Arena
    shop), replacing the old static 'W' glyph. A new `IdleSpriteSheet::
    Shopkeeper` variant reuses the existing `IdleAnimation`/
    `tick_idle_animation` machinery outright rather than a bespoke
    component - the Shopkeeper never moves or turns, so none of the
    facing-rebuild logic that exists for player/enemy entities ever
    triggers for it. `Walk`/`Breathing_Idle` from the same zip went
    unused per explicit instruction (it never moves).

## Sprite art

- Full character portraits and all 18 ability icons across all 5
  classes, plus the Battle Arena Shopkeeper and the dungeon Treasure
  Chest (`c` glyph) — see `Dungeon_Font_Glyph_to_Cell_Map.md`.
- **AOE technique icons** — Whirlwind (`≤`, Barbarian), Blizzard (`÷`,
  Mage), Flurry (`≥`, Rogue), Javelin Volley (`√`, Amazon), Arrow Volley
  (`■`, Hunter) all got real art from user-supplied references.
- **Debug class icons** — fixed a real glyph collision (Debug's own
  player-portrait glyph was `D`, the same codepoint as Deathblow's
  already-finalized icon); Debug's portrait is now `N`, with real art (a
  robot). Defeat (`M`) also got real art, rendered from a black-and-white
  pixel-pattern chart reference rather than painted art. Next Level ended
  up not needing a new icon at all - re-pointed at the plain `>` the
  dungeon's own `TileType::Exit` stairs tile already renders as. Victory
  (`L`) took two tries - the first trophy reference had a visible tiled
  watermark and was declined; a clean second version of the same art was
  supplied and used. Since Debug is a hidden test-only class none of this
  was ever a priority beyond the original collision fix, but all four
  ended up finalized anyway.

## Map tile themes — real per-tile textures, Forest/Dungeon/Sewer, and an easy-to-extend theme pool

The old single-colored-glyph-per-`TileType` map rendering (`.`/`#` for
Dungeon, `;`/`"` for Forest) is gone for three themes so far - real
32x32 pixel-art tiles, randomly picked per generated level via
`map_builder::dungeon_theme_pool()` (adding a fourth theme is one line
there, not a hand-counted range). Full template/row-mapping reference,
every generator gotcha, and the generation algorithm's own reasoning
live in `docs/Map_Tile_Theme_Guide.md` - the map-rendering counterpart
to `Dungeon_Font_Glyph_to_Cell_Map.md`.

- **Generation algorithm**: every tile starts on its theme's plain
  default (the whole point of requiring cell #1/#5 to be the plainest
  look in a theme's 16-cell set), then wall tiles get a scattered
  minority of individual accent swaps and floor tiles get a handful of
  contiguous randomly-sized patches of one alternate variant -
  confirmed necessary after a uniform per-tile random pick made floor
  and wall hard to tell apart at a glance. A further per-variant
  `Patch`/`Scatter` style (`MapTheme::floor_variant_style`) followed
  once Dungeon's torchlight-glow cell, patched as a whole region, came
  back looking like a literal wall of torches - discrete point fixtures
  (a torch, a grate, a bone pile) scatter; spreadable ground cover
  (moss, a puddle, algae) still patches.
- **Two real regressions caught in actual play**: the Shopkeeper
  vanished on any Floor/Wall tile and Orc Warlord visibly popped in and
  out while walking, both from the same root cause - the new tile
  console sat above the dungeon's oldest, most foundational entity
  console (a bare literal "console 1" before this session, now a real
  named `ENTITY_CONSOLE`) instead of below it. Sewer's walls also read
  as too visually flat - fixed with a flat darkening multiply on
  real-texture wall tiles, benefiting every theme at once rather than
  needing new art.
- **`TileType::Water`** is now real, placed data (2026-09-13, see "Map-
  gen refactor" below) - blocking like `Counter`, but deliberately NOT
  opaque, so a moat still lets the player see what's on the other side.

## Map-gen refactor — `map_builder`'s single-tile-set assumptions, three phases

Backlog item 8 - `map_builder` still baked in some single-tile-set
assumptions even after the multi-theme system above shipped. Branch
`refactor-map-builder-item-8`, three phases, each verified with the fast
test suite plus both headless class-survivability simulations (all
consistent with prior documented patterns - Mage weakest, boss walls at
L2/L3):

- **Phase 1 (structural, no visual change)**: `MapTheme` gains `floor_
  variant_count()`/`wall_variant_count()`, defaulting to every existing
  theme's current shape, so a future theme can declare a different
  count instead of being forced into shared global constants (removed:
  `FLOOR_VARIANT_COUNT`/`WALL_VARIANT_COUNT`). Also gains `exit_tile()`/
  `counter_tile()` hooks (raw atlas cell, defaulting to `None`) so a
  theme CAN give Exit/Counter real art later - unused by all three
  themes so far. Every `MapArchitect` had repeated an identical
  `MapBuilder` struct literal just for a throwaway placeholder theme -
  extracted into `MapBuilder::blank()`. `prefab.rs`'s template parsing
  now panics on an unrecognized marker instead of silently `println!`-
  ing and no-op-ing.
- **Phase 2**: Forest's Dirt Path/Path Fork cells rendered as a random
  circular blob (the default `Patch` treatment every other floor variant
  gets) rather than anything resembling a path - the user's own
  complaint ("we just have a circle of path tiles"). New `MapTheme::
  path_variants()` hook; when set (Forest only), `assign_tile_variants`
  excludes both variants from the normal patch/scatter pools and instead
  walks the real shortest route between `player_start` and `amulet_
  start` (greedily descending a BFS distance field, ties broken at
  random so open rooms still wobble naturally while corridors stay
  straight), stamping a genuinely connected line. `Path Fork` marks just
  the path's own north-most endpoint rather than a real second branch -
  a real branch would need to point toward wherever the OTHER branch
  actually goes, which is a materially harder problem than the
  horizontal/vertical rotation fix below (only two fixed orientations,
  decided purely from a tile's own immediate neighbors). Promoted
  `bfs_distance_field` out of `screens/battle.rs`'s class-survivability
  bot (which had its own private copy of the exact same BFS-not-
  DijkstraMap logic) into `Map::bfs_distance_field`, now shared by both.
  - **Follow-up**: the connected line's single dirt-path texture was
    drawn as a north-south trail, so an east-west run of it looked
    visibly wrong - the user's own screenshot caught this. Several
    rounds of a LIVE `set_fancy` rotation were tried and each produced
    its own real rendering defect in practice (black bars, wrong-
    orientation fragments bleeding through, flicker between panning/at-
    rest states, then persistent hairline seams even after tracing
    bracket-terminal's actual shader/vertex source and confirming the
    rotation math itself was sound) - traced as far as confirming
    bracket-terminal's font textures use NEAREST filtering with zero UV
    padding between atlas cells, a well-documented class of bug for
    rotated pixel art generally (an edge fragment landing on a texel
    boundary can round into the adjacent atlas cell), but even the
    standard mitigation for that (a small overscale) still weren't
    enough in real play. **Fixed properly on the user's own suggestion**:
    instead of rotating anything at render time, `resources/map_tiles.
    png` now has a real second atlas cell - the previously-unused Path
    Fork cell (cell 10, nothing used it for real branching anyway),
    replaced with the Dirt Path texture pre-rotated 90 degrees as a
    one-time offline image edit, not a runtime transform. `MapBuilder::
    stamp_theme_path` now decides horizontal vs. vertical for each tile
    at GENERATION time (from its own path neighbors) and bakes that
    choice directly into `tile_variant` as one of the two real cells -
    `map_render.rs` needed no path-specific code at all afterward, since
    a horizontal path tile is now just an ordinary static glyph like any
    other Floor variant. `MapTheme::path_variants()` renamed in spirit
    from `(main, fork)` to `(vertical, horizontal)` to match. A corner
    tile (connects both ways) still has no single right answer and keeps
    the vertical/default look, same as always - real corner art would be
    its own separate piece of work.
- **Phase 3**: the "special wall" row (13-16, never placed by any
  generator before this) put to real use, differently for its two kinds
  of cell:
  - **Liquid cells** (Forest's Water, Sewer's Standing Sewage Water AND
    Toxic Sludge Pool - `MapTheme::water_variants()`, a theme can
    register more than one distinct look) are blocking but deliberately
    NOT opaque (`Map::is_opaque` gained a `TileType::Water` special
    case) - a moat should still let the player see through it, the
    whole reason to use water instead of a solid wall. Used for: any of
    the three `apply_prefab` room shapes (FORTRESS/TURRET/BUNKER)'s own
    wall ring, with a chance (`prefab_moat_variant` + `PREFAB_MOAT_
    CHANCE_PCT`, 50% - Forest uses Water, Sewer uses Sludge, Dungeon
    deliberately stays plain wall always - "the dungeon doesn't really
    have a special tile like [that]"; a real screenshot of a plain
    Turret/Bunker next to an always-watered Fortress prompted widening
    this from Fortress-only-and-guaranteed to all three, each
    independently rolled), the CHEST_ROOM prefab's own ring
    (`chest_moat_variant`, Sewer's Dirty Water only, still guaranteed
    when set - only the Fortress/Turret/Bunker trio was asked to vary),
    and a handful of small isolated Wall-to-Water patches elsewhere on
    the map (`wall_water_patch_variant`, Sewer only) - purely cosmetic,
    since Water is exactly as blocking as the Wall it replaces.
  - **Solid-obstacle cells** (Forest's Stump/Log/Briar, Dungeon's
    Rubble/Pillar/Portcullis Chunk, Sewer's Pipe-Valve/Collapsed Grate)
    join the Wall variant pool via the same two-row split Floor already
    has (`MapTheme::wall_obstacle_variants`), placed through a new,
    much rarer pass than the existing per-tile wall accent, with a hard
    "at most one per 5x5 area" spacing check ("I don't want a lot of
    them... used sparingly") - a plain low-probability independent roll
    alone doesn't guarantee that, hence the explicit neighborhood check
    before placing each one.
  - `MapBuilder::new` had to move theme selection earlier (before
    `apply_prefab`/`apply_chest` instead of after) so those two could
    read the real theme's own moat variants instead of the still-unset
    placeholder.
  - The river-with-a-bridge idea (a winding Sewer water path crossed by
    its Rusted Metal Grating Floor tile) was deliberately deferred - see
    item 12 in Working above.

## Documentation

- **README.md pass** — controls table, combat system section, and
  current-state summary rewritten to match reality (M opens a full
  dashboard, number keys mean abilities, Item Bar/Ability Bar clicking,
  Pause is a cursor menu). Kept up to date in small pieces since (e.g.
  the Dungeon Crawl shop/chest mention added when that shipped).
- **CHANGELOG.md** added — dated, player-facing entries (this project
  doesn't use version numbers).

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
  compile check.
- **A permanent class-survivability simulation** (`screens/battle.rs`'s
  `class_survivability_report`, `#[ignore]`d) - see "Dungeon Crawl
  economy" above.
- **Shared shop-room building, deduplicated across all three callers** —
  `start_arena`'s very first Arena shop, `arena_advance_to_next_shop`'s
  later ones, and Dungeon Crawl's own `dungeon_shop_transition` had all
  accumulated their own copy of the same "build the room, reveal it,
  freeze FOV, spawn the Shopkeeper, stock it, set the Exit tile" logic.
  Extracted into one `build_shop_room` helper; each caller now only
  handles its own TurnState/Battle/ArenaRun/Gold/Stats specifics
  afterward, which differ too much between a fresh-world bootstrap and
  an in-run transition for the helper to guess. Verified all three still
  produce a correct shop world with a real test before removing it.
- **`tooltips.rs` now uses the smooth fractional camera offset during a
  glide** (`components::camera_render_offset`), the same one
  `map_render`/`entity_render` already used for actual drawing, instead
  of the stale integer `Camera::left_x`/`top_y` - hovering an entity
  while the camera was visibly panning could point at the wrong tile (or
  none) for the ~150ms glide window. Needed a new
  `#[read_component(MovingAnimation)]` declaration too, since
  `camera_render_offset` reads that internally - verified this doesn't
  panic while a glide is genuinely in progress, though notably (unlike
  an earlier legion-access bug this project hit) it turns out NOT to
  panic without that declaration either in this schedule's own
  read-only batch - legion's single-entity `entry_ref` lookups aren't
  access-checked as strictly as bulk queries are. Kept the declaration
  anyway since it's correct and matches `entity_render.rs`/
  `map_render.rs`'s own convention for this exact same helper call.
- **Refactoring opportunities** — a read-through of the codebase looking
  specifically for what a refactor could improve, not a bug hunt.
  **Stage 1 (2026-09-13, branch `refactor-item-9-cleanup`): three
  duplication fixes** (find_player/reveal_and_freeze_fov/
  find_prefab_placement) **plus a broader comment-reduction pass** (this
  codebase ran ~39% comment lines to code; `main.rs`'s console-constant
  block alone carried ~400 lines of stale "was slot N, then M, then P"
  renumbering history its own text admitted was outdated - trimmed to
  the facts that matter, and the 45-pair per-frame `ctx.cls()` sweep
  collapsed into a loop over one `ALL_CONSOLES` array).
  **Stage 2 (2026-09-13, branch `refactor-item-9-stage2`): four
  structural splits, each verified with both headless class-
  survivability simulations after every logic-touching change:**
  - `main.rs` didn't follow its own established convention for where
    `State`'s methods live - Battle Arena's own orchestration (9
    methods: `start_arena`, `arena_begin_wave`,
    `arena_advance_to_next_shop`, `arena_spawn_boss_on_current_map`,
    `handle_arena_kill`, `arena_transition_tick`,
    `arena_wave_cleared_tick`, `boost_arena_enemy_fov`,
    `arena_rebuild_keep_player`) moved into a new `arena_state.rs`,
    matching every other screen's own convention.
  - Pure battle-resolution logic and battle rendering shared one file -
    `resolve_player_action`/`trigger_enemy_action`/
    `dismiss_action_result`/`record_enemy_kill`/`finish_battle` (plus
    the `ResultOutcome` enum they share) moved into a new
    `battle/resolve.rs`, alongside the module's existing damage/heal/
    dot/buff/counter/status/stun submodules.
  - `components.rs` (~2300 lines) was a grab-bag of several unrelated
    domains - split into `components/{mod,bars,animation,glide,
    tiles}.rs` by domain (plain data components, Ability/Battle/Item
    Bar slot logic, the ~1250-line sprite-sheet/animation-lookup
    system, camera/glide math, tile rendering).
  - `battle/mod.rs` (~1230 lines) had similarly distinguishable groups -
    split into `battle/{mod,menu,stats}.rs` (core Battle/combat
    resolution, menu/display concerns, entity-stat accessors).

  **Final piece (2026-09-13, same branch): `screens/battle.rs`'s
  `battle_tick`**, at ~735 lines by a wide margin the single largest
  function in the codebase, handled both rendering AND input for every
  `BattleTurn` state (`PlayerMenu`, `Filling`, `ActionResult` for both
  the player and each enemy) in one function. Unlike the four splits
  above, this wasn't a clean "move already-separate functions to a new
  file" - most of the length was shared per-frame preamble (timer
  ticking, ATB gauge fill/state-transition, arena+HUD rendering) that
  every state needs, with only the Actions-box/cursor-nav/match block
  actually being state-specific. Split into three new helper methods
  covering that shared preamble - `tick_battle_timers` (flash/popup/
  idle-frame/action-animation/dying-effects/hit-queue ticking),
  `tick_atb_and_maybe_act` (ATB fill and the Filling-state transition,
  returning early if that triggered an enemy action that itself ended
  the battle), `draw_battle_hud` (arena portraits, name/HP/ATB/status
  text, message log, damage popups) - leaving `battle_tick` itself as
  just the state-specific match block plus calls into the three
  helpers. Verified zero content loss via a sorted-line diff against
  the pre-split file, both headless class-survivability simulations
  still show the same patterns (Mage weakest, boss walls at L2/L3)
  post-split.

## Stats tracking

- Games played/won, enemies killed, deepest level reached, per-ability
  usage counts, and Battle Arena's own separated stats — all from prior
  sessions.

## Map render — stairs/counter rendering solid black while the camera was at rest

`TileType::Exit` (the dungeon stairs) and `TileType::Counter` (the shop
counter) - the only two tile types still on the old single-glyph
dungeonfont rendering rather than a real per-theme texture - rendered
solid black every time the camera was at rest, and rendered correctly
every time the camera was mid-pan. A real screen recording (not just a
screenshot) confirmed this precisely: tracked black-pixel count in the
tile's screen region against an independent background-motion detector
across all 121 frames, and the two flipped in lockstep across 7 separate
transitions - visible exactly when the camera was panning, black exactly
when it settled.

**A genuine recurrence of an already-fixed symptom, confirmed to be a
different cause.** An earlier session hit this exact same "only visible
while moving" symptom once before, from a real `Camera::bottom_y`
off-by-one (see journal.md) - fixed and merged well before this
recurrence. This time the camera math itself was fine (double-checked:
`camera.right_x`/`bottom_y` already span the full display exactly, no
missing edge row/column), and the actual cause traced all the way down
into bracket-terminal 0.8.7's own source - its `.wgsl` shaders for both
console types, the GPU vertex-buffer-building code, `FontScaler`'s UV
math - all identical for both the working (fancy console) and broken
(plain console) paths on paper. The literal reason bracket-lib's plain
console specifically failed for this glyph was never pinned down.

**Fix**: `map_render.rs` now always routes `Exit`/`Counter`/`Water`
tiles through the same "fancy" console (`MAP_SCROLL_CONSOLE`) already
used while the camera pans, unconditionally - at rest or panning alike -
instead of the plain console (console 0) that was failing. Console 0 is
no longer used by map rendering at all as a result. Floor/Wall's
real-texture rendering (`MAP_TILE_CONSOLE`/`MAP_TILE_SCROLL_CONSOLE`)
wasn't reported broken and keeps its original plain/fancy split.

## Victory/Defeat screens — real per-theme painted backgrounds

Implemented 2026-09-13 (added to the backlog 2026-09-11) - real background
images behind the played-once Death/Victory animations, the same "sourced
externally, one full painted scene" approach already proven for Battle
Arena's backgrounds (see "Battle arena backgrounds" in Done below).
`components::VictoryBackground`/`DefeatBackground` pick a background (+ for
Victory, a matching pose) keyed to the run's own `MapTheme::end_scene_theme()`
(Forest/Dungeon/Sewer) or Arena, sharing `resources/battle_backgrounds.png`'s
existing padded 6x6 glyph grid rather than any new console. Each dungeon
theme randomly picks between 2 Victory scenes; Defeat is one fixed scene per
theme/mode, no randomization. Full technical detail in `docs/journal.md`.

- **`VictoryPose::ClimbAway` now has a real animation** (delivered in
  the 2026-09-13 full animation batch alongside the Attack/Defend/
  Death/Idle_Battle_Stance/Walk/technique refresh for all 6 classes,
  4 boss Death animations, Goblin's walk fix, and the Shopkeeper's
  first-ever animation - see "Character & enemy animation" in Done
  for the full writeup) - north-east-facing, new rows 8-13 on
  `character_victory.png`.
- **The Amulet of Yala icon was removed from the Victory screen**
  (2026-09-13, explicit user call) - it was a small flat dungeonfont
  glyph designed for the old plain-fill background, and read as a
  mismatched artifact next to these painted scenes. The body text
  ("You put on the Amulet of Yala...") still carries that narrative
  beat on its own; the actual in-dungeon Amulet item/pickup mechanic
  is unaffected, only the End-screen icon is gone.
- **The hero's own position is now per-background**
  (`VictoryBackground::portrait_grid_position`/`walk_away_position`,
  components.rs) instead of one fixed spot for every scene - screenshot-
  verified across every Victory variant as of 2026-09-13 (Arena,
  Forest Stance, Dungeon Stance, Sewer Stance, Dungeon Corridor,
  Sewer Walk all confirmed good). Forest Stairs needed one real fix
  along the way: was row 4 (bottom), confirmed via screenshot sitting
  directly on top of "Press Enter..." - moved to row 3, matching
  every other foreground pose. The WalkAway pose (Dungeon Corridor/
  Sewer Walk) also needed a real fix, not just positioning: it drew
  at native 1x dungeon-tile size on a plain console and read as an
  almost-invisible speck - switched to `CHARACTER_IDLE_GLIDE_CONSOLE`
  (fancy, otherwise unused here) with a 4x `set_fancy` scale
  (`VICTORY_WALK_AWAY_SCALE`), confirmed via screenshot afterward.
  Defeat's fallen-portrait position stayed at its existing dead-center
  spot - every Defeat scene is a roughly-symmetric "centered focal
  point" composition, unlike Victory's much more varied set, so no
  per-background variation seemed needed there.
- **A real Defeat-screen positioning bug found via user screenshot and
  fixed (2026-09-13).** The death-animation branch of
  `draw_end_screen_fallen_portrait` still had the OLD fixed col=1
  left over from before the Amulet-of-Yala icon (which that offset
  used to leave room for) was removed - a real screenshot of a Debug-
  class Defeat on the Arena background showed the corpse sitting
  well off the courtyard's own centered staircase. Fixed with a new
  `DefeatBackground::portrait_grid_position` (currently the same
  centered value for all four scenes, kept as an explicit per-variant
  match for future flexibility).
- **The Defeat screen's own bottom text also needed fixing (2026-09-13,
  confirmed via screenshot).** "Don't worry, you can always try again
  with a new hero." is gone entirely - it wasn't earning its line, and
  sat at a row the fallen portrait could overlap depending on the
  active background. "Press Enter to return to the title screen."
  now sits at row 60, matching Victory's own equivalent line (the two
  screens used to put it in different places).
- **A text-legibility scrim was tried and reverted (2026-09-13).** A
  real problem: on a bright background (Forest Stairs' own archway
  light), the header/body text washed out completely, confirmed via
  screenshot. Traced bracket-lib's actual shader source to confirm why
  HUD_CONSOLE/BIG_TEXT_CONSOLE (both `_no_bg`) can't take a print-call
  backing color at all (their shader discards near-transparent glyph
  pixels outright, before `bg` ever matters), and added a translucent
  dark band behind the text via a "fancy" console instead (whose
  shader has no such discard, the same mechanism this project's own
  transparent-background trick already relies on) - technically
  correct, but the user disliked how it looked (a flat rectangle with
  hard edges, out of place against painted art) and asked for it
  gone until real UI art exists to do this properly - see item 10
  below. Reverted; the underlying legibility problem is untouched
  (still there on a bright-enough background) but accepted as a known
  gap for now rather than shipping a placeholder that reads as a bug.
- **Every background is now clean - watermark-free (2026-09-13,
  final pass).** The Dungeon theme needed the most regen attempts by
  far: Victory Dungeon "Stance" (vault, atlas glyph 7) came back
  watermarked three times in a row (a different mark each time)
  before a 4th attempt finally landed clean; Victory Dungeon
  "Corridor" (glyph 13) and Defeat Dungeon (glyph 9) each needed one
  regen. Sewer and Forest needed only one regen each across their
  Victory/Defeat art. All 11 backgrounds (7 Victory + 4 Defeat) are
  composited into `resources/battle_backgrounds.png` and confirmed
  watermark-free - nothing left on this front for this branch.
