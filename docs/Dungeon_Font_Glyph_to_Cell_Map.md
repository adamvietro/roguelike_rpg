# Dungeon Font — Glyph-to-Cell Master Map
 
Source of truth: the original `dungeonfont(1).png` supplied for this project.
 
## Fixed atlas geometry
 
- Canvas: **512×512 PNG, RGBA**
- Grid: **16×16 cells**
- Cell size: **32×32 pixels**
- Pixel coordinates are **inclusive**: `(x0,y0)–(x1,y1)`.
- Column `c` starts at `x = c × 32`.
- Row `r` starts at `y = r × 32`.
The glyph order matches the **CP437 0–255 ordering** used by the original dungeon font atlas. The first 128 positions include the standard ASCII/control layout, with custom RPG artwork occupying some glyph cells.
 
## Session protocol (full rules in the Editing Rules doc)
 
- **Confirm the source file at session start.** Before editing, Claude must ask the user to confirm the sprite sheet supplied is the latest version, reflecting all prior edits — see Editing Rules Section 1A.
- **Reference images take precedence.** When the user supplies a reference image for a sprite, match it as closely as possible within the pixel-art constraints, rather than substituting a generic interpretation — see Editing Rules Section 36.
- **This file gets updated at session end.** Whenever a glyph mapping is added, finalized, or changed, this map and the Editing Rules doc are both updated and handed back to the user to replace in project knowledge — see Editing Rules Section 37.
- **Note on this update (Battle Arena session):** every reservation added in this update is a **codepoint reservation only** — no pixel editing of the actual PNG happened this session. Nothing below with "no art yet" in its Current content is safe to treat as finished; it's purely "this letter/symbol is now spoken for" bookkeeping so a future art pass knows which cells are claimed and by what.
## Current project replacements
 
| Glyph | Replacement                                                   |  Row |  Col | Pixel bounds        |
| ----- | ------------------------------------------------------------- | ---: | ---: | ------------------- |
| `0`   | Wooden Staff (Mage weapon tier 0)                             |    3 |    0 | (0,96)–(31,127)     |
| `1`   | Flame Staff (Mage weapon tier 1)                              |    3 |    1 | (32,96)–(63,127)    |
| `2`   | Winged Gem Staff (Mage weapon tier 2)                         |    3 |    2 | (64,96)–(95,127)    |
| `3`   | Dagger tier 0 (ruby rod style)                                |    3 |    3 | (96,96)–(127,127)   |
| `4`   | Dagger tier 1 (silver blade)                                  |    3 |    4 | (128,96)–(159,127)  |
| `5`   | Dagger tier 2 (gold ornate blade)                             |    3 |    5 | (160,96)–(191,127)  |
| `6`   | Bow tier 0                                                    |    3 |    6 | (192,96)–(223,127)  |
| `7`   | Bow tier 1                                                    |    3 |    7 | (224,96)–(255,127)  |
| `8`   | Bow tier 2 (gold, w/ sparkle flare)                           |    3 |    8 | (256,96)–(287,127)  |
| `X`   | Spear tier 0 (official mapping)                               |    5 |    8 | (256,160)–(287,191) |
| `Y`   | Spear tier 1 (tribal/tasseled, official mapping)              |    5 |    9 | (288,160)–(319,191) |
| `Z`   | Spear tier 2 (gold, custom spearhead added, official mapping) |    5 |   10 | (320,160)–(351,191) |
| `r`   | Rogue (hooded, glowing cyan eyes)                             |    7 |    2 | (64,224)–(95,255)   |
| `m`   | Mage (hooded, blue skin, red eyes)                            |    6 |   13 | (416,192)–(447,223) |
| `a`   | Amazon                                                        |    6 |    1 | (32,192)–(63,223)   |
| `B`   | Hunter                                                        |    4 |    2 | (64,128)–(95,159)   |
| `G`   | Goblin Chieftain (boss)                                       |    4 |    7 | (224,128)–(255,159) |
| `K`   | Orc Warlord (boss)                                            |    4 |   11 | (352,128)–(383,159) |
| `V`   | Ettin Overlord (boss)                                         |    5 |    6 | (192,160)–(223,191) |
| `T`   | Trap                                                          |    5 |    4 | (128,160)–(159,191) |
| `W`   | Battle Arena Shopkeeper (finalized, transparent background)   |    5 |    7 | (224,160)–(255,191) |
| `c`   | Treasure Chest (dungeon loot chest, finalized)                |    6 |    3 | (96,192)–(127,223)  |
 
## Battle Arena — icon reservations (this session)
 
New this session: the Battle Arena mode (see the project instructions doc,
Section 1/3) needed a shopkeeper icon and distinct icons for every player
ability, which previously all shared the placeholder glyph `?`. **All of
the following are codepoint reservations only — no pixel art has been
drawn for any of them yet.** They render as the plain default character
shape until a future art pass replaces them, exactly like any other
"standard glyph" cell.
 
`W` (Shopkeeper) was picked specifically because it was the one letter
already documented above as "free and unassigned" — every other letter in
the alphabet was already claimed by a class, enemy, or boss. It's no
longer free as of this session.
 
The 18 ability glyphs were picked from cells with no existing custom art,
favoring the ability's initial letter where a free one existed and falling
back to a related word or a loosely evocative symbol where it didn't
(most single letters were already spoken for by classes/enemies/bosses/
weapons by the time this pass happened). `Victory`/`Defeat`/`Next Level`
(Debug-class test items, not real player abilities) were deliberately left
on the shared `?` placeholder — not part of this reservation pass.
 
| Glyph | Reserved for    | Class     |  Row |  Col | Pixel bounds        |
| ----- | --------------- | --------- | ---: | ---: | ------------------- |
| `D`   | Deathblow       | Barbarian |    4 |    4 | (128,128)–(159,159) |
| `Q`   | Quick Attack    | Barbarian |    5 |    1 | (32,160)–(63,191)   |
| `C`   | Counter Attack  | Barbarian |    4 |    3 | (96,128)–(127,159)  |
| `R`   | Rend            | Barbarian |    5 |    2 | (64,160)–(95,191)   |
| `F`   | Fireball        | Mage      |    4 |    6 | (192,128)–(223,159) |
| `b`   | Burn            | Mage      |    6 |    2 | (64,192)–(95,223)   |
| `I`   | Invisible Cloak | Mage      |    4 |    9 | (288,128)–(319,159) |
| `A`   | Ice Armor       | Mage      |    4 |    1 | (32,128)–(63,159)   |
| `≡`   | Garrote         | Rogue     |   15 |    0 | (0,480)–(31,511)    |
| `d`   | Dodge           | Rogue     |    6 |    4 | (128,192)–(159,223) |
| `·`   | Stealth         | Rogue     |   15 |   10 | (320,480)–(351,511) |
| `J`   | Throw Spear     | Amazon    |    4 |   10 | (320,128)–(351,159) |
| `U`   | Battle Cry      | Amazon    |    5 |    5 | (160,160)–(191,191) |
| `P`   | Poison Spear    | Amazon    |    5 |    0 | (0,160)–(31,191)    |
| `H`   | Shoot           | Hunter    |    4 |    8 | (256,128)–(287,159) |
| `p`   | Poison Shot     | Hunter    |    7 |    0 | (0,224)–(31,255)    |
| `±`   | Stun            | Hunter    |   15 |    1 | (32,480)–(63,511)   |
| `f`   | Feint           | Hunter    |    6 |    6 | (192,192)–(223,223) |
 
