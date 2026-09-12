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
5. **Animation work** — everything still outstanding now that the user is
   assembling a full new animation batch for every class and enemy. What
   already shipped (idle/walk art for all 6 classes and 8 enemies, the
   played-once Death/Victory/Technique framework, the title-screen fix,
   2026-09-11's Attack/Defend/full-technique-roster/enemy-attack batch)
   lives in "Character & enemy animation" in Done below - full row-
   mapping reference in `docs/Dungeon_Font_Glyph_to_Cell_Map.md`,
   session-by-session history in `docs/journal.md`, condensed standing
   rules in `CLAUDE.md`. Still open:
   - **Enemy Death animations.** An enemy currently just vanishes the
     instant it's killed - no animation at all, unlike the player's
     Death pose. Needs its own sheet (`enemy_death.png`?) and a moment to
     actually play it before removing the entity/awarding loot. The user
     is assembling these separately, **boss enemies only** (Goblin
     Chieftain, Orc Warlord, Ogre Warlord, Ettin Overlord) - the four
     basic enemies (Goblin, Orc, Ogre, Ettin) won't get one.
   - **Redo Amazon's Walk animation** - flagged as needing a fresh
     PixelLab pass, independent of the canvas-size/leftover-art bugs
     already fixed for it in an earlier session.
   - **Redo the enemy battle sprites** (`enemy_battle.png`, 9 rows) - a
     quality/style call, not a bug; the user isn't happy with how they
     look and wants another pass.
   - **More animations per class/enemy** (Breathing_Idle, directional
     rotations) - needs a facing-architecture design conversation first
     for anything beyond `south` (the game has no concept of entity
     facing at all today). Only a fraction of what each PixelLab zip
     actually contains is in use so far.
   - **Real moving animations, not just idle-in-place** - right now a
     character/enemy plays its idle loop whether standing still or
     actually walking between tiles; a genuine walk-cycle synced to
     movement is separate, not-yet-built work. Could cover all
     directions of movement, not just south, if needed - ties into the
     same facing-architecture question above.
   - **Enclosed-transparent-region review still needs a human pass.**
     2026-09-11's pixel-health scan (see "Character & enemy animation" in
     Done for the near-black/RGB-under-transparency half, now fixed) also
     flood-filled every frame for TRANSPARENT pixels not connected to the
     frame's own border - a real "hole" fully inside the silhouette,
     as opposed to the normal background. Found in ~39% of frames
     (607/1549) across almost every class/enemy, but a spot check showed
     most are legitimate negative space (the inside curve of Hunter's
     bow, gaps between limbs mid-stride) rather than defects - an
     automated fill would risk destroying real linework, so this was
     deliberately NOT auto-fixed. Needs an actual human look at the
     worst offenders (the biggest was `Hunter_v2/Arrow_Volley/east/
     frame_006.png`, 83 hole pixels) to separate genuine PixelLab
     segmentation defects from correct art. Also still needs the
     still-upcoming 4-directional Walk sheets (movement-animation work
     below) checked once they exist - not built yet.
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
8. **A refactor for how maps get made and tiles are set** (added
   2026-09-08) — supporting more than one tile set (see "Map tile
   themes" in Done below, merged 2026-09-08) already required some
   rethinking of map generation/tile assignment, but `map_builder`'s
   architects still bake in some single-tile-set assumptions worth
   revisiting once that's been lived with for a while.
9. **Refactoring opportunities** — a read-through of the codebase
    looking specifically for what a refactor could improve, not a bug
    hunt - nothing here is a correctness problem, and nothing here has
    been touched. Several are natural pairings (e.g. the two duplication
    items below rhyme with a shop-room/tooltip cleanup already done in an
    earlier session - these are the ones that were left behind).
    - **`arena_begin_wave` still duplicates the reveal-rectangle/
      frozen-FOV block** that `build_shop_room` just got extracted from.
      It builds a wave map, not a shop, so it was out of scope for that
      specific helper - but the "move the player, reveal a no-fog-of-war
      rectangle, freeze their FieldOfView" logic itself is identical code
      in both places. Worth pulling into its own smaller helper (e.g.
      `reveal_and_freeze_fov(&mut self, player, reveal_x, reveal_y,
      reveal_w, reveal_h)`) shared by `build_shop_room` and
      `arena_begin_wave` both.
    - **`apply_prefab` and `apply_chest` (`map_builder/prefab.rs`) share
      the same Dijkstra-based random-placement-attempt loop** (10
      attempts, the same 20.0/2000.0 distance thresholds, the same
      `monster_spawns.retain`) - `apply_chest`'s own doc comment already
      says outright that it "reuses the exact same...loop as
      apply_prefab." Only the part that actually stamps a template's
      characters onto the map differs between them (guard/weapon markers
      vs. guard/chest markers). A shared `find_prefab_placement(mb, rng,
      width, height) -> Option<Point>` helper would leave each function
      with just its own stamping logic.
    - **`main.rs` doesn't follow its own established convention for
      where `State`'s methods live.** Every dungeon/menu screen
      (`screens/pause.rs`, `screens/battle.rs`, `screens/item_menu.rs`,
      `screens/chest.rs`, ...) already adds its own methods to `State`
      from its own file - Rust privacy lets a descendant module see an
      ancestor's private fields, so this works with no `pub` needed.
      Battle Arena's own orchestration (`start_arena`, `arena_begin_wave`,
      `arena_advance_to_next_shop`, `arena_spawn_boss_on_current_map`,
      `handle_arena_kill`, `arena_transition_tick`,
      `arena_wave_cleared_tick`, `boost_arena_enemy_fov`,
      `arena_rebuild_keep_player` - nine methods) never got the same
      treatment and still lives directly in `main.rs`, which is now 1392
      lines partly because of it. Moving these into their own file (an
      `arena_state.rs`, say) would cut main.rs down to general State
      bootstrap/dispatch plus Dungeon Crawl's own two methods
      (`advance_level`, `dungeon_shop_transition`) - a much smaller, more
      focused file.
    - **`screens/battle.rs`'s `battle_tick` is about 735 lines** - by a
      wide margin the single largest function in the codebase - handling
      both rendering AND input for every `BattleTurn` state
      (`PlayerMenu`, `Filling`, `ActionResult` for both the player and
      each enemy) in one function. Worth splitting into one handler per
      state.
    - **Pure battle-resolution logic and battle rendering share one
      file** (`screens/battle.rs`). `resolve_player_action`/
      `trigger_enemy_action`/`dismiss_action_result`/`record_enemy_kill`/
      `finish_battle` never touch `ctx` at all - they're plain logic -
      while `battle_tick`/`draw_battle_arena`/`battle_victory_tick` are
      rendering-heavy. The headless class-survivability simulation needed
      exactly this split to exist (it calls the logic functions directly
      and can never call the rendering ones, which need a real window's
      console registry) - formalizing it into two files (e.g. a
      `battle/resolve.rs` for the logic half) would make that reuse
      pattern the obvious one instead of something that only worked
      because both happened to live in the same module.
    - **`components.rs` (1066 lines) is a grab-bag of several unrelated
      domains**, not really "components" in a narrow sense: plain data
      components (`Health`, `Gold`, `Speed`, ...), a genuine UI subsystem
      (`ability_bar_slots`/`battle_bar_slots`/`item_bar_slots`/
      `usable_menu_items`/`group_items`/`build_roster_slots` and friends
      - real algorithmic logic, not data), animation/camera math
      (`gliding_position`, `camera_render_offset`), and tile-rendering
      helpers (`tile_render_at`). Splitting by domain (e.g. a
      `components/bars.rs` for the UI-bar-slot logic alone) would make
      each piece easier to find and reason about independently.
    - **`battle/mod.rs` (1085 lines) has similarly distinguishable
      groups** worth splitting: entity-stat accessors
      (`entity_damage`/`entity_speed`/`entity_evasion`/`entity_health`/
      `carried_weapon_damage`/...), core combat resolution
      (`resolve_enemy_attack`/`apply_damage`/`apply_player_technique`/
      `tick_dot`/`heal_entity`), and menu/display concerns
      (`available_actions`/`action_name`/`MenuCursor`/`hp_bar_string`)
      all currently live in the one file.
    - **No shared "find the player" helper exists**, despite the same
      query shape (something like `<(Entity, &Point)>::query().filter(
      component::<Player>())`) being hand-rolled at 8+ call sites across
      systems/*.rs and main.rs. A real `buy_nearby_item` bug was exactly
      a missing `.filter(component::<Player>())` on one such hand-rolled
      query - a single shared `find_player(ecs) -> Option<(Entity,
      Point)>` (or similar) helper would make that whole class of mistake
      structurally impossible in new code, not just fixed in the one
      place it was actually found.
10. **Content / world**
    - **More winnable item variety** — right now a chest/shop can only
      ever contain Gold, a Dungeon Map, or a Healing Potion. Not scoped -
      could be equipment, trinkets, or anything else worth finding.
    - **Quest system** — not scoped at all yet: no design conversation
      has happened on objectives, tracking/UI, rewards, or whatever NPC/
      dialogue hook would hand them out. Worth a real design discussion
      (per CLAUDE.md's convention for architectural-sized changes) before
      any code gets written.
11. **Defeat and Victory screens need to be redone** (added 2026-09-11) —
    not scoped yet, no design conversation has happened on what "redone"
    means concretely (layout, new art, something else). They already have
    real played-once Death/Victory animations (see "Character & enemy
    animation" in Done), so this is about something beyond the animation
    itself - worth a real design conversation before touching code, per
    CLAUDE.md's convention for anything this size.
12. **Ability Bar/other HUD panels should go transparent when the player
    is underneath them** (added 2026-09-11) — a side effect of the camera
    changes: the player can now end up positioned under the Ability
    Bar/similar fixed UI panels, which currently just draw solid on top
    of them. Needs a design pass (which panels, "transparent" vs. "hide
    entirely," how to detect the player's screen-space position is
    actually under a given panel's cells) before touching code.
13. **The stairs and the shop counter still only render while the player
    is moving, not while standing still** (added 2026-09-11) — same
    SYMPTOM as an earlier session's real off-by-one bug (`Camera::
    bottom_y` clipping the bottom map row while stationary, only
    reappearing for the ~220ms of a glide via a different, bounds-check-
    free render path - see journal.md), which was fixed and merged to
    master. The user is reporting it's still happening (or happening
    again), so treat this as a fresh investigation rather than assuming
    the identical root cause - don't just re-apply the old fix without
    confirming what's actually clipping this time (could be a related
    but distinct off-by-one, e.g. a horizontal analogue, or a genuine
    regression from a later camera-adjacent change).

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
  overlap, but read as visually flat/robotic once seen live ("I dont
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
- **`TileType::Water`** exists in the data model, impassable and opaque
  with zero extra logic (same free ride `Counter` already got), but
  isn't placed by any generator yet - deliberate, targeted placement (a
  river, a lone obstacle) is its own deferred design pass.

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

## Stats tracking
- Games played/won, enemies killed, deepest level reached, per-ability
  usage counts, and Battle Arena's own separated stats — all from prior
  sessions.
