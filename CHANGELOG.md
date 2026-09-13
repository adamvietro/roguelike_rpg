# Changelog

Notable player-facing changes to Ever Space RRPG, newest first. Dated
rather than versioned, since the project doesn't use version numbers -
see `docs/DEVLOG.md` for the detailed technical history and `docs/ideas.md`
for what's planned next.

## 2026-09-11

### Added
- **Every battle now has a full painted background** matching the
  dungeon's theme (Forest, Dungeon, or Sewer) instead of a flat tinted
  floor.
- **Attack and Defend now play real animations**, instead of just
  holding your battle stance.
- **Every class's battle techniques now have a real animation** -
  previously only one AOE move per class did; now nearly the whole
  roster does.
- **Enemies play a real attack animation when they hit you**, instead
  of just their ordinary battle stance.
- **All 7 out-of-combat abilities now play a real animation when used**:
  Ice Armor, Invisible Cloak, Stealth, Throw Spear, Trap, Freeze Trap,
  and Shoot.
- **Characters and enemies now actually face and walk the direction
  they're moving** (north/south/east/west), instead of only ever
  walking in place facing one way.

### Changed
- Multi-enemy fights position enemies in a staggered formation instead
  of a flat line, tuned per dungeon theme to clear its own background
  art.
- A combatant's hit-flash now lasts as long as their actual attack
  animation, instead of cutting off partway through a longer swing.
- A general art-quality pass across every animation added this update.

### Fixed
- **You'd only see the counter and the stairs while moving, not while
  standing still** - a real camera regression, now fixed.
- **Characters and enemies used to freeze in place for a moment on
  every single step**, instead of visibly walking - fixed as part of
  the new directional movement.
- The title screen's background enemies used to appear frozen instead
  of walking in place.
- The camera could show a black void near map edges.

## 2026-09-08

### Added
- **All 5 playable classes (and the hidden Debug class) now have real
  animated art** - a walking loop, a battle-screen pose, and a still
  portrait - replacing the old placeholder glyphs.
- **All 8 enemies now have their own real animated art** too.
- **Dungeon levels now have distinct visual themes** - Forest, Dungeon,
  and Sewer - each with its own floor/wall art and color palette.
- **Death and Victory now play a real animation for every class**,
  instead of a generic rotated-glyph effect.

### Fixed
- A hidden Class Select shortcut could drop you into the wrong game
  mode.

## 2026-09-07

### Added
- **Dungeon Crawl now has a real gold economy.** Enemies drop gold on
  every kill, same as Battle Arena always has.
- **A guaranteed treasure chest on every dungeon floor**, guarded by
  that floor's toughest ordinary enemy - open it for gold, a Dungeon
  Map, and a few Healing Potions all at once.
- **A shop between dungeon floors** - reach it via the stairs, spend
  your gold on Potions and Maps before heading to the next floor.
- Real art for the 5 AOE techniques (Whirlwind, Blizzard, Flurry,
  Javelin Volley, Arrow Volley) and the hidden Debug class's own
  portrait/Defeat icons.
- A small "x2"-style badge on Ability/Item/Battle Bar icons whenever
  you're carrying more than one.

### Changed
- Potions and Maps no longer litter dungeon floors at random - find
  them in the new chest or buy them at the new shop instead.
- Every class now starts with 3 Healing Potions instead of 1 (Barbarian
  previously started with none at all).
- Mage's Speed increased slightly (6 → 7).

### Fixed
- A dungeon floor's treasure chest room could generate with no way in -
  visible, but completely walled off.
- Standing next to an item in the new dungeon shop and pressing Enter
  could silently do nothing, even with enough gold.
- The mouse tooltip could briefly point at the wrong tile right after
  taking a step.

## 2026-09-06

### Added
- **Item Bar** (blue box, next to the Ability Bar) for Potions, Maps, and
  any other universal item - click an icon to use it directly, no need
  to open the Item Menu for a quick heal.
- **Click-to-use on the Ability Bar** - your Dungeon Abilities can now be
  clicked directly too, same effect as pressing the hotkey.
- **Buff icons on the player portrait** for active lasting effects (Ice
  Armor, Stealth, Invisible Cloak), so you always know what's currently
  affecting you without opening a menu.
- **Item Menu redesign** (`M`) - a full character dashboard instead of a
  potion list: Items, Equipped Items, Stats, Battle Actions, and Dungeon
  Actions, all in one screen, with a description panel that updates for
  whatever's selected.
- **Pause screen overhaul** - much bigger text, and a real arrow-key +
  Enter menu for Resume/Options/Quit (Esc still resumes instantly). Now
  cycles through a handful of gameplay tips while it's open.
- Real descriptions for all 15 weapons (previously blank).

### Changed
- Player health display: replaced the old full-width bar with a compact
  class-portrait + health bar in the top-left corner.
- The Battle Arena shop no longer lists every item in a corner list -
  stand next to what you want and its name/price shows up right next to
  you instead.
- The Stats screen (Item Menu) shows your Arena Level/Wave during a
  Battle Arena run, and your Dungeon Level otherwise.
- The old permanent "how to play" hint on the dungeon screen moved to
  the Pause screen's rotating tips instead.

### Fixed
- The Ability Bar/Battle Bar's red/green boxes could render with almost
  no gap on the right side for some icons, making them look like they
  were touching the border. Fixed for both bars.

## 2026-09-01 to 2026-09-05

### Added
- **Multi-enemy battles** - engage with more than one enemy at a time.
  Stack enemies onto a single tile and fight one to battle all of them,
  or use the new **Wait** hotkey (`Space`) to let nearby enemies close
  in on their own (up to 4 at once - be careful).
- **AOE attacks** for every class - hit every enemy in the fight at
  once, on top of each class's existing single-target options. Custom
  icons for these are still on the to-do list.
- **ATB (Active Time Battle) system** - a gauge under the player and
  each enemy fills based on their Speed; once it's full, that combatant
  can act.
- **Battle Speed and ATB Mode options** - Battle Speed controls how fast
  gauges fill; ATB Mode is Normal (gauges pause while you're choosing an
  attack) or Full ATB (gauges never stop - faster fights, but the enemy
  gets a lot more attacks in if you're slow to choose).
- Arrow-key + Enter selection for battle actions, alongside the existing
  number keys.

### Changed
- Advancing out of a battle result screen is now Enter specifically
  (was "any key") - lets you hold Enter down and it'll auto-queue your
  next action the moment you're able to act.
- The player's ability box border turns yellow when you're able to
  select an attack.

### Fixed
- Multi-hit attacks now show a separate damage number for each hit
  (previously only the last hit's number would show, even though the
  full damage was always being applied correctly).