## Ability icon art — all batches (this session, now complete)
 
**Batch 1** (Barbarian + Rogue) - 7 icons, all finalized:
 
| Glyph | Ability        | Class     |  Row |  Col |
| ----- | -------------- | --------- | ---: | ---: |
| `D`   | Deathblow      | Barbarian |    4 |    4 |
| `Q`   | Quick Attack   | Barbarian |    5 |    1 |
| `C`   | Counter Attack | Barbarian |    4 |    3 |
| `R`   | Rend           | Barbarian |    5 |    2 |
| `≡`   | Garrote        | Rogue     |   15 |    0 |
| `d`   | Dodge          | Rogue     |    6 |    4 |
| `·`   | Stealth        | Rogue     |   15 |   10 |
 
**Batch 2** (Mage + Amazon) - 6 of 7 finalized:
 
| Glyph | Ability      | Class  |  Row |  Col |
| ----- | ------------ | ------ | ---: | ---: |
| `F`   | Fireball     | Mage   |    4 |    6 |
| `b`   | Burn         | Mage   |    6 |    2 |
| `A`   | Ice Armor    | Mage   |    4 |    1 |
| `J`   | Throw Spear  | Amazon |    4 |   10 |
| `U`   | Battle Cry   | Amazon |    5 |    5 |
| `P`   | Poison Spear | Amazon |    5 |    0 |
 
`I` (Invisible Cloak, Mage) was blocked in Batch 2 (watermarked stock
reference) but finalized in Batch 3 below with a proper reference.
 
**Batch 3** (Hunter + Shopkeeper) - 5 of 6 finalized:
 
| Glyph | Ability/item            | Class  |  Row |  Col |
| ----- | ----------------------- | ------ | ---: | ---: |
| `I`   | Invisible Cloak         | Mage   |    4 |    9 |
| `H`   | Shoot                   | Hunter |    4 |    8 |
| `p`   | Poison Shot             | Hunter |    7 |    0 |
| `±`   | Stun                    | Hunter |   15 |    1 |
| `W`   | Battle Arena Shopkeeper | —      |    5 |    7 |
 
**Batch 4** (final) - 1 icon, plus a rework of Batch 3's Shopkeeper:
 
| Glyph | Ability | Class  |  Row |  Col |
| ----- | ------- | ------ | ---: | ---: |
| `f`   | Feint   | Hunter |    6 |    6 |
 
`f` (Feint) was blocked in Batch 3 (mismatched reference - a glowing
tree) but finalized here with the correct reference: an eye seen through
a scope/monocle with a crosshair, read as "watching for a tell" - a
reasonable fit for a misdirection ability.
 
The Shopkeeper (`W`) is the one full CHARACTER sprite in these batches
rather than a small ability icon - treated like this sheet's other class
portraits (Amazon/Mage/Rogue/Hunter/@knight): full body, filling the cell
top-to-bottom, NOT cropped to a face-only bust the way a smaller ability
icon would be. The source reference was a tall portrait-oriented image
(character occupying most of its height but with side padding); rather
than squish her into a square, the character's own tight bounding box was
cropped and padded back out to a square (top-anchored, not centered, so
the squished-thin option was avoided).
 
**Shopkeeper background corrected in Batch 4** - Batch 3's first pass
filled the padded square with the reference's own solid peach background
color, but Amazon/Mage/Rogue's own cells were checked afterward and
confirmed to use TRUE transparency (alpha 0/255 only, no background fill
at all) behind the character, not a solid color. Re-cropped from the
original reference: background keyed to transparent via a flood fill
starting from the image's own outer border (not a flat color-distance
threshold across the whole image, which was tried first and punched
accidental transparent holes into the character's own light-colored
glasses lenses/highlights - color alone can't tell "this pixel is part of
the disconnected exterior background" from "this pixel just happens to be
a similar color inside the character," but a fill that can only spread
from the outer edge inward can). Alpha snapped to a clean 0/255 split
after the resize (a plain resize of a transparent-background image can
leave faint semi-transparent fringe pixels around edges) to match
Amazon/Mage/Rogue's own convention exactly. Verified against a bright
green test backdrop - no halo, no holes.
 
All four batches followed the same process: reference cropped to its
actual content (decorative card borders/checker backgrounds in the
source images were NOT kept, since none of this sheet's existing
finished icons use that framed-card look; each icon's own themed
background fills its cell edge-to-edge instead, matching how the existing
weapon/character sprites already fill theirs), pixel-diffed against the
pre-batch sheet afterward to confirm only the intended cells changed
(confirmed clean all four times), and any large near-black regions
nudged away from true black (floor of RGB 10,10,10) per this project's
known bracket-lib black-pixel-culling quirk - needed for Batch 1's Dodge,
Batch 3's Invisible Cloak and Stun; not needed for Batch 2, the
Shopkeeper, or Feint.
 
**All 18 ability glyphs plus the Shopkeeper are now finalized - this
was the last one.** The "Battle Arena — icon reservations" table above is
now entirely historical (every glyph it lists is finalized) - see the
Complete 256-cell map below for each glyph's current, authoritative
status instead.
 
## Spear mapping correction (an earlier session)
 
A discrepancy was found between the docs and the actual sprite sheet:
 
- The docs previously listed lowercase `x, y, z` as the official Spear tier 0/1/2 mapping.
- The actual sheet had **two** spear-like sets: lowercase `w, x, y` (a gray/silver/purple design, with `z` never actually finished — it was still a plain letter glyph) **and** a separate, previously-undocumented uppercase `X, Y, Z` set (brown arrowhead design).
- The user decided: **uppercase `X, Y, Z` is now the official Spear tier 0/1/2 mapping**, going forward. Both were redesigned that session per new references (see table above).
- Lowercase `w, x, y` (the old orphaned spear art) have been **cleared to transparent**. Lowercase `z` remains an unmodified standard glyph.
## Important custom sprite cells in the original source
 
