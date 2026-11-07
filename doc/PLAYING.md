# Playing DikuMUD: A Guide for Mortals

This guide takes a new player from the first login to level 20. It covers
choosing a class, getting around Midgaard, staying alive, gaining levels,
keeping your gear, and which areas to explore at which level.

The game is the 1991 DikuMUD, so it's unforgiving. **Read [Keeping your
stuff](#5-keeping-your-stuff) before you log off for the first time.**
Quitting the wrong way loses everything you're carrying.

Every number here comes from this server's code and world files. Immortals
and builders have their own guide: [WIZARDS.md](WIZARDS.md).

---

## Contents

1. [Connecting and creating a character](#1-connecting-and-creating-a-character)
2. [Choosing a class](#2-choosing-a-class)
3. [Your first hour](#3-your-first-hour)
4. [Staying alive](#4-staying-alive)
5. [Keeping your stuff](#5-keeping-your-stuff)
6. [Combat](#6-combat)
7. [Experience and levels](#7-experience-and-levels)
8. [Practising skills and spells](#8-practising-skills-and-spells)
9. [Class guides](#9-class-guides)
10. [Midgaard](#10-midgaard)
11. [Where to adventure](#11-where-to-adventure)
12. [Death](#12-death)
13. [Groups](#13-groups)
14. [Command reference](#14-command-reference)
15. [Tips and gotchas](#15-tips-and-gotchas)

---

## 1. Connecting and creating a character

Open the game in your browser. You start at a simulated C shell on an old
university computer. Type:

```
telnet mud
```

`Ctrl+]` gives you the `telnet>` prompt, and `close` leaves the game. Up
and down arrows recall earlier commands.

**Creating a character:**

1. Enter a name. A name nobody has used creates a new character, so check
   your spelling.
2. Choose a password (up to 64 characters) and type it again.
3. Choose your sex (`M` or `F`).
4. Choose a class: `C`leric, `T`hief, `W`arrior or `M`agic-user.
5. Read the message of the day and press Return to reach the menu:

```
0) Exit from DikuMud.
1) Enter the game.
2) Enter description.
3) Read the background story
4) Change password.
```

Choose `2` to write what other players see when they `look` at you (end
it with a line holding just `@`). Choose `1` to play.

Your abilities are rolled the first time you enter: 4d6 with the lowest die
dropped, five times, and the best results go to your class's main abilities
(see [§2](#2-choosing-a-class)). You can't re-roll them yourself.

New characters start at level 1 in the **Temple of Midgaard**, carrying
nothing, with 0 gold.

---

## 2. Choosing a class

| | Warrior | Thief | Cleric | Magic-user |
|---|---|---|---|---|
| **Main ability** (best roll goes to) | Str, Dex, Con | Dex, Str, Con | Wis, Int, Str | Int, Wis, Dex |
| **Hit points per level** | 10–15 | 7–13 | 5–10 | 3–8 |
| **Practices per level** | 1–2 | 1–2 | 2+ (Wis) | 2+ (Wis) |
| **Skills / spells** | kick, bash, rescue | sneak, hide, steal, backstab, pick lock | 30 spells | 27 spells |
| **Exp to level 10** | 500,000 | 160,000 | 450,000 | 250,000 |
| **Difficulty for a beginner** | Easiest | Medium | Medium | Hardest early, strongest late |

Constitution adds hit points every level for every class. Warriors with 18
strength also get a percentage bonus (18/xx).

- **Warrior**: the most hit points and the best fighter. No magic. Simple
  and sturdy, and the best choice for a first character.
- **Thief**: levels fastest. Backstab with a dagger does huge damage on
  the first hit. Starts with some skill in all five thief skills.
- **Cleric**: heals, protects and creates food and water, so you never
  need to buy any. Gets *sanctuary* (no hit can do more than 18 damage)
  at 13 and *heal* at 14.
- **Magic-user**: few hit points, but strong attack spells: *magic
  missile* at level 1 and *fireball* at 15. Very fragile at low levels.

**Everyone has exactly 100 mana**, whatever their class or level.
Higher-level casters don't get more mana; their spells get cheaper (see
[§8.2](#82-casting)).

---

## 3. Your first hour

### 3.1 Get your bearings

```
look          (or l)         describe the room
exits                        list the exits
score                        your hit points, mana, movement, level, exp, gold
inventory     (or i)         what you carry
equipment     (or eq)        what you wear
consider <monster>           how dangerous it is compared with you
help <topic>                 in-game help
```

Commands can be abbreviated, and the first match wins: `n` is north, `s` is
south and `l` is look.

### 3.2 Walk to your guild

The guilds are where you practise. From the **Temple** (where you start),
go `south` to the **Temple Square**, then:

| Class | Route from Temple Square | Guildmaster's room |
|---|---|---|
| Cleric | `w` (guild entrance), `n` (bar), `w` | Cleric's Inner Sanctum |
| Magic-user | `s` (Market Square), `w`, `w` (Main Street), `s` (guild entrance), `s` (bar), `e` | Mage's Laboratory |
| Warrior | `s`, `e`, `e` (Main Street), `s` (guild entrance), `e` (bar), `s` | Tournament and Practise Yard |
| Thief | `s`, `s` (Common Square), `e` (Dark Alley), `s` (guild entrance), `e` (bar), `s` | Secret Yard |

A guard at each guild entrance stops anyone from another class from
passing.

### 3.3 Practise

At your guildmaster, type `practice` to see what you can learn and how many
practice sessions you have, then spend them:

```
practice                       list skills/spells and sessions left
practice magic missile         magic-users
practice cure light            clerics
practice kick                  warriors
practice backstab              thieves
```

**New spells and skills start at 0%**, and with 0% a spell always fails.
Practise before you rely on anything (see [§8](#8-practising-skills-and-spells)).

### 3.4 Earn your first gold and gear

You start broke. Some ways to get going:

- **The Dump** (from the Market Square, `s` to the Common Square, then
  `s` again): drop junk here. **At levels 1–2 each item you drop gives you
  experience**, and from level 3 on it gives a little gold.
- **Kill easy things**: fido, the stray dog that roams the streets
  (level 0), and the ducks and sparrows in the park south of the river.
  Leave the drunk, the beggar and the janitor alone while you're weak: the
  city guards defend them (see [§6](#6-combat)). `get all corpse` after a
  kill to take its gold and items.
- **Sell loot** to the shops (see [§10.2](#102-shops)).

Basic kit to buy first:
- A **weapon** at the Weapon Shop (Market Square `e`, `e`, `n`): dagger
  (1d4, 13 coins; thieves need it for backstab), wooden club (1d3,
  15 coins), small sword (1d6, 78), warhammer (1d5, 65) or long sword
  (1d8, 780).
- **Leather armour** at the Leather Shop or the Armoury.
- A **light**: a torch (24 hours) or lantern (96 hours) from the General
  Store. Dark rooms show nothing without one. Use `hold torch` to light it.
- **Food and drink**: bread and pastries from the Bakery, and bottled
  drinks from the bar at the Grunting Boar. A bag from the General Store
  holds your loot.

### 3.5 Before you stop playing

Go to the **Reception** of the Grunting Boar Inn (Temple Square `e`, `u`)
and type `offer`, then `rent`. **Don't just `quit`.** See [§5](#5-keeping-your-stuff).

---

## 4. Staying alive

### 4.1 Hit points, mana and movement

`score` shows all three. They come back every **mud hour** (75 real
seconds):

| Position | Hit point regen | Mana regen | Movement regen |
|---|---|---|---|
| Standing | normal | normal | normal |
| Sitting | +1/8 | +1/4 | +1/8 |
| Resting | +1/4 | +1/2 | +1/4 |
| Sleeping | +1/2 | double | +1/2 |

Clerics and magic-users regain mana twice as fast but hit points half as
fast. Poison, hunger and thirst each cut regeneration to a quarter.

`rest` or `sleep` between fights, and `stand` (or `wake`) when you're done.
You can't move while sitting or resting.

Moving costs movement points according to the terrain: cities are cheap,
mountains expensive. When you run out you can't move, so rest.

### 4.2 Food and drink

You start full. Hunger and thirst drop by 1 every mud hour (24 = full,
0 = starving), so they run out after about **30 real minutes** of play. At
0 you don't die, but your regeneration is cut to a quarter until you eat or
drink.

| Need | Where |
|---|---|
| Food | Bakery: bread (fills 12 hours, 11 coins), danish pastry (5 hours, 5 coins) |
| Drink | Grunting Boar bar: bottles of beer, ale, local speciality and firebreather. Each guild's bar and the Grubby Inn sell bottles too |
| Water, tea, coffee | Cups at the **Park Cafe** in south Midgaard |
| Clerics | `cast 'create food'` (level 3), and `cast 'create water' <container>` (level 2) to refill an empty bottle |

You can only drink from a container you're carrying (`drink bottle`). The
fountain on Penny Lane is scenery. Keep empty bottles: `pour <bottle> out`
empties one, and `pour <from> <to>` moves a drink between containers.

Beer and ale quench thirst but make you drunk, and too drunk you can't
drink at all. Firebreather is the strongest.

### 4.3 Light

Many rooms are dark: caves, forests at night, Moria and the sewers. In the
dark you can't see the room, its exits, monsters or items. Carry a light
and `hold` it. Torches burn for 24 mud hours (30 real minutes) and lanterns
for 96.

### 4.4 Death traps

Some rooms kill any mortal who enters, with no corpse; your belongings stay
in the trap where nobody can reach them. The room before one usually warns
you in its description. Read descriptions in new areas, and don't flee
blindly in unexplored places, because `flee` picks a random exit
(it does avoid death traps, but you can still walk into one yourself).

---

## 5. Keeping your stuff

This is the most important section of this guide.

| What you do | What happens to your items |
|---|---|
| `rent` at a receptionist | **Kept.** Stored until you return; you start next time at that inn |
| `quit` | **Lost.** Everything you carry and wear is dropped on the floor where you stand |
| Link drops (browser closed) | Your character stays in the game, link-dead. Log in again with the same name to reconnect |
| Idle for 8+ mud hours | Pulled into the Void. After 48 hours you're removed from the game and lose your items |
| The game shuts down or reboots | Your character is saved, but **items you were carrying are lost** unless you rented |
| You die | Everything goes into your corpse ([§12](#12-death)) |

`save` stores your level, experience, gold and stats, **but not your
items**. Only renting stores items.

### 5.1 Renting

Receptionists are in the **Reception of the Grunting Boar Inn** (Temple
Square `e`, `u`) and in the inn in **Ofcol**.

```
offer        the receptionist quotes the cost per day
rent         pay, store your items, and leave the game
```

- Rent is the sum of each item's daily rent, charged **per real day**
  (prorated by the second) when you next log in. A dagger costs 3 coins
  per day; a bronze breast plate costs 1,000.
- At most **15 items**, counting items inside containers.
- Corpses and some special items can't be stored. The receptionist refuses
  them.
- Your gold stays with you and pays the rent.
- **If your gold runs out while you're away, the inn keeps your items.**
  Don't rent expensive gear for weeks with little gold.

---

## 6. Combat

```
kill <monster>     (or hit)      start a fight
flee                             run in a random direction
consider <monster>               compare its level with yours
```

Combat runs automatically, one round every 3 seconds, until someone dies
or flees. `score` and the damage messages tell you how it's going.

**`consider` before every new monster:**

| Message | Monster's level compared with yours |
|---|---|
| Now where did that chicken go? | 10+ levels below |
| You could do it with a needle! | 5–9 below |
| Easy. | 2–4 below |
| Fairly easy. | 1 below |
| The perfect match! | Same level |
| You would need some luck! | 1 above |
| You would need a lot of luck! | 2 above |
| You would need a lot of luck and great equipment! | 3 above |
| Do you feel lucky, punk? | 4–5 above |
| Are you mad!? | 6–10 above |
| You ARE mad! | 11+ above |

`consider` only compares levels. A monster's hit points, damage and spells
vary a lot within a level.

**Fleeing** has a cost: you lose the experience you earned by damaging that
monster (its missing hit points × its level). It can also fail ("PANIC! You
couldn't escape!"), so flee early, not at 2 hit points.

**Aggressive monsters** attack you as soon as they see you. Most of the
dangerous ones in the wild are aggressive: wolves, wargs, bats and rats in
the sewers, the undead in the graveyard.

**The city guards** step into fights in Midgaard. A guard attacks whoever
in the fight has the lowest alignment, as long as it's below 300 and that
person's opponent isn't evil. In practice, if you attack a *good*
townsperson (the drunk, the beggar, the janitor) near a guard, the guard
attacks you, and guards are level 10. If you fight fido or the mercenary,
they help you.

---

## 7. Experience and levels

You gain experience:
- **For every hit**: damage dealt × the monster's level.
- **For the kill**: a third of the monster's experience value, plus 1/8
  more for each level it's above you (up to +8 levels).
- At levels 1–2, for each item you drop at the Dump.

So fighting monsters at or above your level is far more rewarding than
farming easy ones. A level-0 fido gives you almost nothing.

When you have enough experience you rise a level at once ("You raise a
level"), gaining hit points and practice sessions. The `levels` command
shows the table for your class.

**Experience needed** (total, by level):

| Level | Magic-user | Cleric | Thief | Warrior |
|---:|---:|---:|---:|---:|
| 2 | 2,500 | 1,500 | 1,250 | 2,000 |
| 3 | 5,000 | 3,000 | 2,500 | 4,000 |
| 4 | 10,000 | 6,000 | 5,000 | 8,000 |
| 5 | 20,000 | 13,000 | 10,000 | 16,000 |
| 6 | 40,000 | 27,500 | 20,000 | 32,000 |
| 7 | 60,000 | 55,000 | 30,000 | 64,000 |
| 8 | 90,000 | 110,000 | 70,000 | 125,000 |
| 9 | 135,000 | 225,000 | 110,000 | 250,000 |
| 10 | 250,000 | 450,000 | 160,000 | 500,000 |
| 11 | 375,000 | 675,000 | 220,000 | 750,000 |
| 12 | 750,000 | 900,000 | 440,000 | 1,000,000 |
| 13 | 1,125,000 | 1,125,000 | 660,000 | 1,250,000 |
| 14 | 1,500,000 | 1,350,000 | 880,000 | 1,500,000 |
| 15 | 1,875,000 | 1,575,000 | 1,100,000 | 1,750,000 |
| 16 | 2,250,000 | 1,800,000 | 1,320,000 | 2,000,000 |
| 17 | 2,625,000 | 2,025,000 | 1,540,000 | 2,250,000 |
| 18 | 3,000,000 | 2,250,000 | 1,760,000 | 2,500,000 |
| 19 | 3,375,000 | 2,475,000 | 1,980,000 | 2,750,000 |
| 20 | 3,750,000 | 2,700,000 | 2,200,000 | 3,000,000 |

Level 20 is the highest mortal level. Beyond it, enough experience makes
you an immortal (level 21: 2,500,000 to 4,000,000, by class).

**Dying costs half your experience** ([§12](#12-death)). Levels you've
already gained are never taken away, but you have to earn back the lost
experience before the next level.

---

## 8. Practising skills and spells

### 8.1 Practice sessions

You get practice sessions every time you gain a level: magic-users and
clerics get at least 2 (more with high wisdom), warriors and thieves 1–2.
Spend them at your guildmaster. `practice` with no argument shows how many
you have.

Each session raises a skill or spell:
- **Spells**: by at least 25% (more with high intelligence), up to **95%**.
  Four sessions take a spell from 0 to 95%.
- **Thief skills**: by up to 12% (depending on intelligence), up to **85%**.
- **Warrior skills**: by up to 12%, up to **80%**.

Your guildmaster lists only the spells your level allows. Every few levels,
go back to the guild to learn the new ones.

### 8.2 Casting

```
cast '<spell>' [target]
cast 'magic missile' fido
cast 'cure light' (yourself, if no target)
cast 'cure light' bob
cast 'armor'
```

The spell name **must be in single quotes**. You can shorten it (`cast
'mag' fido`).

- Your chance of success is your learned percentage. If you fail ("You lost
  your concentration!") you still lose half the mana.
- **Mana cost** = the larger of the spell's minimum and 100 ÷ (2 + your
  level − the spell's level). A spell costs **50 mana** at the level you
  learn it, 33 a level later and 25 two levels later, falling to its
  minimum (usually 15, or 5 for utility spells). So new spells are
  expensive, and you can cast only two at first.
- Attack spells need you standing or fighting; most others need you
  standing.
- Warriors and thieves can't cast.

Scrolls (`recite`), potions (`quaff`), wands and staves (`use`) work for
everyone.

---

## 9. Class guides

### 9.1 Warrior

**Skills**: `kick` (extra damage), `bash` (knock the target down),
`rescue <ally>` (take the attacker off a friend). All start at 0%:
practise kick first.

**Leveling**: buy the best weapon you can wield and the heaviest armour
you can carry. The long sword (1d8) is the best in the shops. Fill every
slot: body, head, legs, feet, hands, arms, shield, wrists, about, waist,
neck, fingers. Fight things at your level or one above.

**Strength** matters most: it sets how much you can carry and adds to hit
and damage rolls.

### 9.2 Thief

**Skills** (you start at sneak 10, hide 5, steal 15, backstab 10, pick lock
10):

| Skill | Use |
|---|---|
| `backstab <target>` | Huge first-strike damage. **Needs a piercing weapon** (a dagger), and doesn't work once the fight has started |
| `sneak` | Move without the room seeing you arrive or leave, for a while |
| `hide` | Hide in the room until you act |
| `steal <item\|gold> <target>` | Steal. If you're caught, the victim attacks |
| `pick <door\|container>` | Open locks without the key (not pickproof ones) |

**Leveling**: backstab, finish the fight, rest, repeat. Practise backstab
first. Thieves level fastest of all classes.

**Careful**: stealing from players starts trouble, and some townspeople
shout "X is a bloody thief" when they catch you.

### 9.3 Cleric

**Spells by level:**

| Level | Spells |
|---:|---|
| 1 | armor, cure light |
| 2 | create water, detect poison |
| 3 | create food, detect magic |
| 4 | cure blind, detect evil |
| 5 | bless, detect invisibility |
| 6 | blindness, protection from evil |
| 7 | earthquake, sense life |
| 8 | poison, summon |
| 9 | cure critic, remove poison |
| 10 | dispel evil, locate object |
| 11 | word of recall |
| 12 | call lightning, remove curse |
| 13 | sanctuary, control weather |
| 14 | heal |
| 15 | harm |

**Leveling**: fight with a club or warhammer, cast *armor* and *bless*
before fights and *cure light* after. Clerics are welcome in any group.
*Word of recall* (11) takes you back to the Temple from anywhere, and
*sanctuary* (13) limits every hit you take to 18 damage.

### 9.4 Magic-user

**Spells by level:**

| Level | Spells |
|---:|---|
| 1 | magic missile, ventriloquate |
| 2 | detect invisibility, detect magic |
| 3 | chill touch |
| 4 | invisibility |
| 5 | armor, burning hands |
| 6 | locate object |
| 7 | shocking grasp, strength |
| 8 | blindness, teleport |
| 9 | lightning bolt |
| 10 | control weather |
| 11 | colour spray |
| 12 | curse, enchant weapon |
| 13 | energy drain |
| 14 | charm person, sleep |
| 15 | clone, fireball |

**Leveling**: at level 1 a magic missile costs 50 mana, so you get two
casts per full rest. Open with spells, then finish with a weapon, and rest
to full mana before the next fight (sleeping doubles mana regen). It gets
much easier from level 3, as costs fall. Each new attack spell does more
damage than the last. Avoid groups of monsters until you have more hit
points.

**Charm person** (14) makes a monster follow and fight for you (`order
<follower> <command>`). *Sleep* (14) stops a fight.

---

## 10. Midgaard

### 10.1 Map of the centre

North is up. `|` and `-` are exits.

```
                                Temple
                                  |
                  Cleric Guild--Temple Sq--Inn--Grunting Boar
                                  |         (up: Reception)
          Magic Shop   Bakery     |      General   Weapon
              |          |        |       Store     Shop
              |          |        |         |         |
West Gate--Main St----Main St--Market Sq--Main St---Main St--East Gate
              |          |        |         |         |
          Mage Guild  Armoury     |     Jeweller   Warrior Guild
                                  |
  Poor Alley--E. Poor Alley--Common Square--Dark Alley--Alley at Levee
                   |              |             |            |
               Grubby Inn     The Dump     Thief Guild     Levee
```

The Leather Shop is north of the west end of Poor Alley, and the Pet Shop
is north of the Alley at Levee. Wall Road runs south from inside the West
Gate, past the west end of Poor Alley, to the bridge.

Exact directions from the **Market Square**, which is the centre of town:

| Place | Directions | What's there |
|---|---|---|
| Temple Square | `n` | Hub |
| Temple of Midgaard | `n`, `n` | Start and recall point. The altar is `n` again |
| Grunting Boar Inn | `n`, `e` | Bar `e`; Reception `u` |
| Bakery | `w`, `n` | Bread, pastry |
| Armoury | `w`, `s` | Armour (chain and heavier) |
| Magic Shop | `w`, `w`, `n` | Scrolls, potions, wands, staves |
| General Store | `e`, `n` | Torches, lanterns, bags, boxes, paper, quills |
| Weapon Shop | `e`, `e`, `n` | Weapons |
| Jeweller | `e`, `s` | Buys and sells treasure (open 9–12 and 14–17) |
| Leather Shop | `s`, `w`, `w`, `n` | Leather armour (open 8–14 and 15–21) |
| Common Square | `s` | Hub for the south side |
| The Dump | `s`, `s` | Drop items for exp (levels 1–2) or gold |
| Pet Shop | `s`, `e`, `e`, `n` | Buy pets that follow and fight for you |
| Grubby Inn | `s`, `w`, `s` | A cheap bar |
| West / East Gates | `w`×3 / `e`×3 | Out of town |

Shops listed without hours are always open. Mud time runs 24 hours to a
mud day (30 real minutes); `time` shows it.

**The south side**: from inside the West Gate, Wall Road leads south past
Poor Alley to the **Bridge** and the southern district. There you'll find
the park and its pond, the **Park Cafe** (water, tea and coffee), the
**Cityguard headquarters** (don't go in looking for a fight), the
**mayor's office**, and the **grate to the graveyard** on the Concourse.
The Captain on the **Levee** (south of the Alley at Levee) sells a raft
and a canoe. You need a boat to cross deep water.

The **bulletin board** is in the Temple.

### 10.2 Shops

```
list              what the shopkeeper sells, with prices
buy <item>
sell <item>
value <item>      what the shopkeeper would pay you
```

Each shop buys only certain types of item. The weapon shop buys weapons,
the General Store buys lights, containers, food, trash and treasure, and so
on. Shops pay much less than they charge: the weapon shop sells at 130% of
the item's value and buys at 40%. The jeweller buys treasure at 50%.
Shopkeepers have limited gold and won't trade with anyone they can't see.

Don't attack a shopkeeper. They're level 23 with 30,000 hit points.

### 10.3 Pets

At the Pet Shop, `list` the animals and `buy <pet> [name]`. Pets cost three
times their experience value: a kitten or puppy 300 coins, a wolf 4,800. A
pet follows you and fights beside you. Command it with
`order <pet> <command>`.

---

## 11. Where to adventure

Ranges are a rough guide based on the monsters' levels and how aggressive
they are. "Aggressive" means they attack on sight.

| Levels | Area | How to get there | Notes |
|---:|---|---|---|
| 1–3 | **Midgaard streets** | – | Fido, janitor, drunk, beggar. Mind the guards ([§6](#6-combat)) |
| 1–3 | **The park** (south Midgaard) | Bridge, then the Concourse | Ducks, swan, sparrow. Safe and easy |
| 1–5 | **Haon-Dor, the light forest** | West gate, `w` | Rabbits, fox, deer (1–2), ferocious rabbit (3, aggressive), John the Lumberjack (5). The brown bear (8) is aggressive |
| 3–6 | **The sewers** | Down from any guild's back room, or the Dump | Small bats and spiders (1, aggressive), the great sewer rat (5, aggressive), beetles and snakes. Dark: bring a light |
| 4–7 | **The graveyard** | The grate at the south end of the Concourse | Henry the gardener (4); the rotting corpses (4) and ghouls (6) are aggressive |
| 3–8 | **Moria, levels 1–2** | East gate, onto the plains, north to the hills | Centipedes (3), kobolds (4), orcs (5–8), hobgoblins (6–10), snakes (7–10, poisonous). Dark |
| 5–8 | **Haon-Dor, the dark forest** | West through the light forest | Wolves (3) and wargs (5), all aggressive, in packs |
| 5–10 | **Ofcol village** | West gate, `n` along the trail and onto the northern plains | Citizens (5). The merchant (21) and the innkeepers are far too strong to fight. Has an inn with a receptionist |
| 7–14 | **Moria, levels 3–4** | Down holes and tunnels in upper Moria | Trolls (8–14), thieves (8–12), mages (11–13, cast spells), coloured bats (7, breath weapons) |
| 5–12 | **The Main Matrix** | Through the holy grove, which you reach from the plains east of the city or from Haon-Dor | A strange computer-themed area: bugs (1), cursors (3), cowboy (5), shredder (9), daemon (10). The Hierophant (20) guards the grove |
| 8–15 | **The deep sewers and the sewer maze** | Down pits and wells in the sewers | Mudmonsters (12), mindflayers (6–14), basilisk (13), sea hag (14), naga. The **red dragon** (19) lives down here |
| 12–20 | **Haon-Dor's depths** | The dark forest | Poisonous spiders (8), the Queen Spider (15) and a **green dragon (20)** |
| 4–15 | **The plains north of the city** | West gate, `n` along the trail around the wall | Sorbus the hermit (4), Aruncus the druid (13), and **Shudde-M'ell** (24), a giant worm. Don't fight the worm at any mortal level |

Things to avoid at any mortal level: Death (level 30), the walking ancient
trees in Haon-Dor (30), the Grand Knight of Paladins (29), Puff the Fractal
Dragon (26), and anything that `consider` calls "You ARE mad!".

---

## 12. Death

When your hit points fall below 0 you're incapacitated, then mortally
wounded, losing a little more every round. At -11 you die.

When you die:
1. **You lose half your experience** (never more than 500,000 at once).
   Your level stays.
2. **Everything you carry and wear, and all your gold, goes into your
   corpse** where you died.
3. You go back to the menu. When you enter the game again you start in the
   Temple with almost no hit points. Rest first.

**Getting your things back**: walk back and `get all corpse`. Your corpse
lasts about **12 real minutes** (10 mud hours) before it rots and
everything in it is gone. Hurry, but heal first or you'll die again on the
way. Fido and other scavengers eat corpses, contents and all. Other players
can loot your corpse.

---

## 13. Groups

Grouping lets players hunt together and share experience.

```
follow <leader>         each member follows the leader
group <member>          the leader adds a follower (again to remove them)
group <yourself>        the leader must add themself too
group                   list the group
```

- Only the leader can add members, and only players who follow them.
- **The leader must `group` themself**, or nobody gets a share.
- When a group member kills something, every grouped member **in the same
  room** gets an equal share of a third of the monster's experience. There
  is no bonus for monsters above your level in a group.
- Everyone still earns experience for the damage they deal personally.
- `rescue` (warriors) pulls attackers off weaker members. Clerics heal.
  Magic-users stay at the back.
- `follow <yourself>` leaves the group.

---

## 14. Command reference

**Movement**: `north` `south` `east` `west` `up` `down` (`n` `s` `e` `w` `u`
`d`), `enter`, `leave`, `exits`, `open`/`close`/`lock`/`unlock <door>`,
`stand` `sit` `rest` `sleep` `wake`.

**Looking**: `look [thing]`, `examine <thing>`, `read <thing>`, `score`,
`inventory`, `equipment`, `consider`, `where <name>` (finds someone in your
zone), `time`, `weather`, `who`, `levels`.

**Items**: `get <item> [container]`, `get all`, `take`, `drop`, `put <item>
<container>`, `give <item> <person>`, `give <n> coins <person>`, `wear`,
`wield`, `hold`/`grab`, `remove`, `eat`, `drink`, `sip`, `taste`, `pour`,
`quaff` (potions), `recite` (scrolls), `use` (wands, staves).

**Shops and inns**: `list` `buy` `sell` `value`, `offer` `rent`.

**Combat**: `kill`/`hit`, `flee`, `consider`, `kick` `bash` `rescue`
(warriors), `backstab` `steal` (thieves), `cast`, `order`.

**Talking**: `say <text>` (or `'text`), `tell <name> <text>`, `whisper`,
`ask`, `shout` (everyone in the game), `emote <action>` (or `,action`),
and dozens of socials (`smile`, `bow`, `hug`, `poke`…; `help social`).

**Bulletin board** (in the Temple): `look board`, `read <n>`,
`write <title>` (end with `@`), `remove <n>`.

**Settings**: `brief` (shorter room descriptions), `compact` (no blank line
after output).

**Information**: `help`, `news`, `info`, `credits`, `wizlist`.

**Feedback to the gods**: `bug <text>`, `typo <text>`, `idea <text>`.
They're saved with the room you were in.

**Leaving**: `save`, `rent` at an inn, and `quit` (which drops your
belongings, see [§5](#5-keeping-your-stuff)).

---

## 15. Tips and gotchas

- **Rent, don't quit.** It's worth saying twice.
- **Practise before you rely on a spell or skill.** Everything starts at 0%.
- **Read `help <command>`** whenever you're unsure.
- **Carry two lights**, plus food and water, on any trip out of town.
- **Check `score` during fights**, and flee while you still can.
- **Containers**: a bag (`put all bag`) holds 50 pounds and saves
  inventory slots. Containers count towards the 15-item rent limit, along
  with their contents.
- **Encumbrance**: strength limits how much weight you carry. The number
  of items is limited to 5 + half your dexterity + half your level. If you
  can't pick something up, drop or sell something.
- **Doors**: `open door`, and `unlock door` if you have the key. Keys have
  names like "key" or "brass", so `look key` helps.
- **Alignment** moves with what you kill: killing good things makes you
  evil and the other way round. Some equipment burns wearers of the wrong
  alignment, and cityguards don't help evil characters.
- **Lost items**: if an immortal tells you they've set aside an item for
  you, `pray` at the altar north of the Temple to receive it. Its value is
  taken from your gold.
- **Ask for help.** Use `shout` to reach everyone, or `tell` an immortal
  (`wizlist` shows who they are).
