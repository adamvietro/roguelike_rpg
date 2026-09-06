# Ever Space RRPG — Ideas & Backlog

Working list of what's still ahead, and a running record of what's already
shipped. Pull individual "Working" items into a real session when ready to
build them — nothing there is scoped or scheduled just by being listed.

---

# Working

## Near-term priorities (carried over from the project instructions doc)

Roughly in the order we've been tackling them:

1. **Mouse-targeted ranged AOE outside battle** ("rain of fire" for
   ranged classes) — not started.
2. **Standing "fix issues with the battle system" bucket** — not a fixed
   list, just wherever ATB/multi-enemy turns up real bugs as they get
   more play (e.g. a fight ending mid-multi-hit-sequence, True ATB
   combined with 3–4 enemies at once).
3. **Rebindable hotkeys, beyond movement** — Potions next, then Maps,
   then most everything else eventually. Today Potion/Map use is still
   fixed to slot keys 1/2 by position, not a real `Action` binding
   (movement arrows are already rebindable via the Options screen).
4. **A shop for Potions/Maps in Dungeon Crawl mode** — pull them out of
   floor loot entirely, likely reusing a good chunk of the Battle Arena
   shop's existing pricing/stock/purchase code.
5. **Idle walk-in-place animation art** — the cycling infrastructure
   (`IdleAnimation` component) is built and genuinely cycling; every
   frame just points at the same placeholder glyph. Needs real distinct
   per-frame art, and maybe a move to a sprite sheet per class instead of
   cramming more cells into the one shared `dungeonfont.png`.
6. **Music & sound effects** — no crate picked yet (`rodio` is the
   leading candidate, since bracket-lib has no built-in audio support).
7. **Cleanup:** `arena_advance_to_next_shop` duplicates a chunk of
   `start_arena`'s shop-building code — not urgent, just flagged.
8. **Minor/cosmetic:** `tooltips.rs` still reads the old integer camera
   offset during a glide, instead of the smooth fractional one.

## Content / world

- **Dungeon Shop** — replace most dungeon floor items with a shop at the
  end of each floor. Needs mobs to drop gold first (Battle Arena already
  has a gold economy to borrow patterns from).
- **Chests** — findable in the dungeon, holding items; a way to keep some
  of the "find an item" feeling once floor-item drops move to the shop.
  Chests should be defended by enemies, not free loot.

## Item Menu
Work on an item menu that will allow a player to use items for now but later equip items.

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
  bigger multi-hit, which is now built).
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
- **Chain Lightning** — A multi-hit spell (Blizzard now fills this
  general niche, but a single-target chained version is still open).
- **Mana Shield** — An absorb/block effect, parallel to Ice Armor but
  reactive.
- **Arcane Missile** — Guaranteed hit, ignores evasion.
- **Meteor** — Skip a turn to wind up, then one big guaranteed hit.
- **Drain Life** — Damage plus self-heal in one action.

### Hunter
- **Multi-shot** — Bow's own multi-hit (Arrow Volley now covers the AOE
  version of this; a single-target multi-hit is still open).
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

- Each class has default Attack/Defend/Flee always available.
- Class-specific weapon tiers (swords, staffs, daggers, spears, bows),
  each tagged so the right tier drops for the right class.
- Class-specific starting kits so a run is never barefisted from turn one.
- Battle items/techniques won from battle loot are class-specific "cards."
- Debug class: hidden (`D` key from class select), 100 HP / 50 evasion /
  5 damage / 10 speed, with Victory/Defeat/Next Level instant-win/lose/
  skip items for testing.

## Combat system
- **ATB (Active Time Battle)** — replaced the old fixed-round system.
  Random gauge starts, Battle Speed setting (Slow/Normal/Fast), True ATB
  vs. Wait mode, queued player actions under Active mode, yellow border
  when the player can act.
- **Multi-enemy battles** — up to 4 enemies per fight, per-enemy gauges/
  statuses, highest-Speed auto-targeting with a `>` marker, per-enemy-
  count formation layouts, a "Wait" hotkey (Space) to gather nearby
  enemies into one fight.
- **AOE techniques** — one per class (Flurry, Whirlwind, Blizzard,
  Javelin Volley, Arrow Volley) exercising the multi-enemy system.
- Defense stat, Speed-based initiative (faster combatant acts first),
  Evasion/dodge, persistent Ice Armor, Counter Attack, damage-over-time
  (Rend/Burn/Poison/Garrote), stun/feint, buffs generalized into one
  `Buff`/`BuffKind` system (Evasion bonus, damage reduction).
- Real damage numbers in combat log (fixed a bug where messages showed
  pre-Defense-reduction numbers).