Codes 0–31 (the CP437 control-character icons and box-drawing arrows) have
been removed from this table - they were never customized for this
project and aren't targets for anything. They're still listed normally
in the Complete 256-cell map below, marked Unused.
 
| Code | Glyph | Current artwork/content                                    |  Row |  Col | Pixel bounds        |
| ---: | ----- | ---------------------------------------------------------- | ---: | ---: | ------------------- |
|   48 | `0`   | Wooden Staff (Mage tier 0)                                 |    3 |    0 | (0,96)–(31,127)     |
|   49 | `1`   | Flame Staff (Mage tier 1)                                  |    3 |    1 | (32,96)–(63,127)    |
|   50 | `2`   | Winged Gem Staff (Mage tier 2)                             |    3 |    2 | (64,96)–(95,127)    |
|   51 | `3`   | Dagger tier 0                                              |    3 |    3 | (96,96)–(127,127)   |
|   52 | `4`   | Dagger tier 1                                              |    3 |    4 | (128,96)–(159,127)  |
|   53 | `5`   | Dagger tier 2                                              |    3 |    5 | (160,96)–(191,127)  |
|   54 | `6`   | Bow tier 0                                                 |    3 |    6 | (192,96)–(223,127)  |
|   55 | `7`   | Bow tier 1                                                 |    3 |    7 | (224,96)–(255,127)  |
|   56 | `8`   | Bow tier 2 (gold, sparkle flare)                           |    3 |    8 | (256,96)–(287,127)  |
|   64 | `@`   | @ — current blue-armored knight/soldier sprite             |    4 |    0 | (0,128)–(31,159)    |
|   65 | `A`   | Ice Armor (Mage) (finalized)                               |    4 |    1 | (32,128)–(63,159)   |
|   66 | `B`   | Hunter (finalized)                                         |    4 |    2 | (64,128)–(95,159)   |
|   67 | `C`   | Counter Attack (Barbarian) (finalized)                     |    4 |    3 | (96,128)–(127,159)  |
|   68 | `D`   | Deathblow (Barbarian) (finalized)                          |    4 |    4 | (128,128)–(159,159) |
|   69 | `E`   | E — current barbarian/orc-like warrior sprite              |    4 |    5 | (160,128)–(191,159) |
|   70 | `F`   | Fireball (Mage) (finalized)                                |    4 |    6 | (192,128)–(223,159) |
|   71 | `G`   | G — assigned to Goblin Chieftain boss (custom art pending) |    4 |    7 | (224,128)–(255,159) |
|   72 | `H`   | Shoot (Hunter) (finalized)                                 |    4 |    8 | (256,128)–(287,159) |
|   73 | `I`   | Invisible Cloak (Mage) (finalized)                         |    4 |    9 | (288,128)–(319,159) |
|   74 | `J`   | Throw Spear (Amazon) (finalized)                           |    4 |   10 | (320,128)–(351,159) |
|   75 | `K`   | K — assigned to Orc Warlord boss (custom art pending)      |    4 |   11 | (352,128)–(383,159) |
|   79 | `O`   | O — current stone/golem-like monster sprite                |    4 |   15 | (480,128)–(511,159) |
|   80 | `P`   | Poison Spear (Amazon) (finalized)                          |    5 |    0 | (0,160)–(31,191)    |
|   81 | `Q`   | Quick Attack (Barbarian) (finalized)                       |    5 |    1 | (32,160)–(63,191)   |
|   82 | `R`   | Rend (Barbarian) (finalized)                               |    5 |    2 | (64,160)–(95,191)   |
|   83 | `S`   | S — current sword sprite                                   |    5 |    3 | (96,160)–(127,191)  |
|   84 | `T`   | T — assigned to Trap (custom art pending)                  |    5 |    4 | (128,160)–(159,191) |
|   85 | `U`   | Battle Cry (Amazon) (finalized)                            |    5 |    5 | (160,160)–(191,191) |
|   86 | `V`   | V — assigned to Ettin Overlord boss (custom art pending)   |    5 |    6 | (192,160)–(223,191) |
|   87 | `W`   | Battle Arena Shopkeeper (finalized)                        |    5 |    7 | (224,160)–(255,191) |
|   88 | `X`   | Spear tier 0 (finalized, official mapping)                 |    5 |    8 | (256,160)–(287,191) |
|   89 | `Y`   | Spear tier 1 (finalized, official mapping)                 |    5 |    9 | (288,160)–(319,191) |
|   90 | `Z`   | Spear tier 2 (finalized, official mapping)                 |    5 |   10 | (320,160)–(351,191) |
|   97 | `a`   | Amazon (finalized)                                         |    6 |    1 | (32,192)–(63,223)   |
|   98 | `b`   | Burn (Mage) (finalized)                                    |    6 |    2 | (64,192)–(95,223)   |
|  100 | `d`   | Dodge (Rogue) (finalized)                                  |    6 |    4 | (128,192)–(159,223) |
|  102 | `f`   | Feint (Hunter) (finalized)                                 |    6 |    6 | (192,192)–(223,223) |
|  103 | `g`   | g — current Goblin sprite                                  |    6 |    7 | (224,192)–(255,223) |
|  109 | `m`   | Mage (finalized)                                           |    6 |   13 | (416,192)–(447,223) |
|  111 | `o`   | o — current Orc sprite                                     |    6 |   15 | (480,192)–(511,223) |
|  112 | `p`   | Poison Shot (Hunter) (finalized)                           |    7 |    0 | (0,224)–(31,255)    |
|  114 | `r`   | Rogue (finalized)                                          |    7 |    2 | (64,224)–(95,255)   |
|  115 | `s`   | s — Rusty Sword (sword artwork)                            |    7 |    3 | (96,224)–(127,255)  |
|  119 | `w`   | Cleared to transparent (orphaned spear art removed)        |    7 |    7 | (224,224)–(255,255) |
|  120 | `x`   | Cleared to transparent (orphaned spear art removed)        |    7 |    8 | (256,224)–(287,255) |
|  121 | `y`   | Cleared to transparent (orphaned spear art removed)        |    7 |    9 | (288,224)–(319,255) |
|  122 | `z`   | Unmodified — standard letter glyph, not a spear            |    7 |   10 | (320,224)–(351,255) |
|  127 | `DEL` | DEL — custom triangle-like icon                            |    7 |   15 | (480,224)–(511,255) |
|  240 | `≡`   | Garrote (Rogue) (finalized)                                |   15 |    0 | (0,480)–(31,511)    |
|  241 | `±`   | Stun (Hunter) (finalized)                                  |   15 |    1 | (32,480)–(63,511)   |
|  250 | `·`   | Stealth (Rogue) (finalized)                                |   15 |   10 | (320,480)–(351,511) |
 
