# Working 
Enemies will initiate a turn based RPG battle. Loot at the end will be class specific "cards" or abilities that a player can use.



New classes
Rogue


Amazon

Make the player or the Enemy stunned after a flee

More fluid motion - On hold

Battle has a more timed feel where it automatically ticks similar to FF7 


Refactor main.rs

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


### Abilities
Attack
Defend
Flee

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