- 30% Defend success chance (was previously guaranteed).
- Boss per level, guaranteed boss loot, boss placed directly on the
  level's exit/amulet tile so the fight is unavoidable.
- Battle system refactored from one flat file into `battle/` category
  modules (damage, dot, stun, buff, counter, heal, status) for easier
  extension.
- **Multi-hit damage popup sequencing** — every hit in a multi-hit
   technique (AOE or single-target) currently resolves in one synchronous
   loop, so only the *last* hit's number ever shows as a floating popup.
   Real fix needs hits spread out over real time (a queued "wave" every
   ~150ms or so, similar in shape to how ATB gauges already tick), not
   just more popup slots.

## Battle screen / UI
- Bordered actions box, positioned beside the player portrait; full
  class roster shown with unowned techniques greyed out (not hidden).
- Battle Victory screen (shows loot instead of silently returning to the
  dungeon).
- Floating damage numbers, a persistent battle log (last 4 lines, not a
  single vanishing message), active-effect status lines (Defending, Ice
  Armor, Rend/Burn, Battle Cry, Countering) shown under each combatant's
  HP bar.
- Battle Item HUD: separate "Battle Attacks"/"Weapons" panels from the
  regular carried-items list, with hover tooltips.
- Theme-aware battle arena backgrounds (dungeon vs. forest palettes and
  scenery).
- Portrait hit feedback: color flash (Attacking/Hit), then a genuine
  attack wiggle once a transparent-background fancy console made motion
  possible without revealing a box edge.
- Fixed: bracket-lib's near-black-pixel culling bug worked around by
  flooring source art at RGB(16,16,16) — confirmed via the new sprite art
  batches.

## Battle Arena mode — feature-complete for a full playable loop
Adventure Select → starting shop → 3 levels of (5/5/3 waves + boss) each →
shop between levels → Victory. Real gold economy, separated stats
tracking, its own shop map/UI (walk up to an item, press Enter to buy),
a Shopkeeper NPC.

## Title / meta screens
- Title screen with a real decorative background (live map + wandering
  monsters, paced independently of real gameplay).
- Class Select (with per-class descriptions), Adventure Select
  (Dungeon Crawl vs. Battle Arena).
- Pause screen (Esc), Options screen (hotkey rebinding for movement,
  reachable from Pause or the title screen), History/Stats screen
  (win rate, per-class stats, ability-use breakdown).
- Bigger, better Victory and Game Over screens: tinted/dimmed real dungeon
  background, a fallen/rotated red portrait on defeat, hero + amulet icons
  on victory.

## Player / dungeon
- Auto-pickup (walking onto an item tile picks it up — no separate key).
- Fixed hotkeys: Potion is always slot 1, Map is always slot 2 (rather
  than compacting when one isn't carried).
- Red tint at low HP, grey tint while Invisible/Stealthed.
- Smooth camera scrolling: the world glides in lockstep with the player's
  own movement animation, instead of snapping the camera and animating
  the player separately.
- Idle walk-cycle animation infrastructure (see Working list above for
  the remaining art pass).
- Fixed: unreachable map regions from the Cellular Automata builder
  (now culled via the same reachability check Drunkard's Walk already
  used), verified across 200 generated seeds.
- Fixed: Amulet of Yala silently un-winnable after auto-pickup started
  grabbing it before the victory check could run.

## Sprite art
- Full character portraits for Amazon, Mage, Rogue, Hunter, and the
  Battle Arena Shopkeeper (transparent backgrounds, matching convention).
- All 18 ability icons across all 5 classes, plus the Shopkeeper — see
  `Dungeon_Font_Glyph_to_Cell_Map.md` for the authoritative glyph map.
- Spear glyph mapping corrected (uppercase X/Y/Z is now official).

## Engineering / refactors
- `main.rs` split from ~1,700 lines into a `screens/` module (title,
  pause, battle, end) plus `render_helpers.rs` — `main.rs` now only holds
  app bootstrap, `State` lifecycle, and the tick dispatcher.
- Battle logic split into `battle/` category modules (see Combat system
  above).
- Persistent settings/data: `keymap.ron`, `battle_speed.ron`,
  `atb_mode.ron`, `stats.ron`, all under the git-ignored `saves/`.
- `.cargo/config.toml` properly forcing X11 on WSL (was previously a
  dead, silently-ignored `[env]` block in `Cargo.toml`).
- Fortress-guaranteed weapon placement fixed (guard markers no longer
  feed into the general random spawn lottery).
- Cursor select for menus and battle systems.

## Stats tracking
- Games played/won (overall and per class), enemies killed, deepest
  level reached per class, and per-ability usage counts — all persisted
  and viewable from the History screen.
- Battle Arena tracks its own separated stats (highest level reached per
  class, runs completed).