## Complete 256-cell map
 
| Code | Glyph   |  Row |  Col | Pixel bounds        | Current content                                                                     | Planned replacement |
| ---: | ------- | ---: | ---: | ------------------- | ----------------------------------------------------------------------------------- | ------------------- |
|    0 | `blank` |    0 |    0 | (0,0)–(31,31)       | Unused                                                                              |                     |
|    1 | `☺`     |    0 |    1 | (32,0)–(63,31)      | Unused                                                                              |                     |
|    2 | `☻`     |    0 |    2 | (64,0)–(95,31)      | Unused                                                                              |                     |
|    3 | `♥`     |    0 |    3 | (96,0)–(127,31)     | Unused                                                                              |                     |
|    4 | `♦`     |    0 |    4 | (128,0)–(159,31)    | Unused                                                                              |                     |
|    5 | `♣`     |    0 |    5 | (160,0)–(191,31)    | Unused                                                                              |                     |
|    6 | `♠`     |    0 |    6 | (192,0)–(223,31)    | Unused                                                                              |                     |
|    7 | `•`     |    0 |    7 | (224,0)–(255,31)    | Unused                                                                              |                     |
|    8 | `◘`     |    0 |    8 | (256,0)–(287,31)    | Unused                                                                              |                     |
|    9 | `○`     |    0 |    9 | (288,0)–(319,31)    | Unused                                                                              |                     |
|   10 | `◙`     |    0 |   10 | (320,0)–(351,31)    | Unused                                                                              |                     |
|   11 | `♂`     |    0 |   11 | (352,0)–(383,31)    | Unused                                                                              |                     |
|   12 | `♀`     |    0 |   12 | (384,0)–(415,31)    | Unused                                                                              |                     |
|   13 | `♪`     |    0 |   13 | (416,0)–(447,31)    | Unused                                                                              |                     |
|   14 | `♫`     |    0 |   14 | (448,0)–(479,31)    | Unused                                                                              |                     |
|   15 | `☼`     |    0 |   15 | (480,0)–(511,31)    | Unused                                                                              |                     |
|   16 | `►`     |    1 |    0 | (0,32)–(31,63)      | Unused                                                                              |                     |
|   17 | `◄`     |    1 |    1 | (32,32)–(63,63)     | Unused                                                                              |                     |
|   18 | `↕`     |    1 |    2 | (64,32)–(95,63)     | Unused                                                                              |                     |
|   19 | `‼`     |    1 |    3 | (96,32)–(127,63)    | Unused                                                                              |                     |
|   20 | `¶`     |    1 |    4 | (128,32)–(159,63)   | Unused                                                                              |                     |
|   21 | `§`     |    1 |    5 | (160,32)–(191,63)   | Unused                                                                              |                     |
|   22 | `▬`     |    1 |    6 | (192,32)–(223,63)   | Unused                                                                              |                     |
|   23 | `↨`     |    1 |    7 | (224,32)–(255,63)   | Unused                                                                              |                     |
|   24 | `↑`     |    1 |    8 | (256,32)–(287,63)   | Unused                                                                              |                     |
|   25 | `↓`     |    1 |    9 | (288,32)–(319,63)   | Unused                                                                              |                     |
|   26 | `→`     |    1 |   10 | (320,32)–(351,63)   | Unused                                                                              |                     |
|   27 | `←`     |    1 |   11 | (352,32)–(383,63)   | Unused                                                                              |                     |
|   28 | `∟`     |    1 |   12 | (384,32)–(415,63)   | Unused                                                                              |                     |
|   29 | `↔`     |    1 |   13 | (416,32)–(447,63)   | Unused                                                                              |                     |
|   30 | `▲`     |    1 |   14 | (448,32)–(479,63)   | Unused                                                                              |                     |
|   31 | `▼`     |    1 |   15 | (480,32)–(511,63)   | Unused                                                                              |                     |
|   32 | `SPACE` |    2 |    0 | (0,64)–(31,95)      | standard glyph                                                                      |                     |
|   33 | `!`     |    2 |    1 | (32,64)–(63,95)     | standard glyph                                                                      |                     |
|   34 | `"`     |    2 |    2 | (64,64)–(95,95)     | standard glyph                                                                      |                     |
|   35 | `#`     |    2 |    3 | (96,64)–(127,95)    | standard glyph                                                                      |                     |
|   36 | `$`     |    2 |    4 | (128,64)–(159,95)   | standard glyph                                                                      |                     |
|   37 | `%`     |    2 |    5 | (160,64)–(191,95)   | standard glyph                                                                      |                     |
|   38 | `&`     |    2 |    6 | (192,64)–(223,95)   | standard glyph                                                                      |                     |
|   39 | `'`     |    2 |    7 | (224,64)–(255,95)   | standard glyph                                                                      |                     |
|   40 | `(`     |    2 |    8 | (256,64)–(287,95)   | standard glyph                                                                      |                     |
|   41 | `)`     |    2 |    9 | (288,64)–(319,95)   | standard glyph                                                                      |                     |
|   42 | `*`     |    2 |   10 | (320,64)–(351,95)   | standard glyph                                                                      |                     |
|   43 | `+`     |    2 |   11 | (352,64)–(383,95)   | standard glyph                                                                      |                     |
|   44 | `,`     |    2 |   12 | (384,64)–(415,95)   | standard glyph                                                                      |                     |
|   45 | `-`     |    2 |   13 | (416,64)–(447,95)   | standard glyph                                                                      |                     |
|   46 | `.`     |    2 |   14 | (448,64)–(479,95)   | standard glyph                                                                      |                     |
|   47 | `/`     |    2 |   15 | (480,64)–(511,95)   | standard glyph                                                                      |                     |
|   48 | `0`     |    3 |    0 | (0,96)–(31,127)     | Wooden Staff (Mage tier 0)                                                          |                     |
|   49 | `1`     |    3 |    1 | (32,96)–(63,127)    | Flame Staff (Mage tier 1)                                                           |                     |
|   50 | `2`     |    3 |    2 | (64,96)–(95,127)    | Winged Gem Staff (Mage tier 2)                                                      |                     |
|   51 | `3`     |    3 |    3 | (96,96)–(127,127)   | Dagger tier 0                                                                       |                     |
|   52 | `4`     |    3 |    4 | (128,96)–(159,127)  | Dagger tier 1                                                                       |                     |
|   53 | `5`     |    3 |    5 | (160,96)–(191,127)  | Dagger tier 2                                                                       |                     |
|   54 | `6`     |    3 |    6 | (192,96)–(223,127)  | Bow tier 0                                                                          |                     |
|   55 | `7`     |    3 |    7 | (224,96)–(255,127)  | Bow tier 1                                                                          |                     |
|   56 | `8`     |    3 |    8 | (256,96)–(287,127)  | Bow tier 2 (sparkle flare)                                                          |                     |
|   57 | `9`     |    3 |    9 | (288,96)–(319,127)  | standard glyph                                                                      |                     |
|   58 | `:`     |    3 |   10 | (320,96)–(351,127)  | standard glyph                                                                      |                     |
|   59 | `;`     |    3 |   11 | (352,96)–(383,127)  | standard glyph                                                                      |                     |
|   60 | `<`     |    3 |   12 | (384,96)–(415,127)  | standard glyph                                                                      |                     |
|   61 | `=`     |    3 |   13 | (416,96)–(447,127)  | standard glyph                                                                      |                     |
|   62 | `>`     |    3 |   14 | (448,96)–(479,127)  | standard glyph                                                                      |                     |
|   63 | `?`     |    3 |   15 | (480,96)–(511,127)  | standard glyph (shared placeholder for un-iconed items - see Complete Ability list) |                     |
|   64 | `@`     |    4 |    0 | (0,128)–(31,159)    | @ — current blue-armored knight/soldier sprite                                      |                     |
|   65 | `A`     |    4 |    1 | (32,128)–(63,159)   | Ice Armor (Mage) (finalized)                                                        |                     |
|   66 | `B`     |    4 |    2 | (64,128)–(95,159)   | Hunter (finalized)                                                                  |                     |
|   67 | `C`     |    4 |    3 | (96,128)–(127,159)  | Counter Attack (Barbarian) (finalized)                                              |                     |
|   68 | `D`     |    4 |    4 | (128,128)–(159,159) | Deathblow (Barbarian) (finalized)                                                   |                     |
|   69 | `E`     |    4 |    5 | (160,128)–(191,159) | E — current barbarian/orc-like warrior sprite                                       |                     |
|   70 | `F`     |    4 |    6 | (192,128)–(223,159) | Fireball (Mage) (finalized)                                                         |                     |
|   71 | `G`     |    4 |    7 | (224,128)–(255,159) | Assigned: Goblin Chieftain boss (custom art pending)                                |                     |
|   72 | `H`     |    4 |    8 | (256,128)–(287,159) | Shoot (Hunter) (finalized)                                                          |                     |
|   73 | `I`     |    4 |    9 | (288,128)–(319,159) | Invisible Cloak (Mage) (finalized)                                                  |                     |
|   74 | `J`     |    4 |   10 | (320,128)–(351,159) | Throw Spear (Amazon) (finalized)                                                    |                     |
|   75 | `K`     |    4 |   11 | (352,128)–(383,159) | Assigned: Orc Warlord boss (custom art pending)                                     |                     |
|   76 | `L`     |    4 |   12 | (384,128)–(415,159) | standard glyph                                                                      |                     |
|   77 | `M`     |    4 |   13 | (416,128)–(447,159) | standard glyph                                                                      |                     |
|   78 | `N`     |    4 |   14 | (448,128)–(479,159) | standard glyph                                                                      |                     |
|   79 | `O`     |    4 |   15 | (480,128)–(511,159) | O — current stone/golem-like monster sprite                                         |                     |
|   80 | `P`     |    5 |    0 | (0,160)–(31,191)    | Poison Spear (Amazon) (finalized)                                                   |                     |
|   81 | `Q`     |    5 |    1 | (32,160)–(63,191)   | Quick Attack (Barbarian) (finalized)                                                |                     |
|   82 | `R`     |    5 |    2 | (64,160)–(95,191)   | Rend (Barbarian) (finalized)                                                        |                     |
|   83 | `S`     |    5 |    3 | (96,160)–(127,191)  | S — current sword sprite                                                            |                     |
|   84 | `T`     |    5 |    4 | (128,160)–(159,191) | Assigned: Trap (custom art pending)                                                 |                     |
|   85 | `U`     |    5 |    5 | (160,160)–(191,191) | Battle Cry (Amazon) (finalized)                                                     |                     |
|   86 | `V`     |    5 |    6 | (192,160)–(223,191) | Assigned: Ettin Overlord boss (custom art pending)                                  |                     |
|   87 | `W`     |    5 |    7 | (224,160)–(255,191) | Battle Arena Shopkeeper (finalized)                                                 |                     |
|   88 | `X`     |    5 |    8 | (256,160)–(287,191) | Spear tier 0 (finalized, official mapping)                                          |                     |
|   89 | `Y`     |    5 |    9 | (288,160)–(319,191) | Spear tier 1 (finalized, official mapping)                                          |                     |
|   90 | `Z`     |    5 |   10 | (320,160)–(351,191) | Spear tier 2 (finalized, official mapping)                                          |                     |
|   91 | `[`     |    5 |   11 | (352,160)–(383,191) | standard glyph                                                                      |                     |
|   92 | `\`     |    5 |   12 | (384,160)–(415,191) | standard glyph                                                                      |                     |
|   93 | `]`     |    5 |   13 | (416,160)–(447,191) | standard glyph                                                                      |                     |
|   94 | `^`     |    5 |   14 | (448,160)–(479,191) | standard glyph                                                                      |                     |
|   95 | `_`     |    5 |   15 | (480,160)–(511,191) | standard glyph                                                                      |                     |
|   96 | ```     |    6 |    0 | (0,192)–(31,223)    | standard glyph                                                                      |                     |
|   97 | `a`     |    6 |    1 | (32,192)–(63,223)   | Amazon (finalized)                                                                  |                     |
|   98 | `b`     |    6 |    2 | (64,192)–(95,223)   | Burn (Mage) (finalized)                                                             |                     |
|   99 | `c`     |    6 |    3 | (96,192)–(127,223)  | Treasure Chest (dungeon loot chest, finalized)                                      |                     |
|  100 | `d`     |    6 |    4 | (128,192)–(159,223) | Dodge (Rogue) (finalized)                                                           |                     |
|  101 | `e`     |    6 |    5 | (160,192)–(191,223) | standard glyph                                                                      |                     |
|  102 | `f`     |    6 |    6 | (192,192)–(223,223) | Feint (Hunter) (finalized)                                                          |                     |
|  103 | `g`     |    6 |    7 | (224,192)–(255,223) | g — current Goblin sprite                                                           |                     |
|  104 | `h`     |    6 |    8 | (256,192)–(287,223) | standard glyph                                                                      |                     |
|  105 | `i`     |    6 |    9 | (288,192)–(319,223) | standard glyph                                                                      |                     |
|  106 | `j`     |    6 |   10 | (320,192)–(351,223) | standard glyph                                                                      |                     |
|  107 | `k`     |    6 |   11 | (352,192)–(383,223) | standard glyph                                                                      |                     |
|  108 | `l`     |    6 |   12 | (384,192)–(415,223) | standard glyph                                                                      |                     |
|  109 | `m`     |    6 |   13 | (416,192)–(447,223) | Mage (finalized)                                                                    |                     |
|  110 | `n`     |    6 |   14 | (448,192)–(479,223) | standard glyph                                                                      |                     |
|  111 | `o`     |    6 |   15 | (480,192)–(511,223) | o — current Orc sprite                                                              |                     |
|  112 | `p`     |    7 |    0 | (0,224)–(31,255)    | Poison Shot (Hunter) (finalized)                                                    |                     |
|  113 | `q`     |    7 |    1 | (32,224)–(63,255)   | standard glyph                                                                      |                     |
|  114 | `r`     |    7 |    2 | (64,224)–(95,255)   | Rogue (finalized)                                                                   |                     |
|  115 | `s`     |    7 |    3 | (96,224)–(127,255)  | s — Rusty Sword                                                                     |                     |
|  116 | `t`     |    7 |    4 | (128,224)–(159,255) | standard glyph                                                                      |                     |
|  117 | `u`     |    7 |    5 | (160,224)–(191,255) | standard glyph                                                                      |                     |
|  118 | `v`     |    7 |    6 | (192,224)–(223,255) | standard glyph                                                                      |                     |
|  119 | `w`     |    7 |    7 | (224,224)–(255,255) | Cleared to transparent (orphaned spear art removed)                                 |                     |
|  120 | `x`     |    7 |    8 | (256,224)–(287,255) | Cleared to transparent (orphaned spear art removed)                                 |                     |
|  121 | `y`     |    7 |    9 | (288,224)–(319,255) | Cleared to transparent (orphaned spear art removed)                                 |                     |
|  122 | `z`     |    7 |   10 | (320,224)–(351,255) | standard glyph (unmodified, not a spear)                                            |                     |
|  123 | `{`     |    7 |   11 | (352,224)–(383,255) | standard glyph                                                                      |                     |
|  124 | `       |    ` |    7 | 12                  | (384,224)–(415,255)                                                                 | standard glyph      |  |
|  125 | `}`     |    7 |   13 | (416,224)–(447,255) | standard glyph                                                                      |                     |
|  126 | `~`     |    7 |   14 | (448,224)–(479,255) | standard glyph                                                                      |                     |
|  127 | `DEL`   |    7 |   15 | (480,224)–(511,255) | DEL — custom triangle-like icon                                                     |                     |
|  128 | `Ç`     |    8 |    0 | (0,256)–(31,287)    | standard glyph                                                                      |                     |
|  129 | `ü`     |    8 |    1 | (32,256)–(63,287)   | standard glyph                                                                      |                     |
|  130 | `é`     |    8 |    2 | (64,256)–(95,287)   | standard glyph                                                                      |                     |
|  131 | `â`     |    8 |    3 | (96,256)–(127,287)  | standard glyph                                                                      |                     |
|  132 | `ä`     |    8 |    4 | (128,256)–(159,287) | standard glyph                                                                      |                     |
|  133 | `à`     |    8 |    5 | (160,256)–(191,287) | standard glyph                                                                      |                     |
|  134 | `å`     |    8 |    6 | (192,256)–(223,287) | standard glyph                                                                      |                     |
|  135 | `ç`     |    8 |    7 | (224,256)–(255,287) | standard glyph                                                                      |                     |
|  136 | `ê`     |    8 |    8 | (256,256)–(287,287) | standard glyph                                                                      |                     |
|  137 | `ë`     |    8 |    9 | (288,256)–(319,287) | standard glyph                                                                      |                     |
|  138 | `è`     |    8 |   10 | (320,256)–(351,287) | standard glyph                                                                      |                     |
|  139 | `ï`     |    8 |   11 | (352,256)–(383,287) | standard glyph                                                                      |                     |
|  140 | `î`     |    8 |   12 | (384,256)–(415,287) | standard glyph                                                                      |                     |
|  141 | `ì`     |    8 |   13 | (416,256)–(447,287) | standard glyph                                                                      |                     |
|  142 | `Ä`     |    8 |   14 | (448,256)–(479,287) | standard glyph                                                                      |                     |
|  143 | `Å`     |    8 |   15 | (480,256)–(511,287) | standard glyph                                                                      |                     |
|  144 | `É`     |    9 |    0 | (0,288)–(31,319)    | standard glyph                                                                      |                     |
|  145 | `æ`     |    9 |    1 | (32,288)–(63,319)   | standard glyph                                                                      |                     |
|  146 | `Æ`     |    9 |    2 | (64,288)–(95,319)   | standard glyph                                                                      |                     |
|  147 | `ô`     |    9 |    3 | (96,288)–(127,319)  | standard glyph                                                                      |                     |
|  148 | `ö`     |    9 |    4 | (128,288)–(159,319) | standard glyph                                                                      |                     |
|  149 | `ò`     |    9 |    5 | (160,288)–(191,319) | standard glyph                                                                      |                     |
|  150 | `û`     |    9 |    6 | (192,288)–(223,319) | standard glyph                                                                      |                     |
|  151 | `ù`     |    9 |    7 | (224,288)–(255,319) | standard glyph                                                                      |                     |
|  152 | `ÿ`     |    9 |    8 | (256,288)–(287,319) | standard glyph                                                                      |                     |
|  153 | `Ö`     |    9 |    9 | (288,288)–(319,319) | standard glyph                                                                      |                     |
|  154 | `Ü`     |    9 |   10 | (320,288)–(351,319) | standard glyph                                                                      |                     |
|  155 | `¢`     |    9 |   11 | (352,288)–(383,319) | standard glyph                                                                      |                     |
|  156 | `£`     |    9 |   12 | (384,288)–(415,319) | standard glyph                                                                      |                     |
|  157 | `¥`     |    9 |   13 | (416,288)–(447,319) | standard glyph                                                                      |                     |
|  158 | `₧`     |    9 |   14 | (448,288)–(479,319) | standard glyph                                                                      |                     |
|  159 | `ƒ`     |    9 |   15 | (480,288)–(511,319) | standard glyph                                                                      |                     |
|  160 | `á`     |   10 |    0 | (0,320)–(31,351)    | standard glyph                                                                      |                     |
|  161 | `í`     |   10 |    1 | (32,320)–(63,351)   | standard glyph                                                                      |                     |
|  162 | `ó`     |   10 |    2 | (64,320)–(95,351)   | standard glyph                                                                      |                     |
|  163 | `ú`     |   10 |    3 | (96,320)–(127,351)  | standard glyph                                                                      |                     |
|  164 | `ñ`     |   10 |    4 | (128,320)–(159,351) | standard glyph                                                                      |                     |
|  165 | `Ñ`     |   10 |    5 | (160,320)–(191,351) | standard glyph                                                                      |                     |
|  166 | `ª`     |   10 |    6 | (192,320)–(223,351) | standard glyph                                                                      |                     |
|  167 | `º`     |   10 |    7 | (224,320)–(255,351) | standard glyph                                                                      |                     |
|  168 | `¿`     |   10 |    8 | (256,320)–(287,351) | standard glyph                                                                      |                     |
|  169 | `⌐`     |   10 |    9 | (288,320)–(319,351) | standard glyph                                                                      |                     |
|  170 | `¬`     |   10 |   10 | (320,320)–(351,351) | standard glyph                                                                      |                     |
|  171 | `½`     |   10 |   11 | (352,320)–(383,351) | standard glyph                                                                      |                     |
|  172 | `¼`     |   10 |   12 | (384,320)–(415,351) | standard glyph                                                                      |                     |
|  173 | `¡`     |   10 |   13 | (416,320)–(447,351) | standard glyph                                                                      |                     |
|  174 | `«`     |   10 |   14 | (448,320)–(479,351) | standard glyph                                                                      |                     |
|  175 | `»`     |   10 |   15 | (480,320)–(511,351) | standard glyph                                                                      |                     |
|  176 | `░`     |   11 |    0 | (0,352)–(31,383)    | standard glyph                                                                      |                     |
|  177 | `▒`     |   11 |    1 | (32,352)–(63,383)   | standard glyph                                                                      |                     |
|  178 | `▓`     |   11 |    2 | (64,352)–(95,383)   | standard glyph                                                                      |                     |
|  179 | `│`     |   11 |    3 | (96,352)–(127,383)  | standard glyph                                                                      |                     |
|  180 | `┤`     |   11 |    4 | (128,352)–(159,383) | standard glyph                                                                      |                     |
|  181 | `╡`     |   11 |    5 | (160,352)–(191,383) | standard glyph                                                                      |                     |
|  182 | `╢`     |   11 |    6 | (192,352)–(223,383) | standard glyph                                                                      |                     |
|  183 | `╖`     |   11 |    7 | (224,352)–(255,383) | standard glyph                                                                      |                     |
|  184 | `╕`     |   11 |    8 | (256,352)–(287,383) | standard glyph                                                                      |                     |
|  185 | `╣`     |   11 |    9 | (288,352)–(319,383) | standard glyph                                                                      |                     |
|  186 | `║`     |   11 |   10 | (320,352)–(351,383) | standard glyph                                                                      |                     |
|  187 | `╗`     |   11 |   11 | (352,352)–(383,383) | standard glyph                                                                      |                     |
|  188 | `╝`     |   11 |   12 | (384,352)–(415,383) | standard glyph                                                                      |                     |
|  189 | `╜`     |   11 |   13 | (416,352)–(447,383) | standard glyph                                                                      |                     |
|  190 | `╛`     |   11 |   14 | (448,352)–(479,383) | standard glyph                                                                      |                     |
|  191 | `┐`     |   11 |   15 | (480,352)–(511,383) | standard glyph                                                                      |                     |
|  192 | `└`     |   12 |    0 | (0,384)–(31,415)    | standard glyph                                                                      |                     |
|  193 | `┴`     |   12 |    1 | (32,384)–(63,415)   | standard glyph                                                                      |                     |
|  194 | `┬`     |   12 |    2 | (64,384)–(95,415)   | standard glyph                                                                      |                     |
|  195 | `├`     |   12 |    3 | (96,384)–(127,415)  | standard glyph                                                                      |                     |
|  196 | `─`     |   12 |    4 | (128,384)–(159,415) | standard glyph                                                                      |                     |
|  197 | `┼`     |   12 |    5 | (160,384)–(191,415) | standard glyph                                                                      |                     |
|  198 | `╞`     |   12 |    6 | (192,384)–(223,415) | standard glyph                                                                      |                     |
|  199 | `╟`     |   12 |    7 | (224,384)–(255,415) | standard glyph                                                                      |                     |
|  200 | `╚`     |   12 |    8 | (256,384)–(287,415) | standard glyph                                                                      |                     |
|  201 | `╔`     |   12 |    9 | (288,384)–(319,415) | standard glyph                                                                      |                     |
|  202 | `╩`     |   12 |   10 | (320,384)–(351,415) | standard glyph                                                                      |                     |
|  203 | `╦`     |   12 |   11 | (352,384)–(383,415) | standard glyph                                                                      |                     |
|  204 | `╠`     |   12 |   12 | (384,384)–(415,415) | standard glyph                                                                      |                     |
|  205 | `═`     |   12 |   13 | (416,384)–(447,415) | standard glyph                                                                      |                     |
|  206 | `╬`     |   12 |   14 | (448,384)–(479,415) | standard glyph                                                                      |                     |
|  207 | `╧`     |   12 |   15 | (480,384)–(511,415) | standard glyph                                                                      |                     |
|  208 | `╨`     |   13 |    0 | (0,416)–(31,447)    | standard glyph                                                                      |                     |
|  209 | `╤`     |   13 |    1 | (32,416)–(63,447)   | standard glyph                                                                      |                     |
|  210 | `╥`     |   13 |    2 | (64,416)–(95,447)   | standard glyph                                                                      |                     |
|  211 | `╙`     |   13 |    3 | (96,416)–(127,447)  | standard glyph                                                                      |                     |
|  212 | `╘`     |   13 |    4 | (128,416)–(159,447) | standard glyph                                                                      |                     |
|  213 | `╒`     |   13 |    5 | (160,416)–(191,447) | standard glyph                                                                      |                     |
|  214 | `╓`     |   13 |    6 | (192,416)–(223,447) | standard glyph                                                                      |                     |
|  215 | `╫`     |   13 |    7 | (224,416)–(255,447) | standard glyph                                                                      |                     |
|  216 | `╪`     |   13 |    8 | (256,416)–(287,447) | standard glyph                                                                      |                     |
|  217 | `┘`     |   13 |    9 | (288,416)–(319,447) | standard glyph                                                                      |                     |
|  218 | `┌`     |   13 |   10 | (320,416)–(351,447) | standard glyph                                                                      |                     |
|  219 | `█`     |   13 |   11 | (352,416)–(383,447) | standard glyph                                                                      |                     |
|  220 | `▄`     |   13 |   12 | (384,416)–(415,447) | standard glyph                                                                      |                     |
|  221 | `▌`     |   13 |   13 | (416,416)–(447,447) | standard glyph                                                                      |                     |
|  222 | `▐`     |   13 |   14 | (448,416)–(479,447) | standard glyph                                                                      |                     |
|  223 | `▀`     |   13 |   15 | (480,416)–(511,447) | standard glyph                                                                      |                     |
|  224 | `α`     |   14 |    0 | (0,448)–(31,479)    | standard glyph                                                                      |                     |
|  225 | `ß`     |   14 |    1 | (32,448)–(63,479)   | standard glyph                                                                      |                     |
|  226 | `Γ`     |   14 |    2 | (64,448)–(95,479)   | standard glyph                                                                      |                     |
|  227 | `π`     |   14 |    3 | (96,448)–(127,479)  | standard glyph                                                                      |                     |
|  228 | `Σ`     |   14 |    4 | (128,448)–(159,479) | standard glyph                                                                      |                     |
|  229 | `σ`     |   14 |    5 | (160,448)–(191,479) | standard glyph                                                                      |                     |
|  230 | `µ`     |   14 |    6 | (192,448)–(223,479) | standard glyph                                                                      |                     |
|  231 | `τ`     |   14 |    7 | (224,448)–(255,479) | standard glyph                                                                      |                     |
|  232 | `Φ`     |   14 |    8 | (256,448)–(287,479) | standard glyph                                                                      |                     |
|  233 | `Θ`     |   14 |    9 | (288,448)–(319,479) | standard glyph                                                                      |                     |
|  234 | `Ω`     |   14 |   10 | (320,448)–(351,479) | standard glyph                                                                      |                     |
|  235 | `δ`     |   14 |   11 | (352,448)–(383,479) | standard glyph                                                                      |                     |
|  236 | `∞`     |   14 |   12 | (384,448)–(415,479) | standard glyph                                                                      |                     |
|  237 | `φ`     |   14 |   13 | (416,448)–(447,479) | standard glyph                                                                      |                     |
|  238 | `ε`     |   14 |   14 | (448,448)–(479,479) | standard glyph                                                                      |                     |
|  239 | `∩`     |   14 |   15 | (480,448)–(511,479) | standard glyph                                                                      |                     |
|  240 | `≡`     |   15 |    0 | (0,480)–(31,511)    | Garrote (Rogue) (finalized)                                                         |                     |
|  241 | `±`     |   15 |    1 | (32,480)–(63,511)   | Stun (Hunter) (finalized)                                                           |                     |
|  242 | `≥`     |   15 |    2 | (64,480)–(95,511)   | standard glyph                                                                      |                     |
|  243 | `≤`     |   15 |    3 | (96,480)–(127,511)  | standard glyph                                                                      |                     |
|  244 | `⌠`     |   15 |    4 | (128,480)–(159,511) | standard glyph                                                                      |                     |
|  245 | `⌡`     |   15 |    5 | (160,480)–(191,511) | standard glyph                                                                      |                     |
|  246 | `÷`     |   15 |    6 | (192,480)–(223,511) | standard glyph                                                                      |                     |
|  247 | `≈`     |   15 |    7 | (224,480)–(255,511) | standard glyph                                                                      |                     |
|  248 | `°`     |   15 |    8 | (256,480)–(287,511) | standard glyph                                                                      |                     |
|  249 | `∙`     |   15 |    9 | (288,480)–(319,511) | standard glyph                                                                      |                     |
|  250 | `·`     |   15 |   10 | (320,480)–(351,511) | Stealth (Rogue) (finalized)                                                         |                     |
|  251 | `√`     |   15 |   11 | (352,480)–(383,511) | standard glyph                                                                      |                     |
|  252 | `ⁿ`     |   15 |   12 | (384,480)–(415,511) | standard glyph                                                                      |                     |
|  253 | `²`     |   15 |   13 | (416,480)–(447,511) | standard glyph                                                                      |                     |
|  254 | `■`     |   15 |   14 | (448,480)–(479,511) | standard glyph                                                                      |                     |
|  255 | ` `     |   15 |   15 | (480,480)–(511,511) | standard glyph                                                                      |                     |
 
