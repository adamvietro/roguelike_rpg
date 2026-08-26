# Working 
New classes
Rogue
Vanish - Flee and Stealth at the same time
Flurry - Multi Hit
Riposte - Counter attack for 2x damage

Barbarian
Whirlwind - For the next 3 turns auto attack with 2 extra attack damage

Mage




Amazon
animation for the throw spear
In-battle techniques

Javelin Volley — throw multiple spears in one turn (like a ranged version of Quick Attack's multi-hit).
Pierce Thrust — a spear jab that ignores some or all of the enemy's Defense, rewarding you for facing armored enemies.
Retreating Shot — deal damage and immediately guarantee your next Defend/Flee succeeds better, playing into "hit and create distance" instead of trading blows.
Called Shot — a slower wind-up attack (skip this turn) that guarantees a big hit next turn, a ranged cousin to Counter Attack.

Out-of-combat tools

Net Trap — a second trap variant: instead of damage, it roots/slows the first enemy that steps on it for a few turns (crowd control rather than damage).
Scout (Eagle Eye) — temporarily increases your FOV radius, letting you spot enemies (and Throw Spear targets) from farther away.
Reposition/Vault — a short instant dash a few tiles, useful for breaking line of sight or repositioning before a fight.

Passive/identity

Momentum — a passive that gives a small damage or evasion bonus while at full HP, encouraging hit-and-run play.
Keen Eyes — a small passive bonus specifically to Throw Spear's targeting range or bonus damage, making the ranged kit scale with level progression later.

Bosses will guarantee Loot.


Make the player or the Enemy stunned after a flee

More fluid motion - On hold

Battle has a more timed feel where it automatically ticks similar to FF7 

Sounds and Music

<br />

---
<br />

# Done

## HUD
Better UI for attacks
HUD Battle Item Tool Tip.
Add a 0 command for the item use
Hotkeys for abilities 
Different colors for active states of the player
Better Winning Screen.
Better Death Screen.

## Battle Arena
Have a Battle Victory Screen to show the items that you won.
Battle Screen
Background for the battle system based off the theme of the map.
Better looking battle options.
    I want a box around the abilities and I want them in the lower right

Floating attack values after an attack
battle messages in a list
Move the action box up
battle messages damage centered right and message centered left
30% change to absorb 50% of the damage when you defend.

### Bosses
Implemented a boss for each level.
Guarantee boss loot

### Abilities
Attack
Defend
Flee
Enemies will initiate a turn based RPG battle. Loot at the end will be class specific "cards" or abilities that a player can use.

### Stats
Add in speed for each type of enemy, and the player so that we can have the ones with low HP get an attack in, for the bigger enemies we want to have their speed slower.

Defense Stat 1 defense is a lot right now so might need to make a negative attack stat, maybe even a half defense stat.


## Player Animations
Flash on hit

## Classes
Each class will have default abilities that will need to be always available.
Add in items that can be won from battle that can be a single use.

### Barbarian
deathblow x2 dmg
quick attack x2 attacks
Rend 2 dmg for the next 3 turns
counter attack - chance (65%) for x3 damage
weapons is swords 3 types
hp 15
defense (0)
speed (6)

### Mage
Fireball (2 dmg) 
Invisible Cloak - For 20 moves you cannot be attacked but can pick up items (used out of combat)
Burn - Similar to Garrote, but a small damage to begin with
Defense is -1
Health is 10
Weapons is staffs 3 types
Ice Armor Increases Armor by 1 for 20 attacks
speed - 6

### Rogue
Stealth (used out of combat) 20 moves to pick up items, if you run into an enemy you will go first and get 3 times dmg.
garrote - wound for 2 dmg for next 3 turns
Dodge - for next 3 turns in battle 70% chance to dodge attacks.
Back-stab - 2x damage
Speed - 10
health- 10
defense - 0
attack - 1
Evasion- 10%

### Debug
hp - 100hp
evasion - 50
damage - 5
speed - 10

abilities
Victory - Instantly finishes the game
Defeat - Instantly Loses the game
Next Level - Instantly move to the next level

Starting Items
Victory - x1
Defeat - X1
Next Level - x2

### Amazon
Glyph - A
Weapons Icons - w, x, y

Attack - 1 
Defense - 0
Speed - 6
Evasion - 5%
Health - 12

Abilities
Spear - Outside battle attack that will target the nearest enemy and do attack + weapon dmg + 2 dmg
Battle Cry - For the next 3 turns enemies will do 1-2 less damage
Trap - Used outside combat will place a trap and if an enemy runs over it will instantly take 5 damage. Icon will be T
Poison Spear - Will do 1 dmg then 2 dmg for the next 3 turns

Starting Items
Trap x1, potion x1, Battle Cry x1


## Options
Title Screen
Start Screen for the game where you can pick a class.
Pause screen when pressing esc
esc on the title screen to quit the game

## Map
Remove random sword drops, make them only appear in presigned forts or final enemies in a dungeon.
More Prefabs

## Balance
Starting Items

## Refactors
Main into distinct Modules