## Boss glyph status
 
- Goblin Chieftain: **`G`** (finalized)
- Orc Warlord: **`K`** (finalized - moved off `W` after it read as visually
  identical to lowercase `w`, which was the Wooden Spear glyph at the time)
- Ettin Overlord: **`V`** (finalized - moved off `X` at the time; `X` has
  since been redesigned and is now the official Spear tier 0 glyph)
- Trap: **`T`** (finalized)
- Battle Arena Shopkeeper: **`W`** (finalized this session - see
  "Ability icon art — batches so far" above)
- Lowercase `w`, `x`, `y` are now empty/transparent (cleared this session
  — see "Spear mapping correction" above). Lowercase `z` is an unmodified
  standard glyph, not a spear.
- Uppercase `X`, `Y`, `Z` are the official, code-assigned Spear tier
  0/1/2 glyphs.
- **`W` is no longer free/unassigned** — it was previously listed here as
  available for future use, but the Battle Arena session claimed it for
  the Shopkeeper (see above). Any future "what's still free" scan should
  no longer include it.
## Follow-up: `template.ron` sync (an earlier session)
 
The mapping above (uppercase `X`/`Y`/`Z` official, lowercase `w`/`x`/`y`
cleared) was correct in this doc as soon as it was written, but
`resources/template.ron`'s Amazon Spear items (Wooden/Bronze/Iron Spear)
had been missed and were still pointing at the old lowercase `x`/`y`
glyphs — meaning Amazon's spears likely rendered as blank/transparent
tiles in-game for at least one full session after the glyph correction
above, despite this doc already being accurate. The user fixed this
directly in `template.ron`, moving those three items onto uppercase
`X`/`Y`/`Z`. No further doc change was needed since the mapping itself
was never wrong — just a reminder that a glyph correction here isn't
complete until every `template.ron` entry referencing the old glyph is
also updated to match. (Numeral glyphs `0`/`1`/`2` are unrelated to this
— those are Mage's Wooden/Silver/Arcane Staff tier, not spears.)
 
**This session's reservations (the 18 ability glyphs + `W`) were applied
directly in `template.ron` at the same time this doc was written** - see
the project instructions doc's Section 1 for confirmation. Unlike the
spear follow-up above, there's no known lag between doc and code for this
pass.
 
## Editing rule reminder
 
Every future sprite replacement must use the original PNG, clear the complete target 32×32 cell, place the replacement inside that same cell, and verify by pixel diff that no pixels outside the requested cells changed.
 
Additionally: confirm the source PNG is current before editing (Editing Rules §1A), match any user-supplied reference image as closely as possible (Editing Rules §36), and update both source-of-truth files at the end of any session where the sprite sheet changed (Editing Rules §37). **A session that only reserves codepoints in code/docs without touching the actual PNG (like this one) still counts as "the sprite sheet changed" for the purposes of §37** — the mapping itself changed, even though no pixels did.
