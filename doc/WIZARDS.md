# The Wizard's Guide to DikuMUD in Rust

A guide for immortals (levels 21–24) and world builders on this server. It
covers immortal levels and powers, every immortal command, the world data
files, zone resets, shops, special procedures and adding commands.

Everything here was checked against the source in `src/` and the data in
`lib/`. Where the Rust port behaves differently from the original 1991 docs
in `doc/*.doc`, this guide follows the code. File and line references are
correct at commit `0c2ac99`.

---

## Contents

1. [Becoming an immortal](#1-becoming-an-immortal)
2. [Immortal levels](#2-immortal-levels)
3. [What being immortal changes](#3-what-being-immortal-changes)
4. [Command reference](#4-command-reference)
5. [Moderation and running the game](#5-moderation-and-running-the-game)
6. [How the world is put together](#6-how-the-world-is-put-together)
7. [Rooms: `tinyworld.wld`](#7-rooms-tinyworldwld)
8. [Mobiles: `tinyworld.mob`](#8-mobiles-tinyworldmob)
9. [Objects: `tinyworld.obj`](#9-objects-tinyworldobj)
10. [Zones and resets: `tinyworld.zon`](#10-zones-and-resets-tinyworldzon)
11. [Shops: `tinyworld.shp`](#11-shops-tinyworldshp)
12. [Special procedures](#12-special-procedures)
13. [Help, socials and other text files](#13-help-socials-and-other-text-files)
14. [Building workflow and checklist](#14-building-workflow-and-checklist)
15. [For code implementers](#15-for-code-implementers)
16. [Reference tables](#16-reference-tables)

---

## 1. Becoming an immortal

**The first character ever created becomes the Implementor (level 24).** The
server decides this when no player files exist yet (`init_char`,
`src/db.rs:921`). That character skips the stat roll, so it starts with every
ability at 0 and can't carry anything. Fix it right away:

```
restore <yourname>
```

At level 23 and above, `restore` sets all six abilities to 25 (18/100
strength) and teaches every skill at 100%.

New characters after that start as normal level-1 players. To make one
immortal, a level-24 character uses `advance`. See [§4.6](#46-advance).

Mortals can also reach level 21 through experience. `gain_exp`
(`src/limits.rs:178`) keeps raising the level while the character's
experience passes the next threshold in the title table, and it only stops
for characters who are already level 21 or higher. A level-20 player with
enough experience (2,500,000 to 4,000,000, depending on class) becomes an
immortal. The original game did the same.

---

## 2. Immortal levels

| Level | Rank (`wizlist` term) | Title, e.g. Magic User          | Experience (MU) |
|------:|-----------------------|---------------------------------|----------------:|
| 21    | Immortal / Demigod    | the Immortal Warlock/Enchantress | 4,000,000      |
| 22    | Avatar / Demigod      | the Avatar of Magic             | 5,000,000       |
| 23    | God / Greater God     | the God(dess) of magic          | 6,000,000       |
| 24    | Implementor           | the Implementator/Implementress | 7,000,000       |

Titles come from the `TITLES` table in `src/constants.rs`. The experience
needed for levels 21–24 varies by class. The game only checks the number.
What each rank may do comes from each command's `min_level` in the command
table (`CMD_INFO`, `src/interpreter.rs:28`) and from a few level checks
spread through the code ([§3](#3-what-being-immortal-changes)).

`wizhelp` lists every command with a minimum level of 21 or more that your
level allows.

---

## 3. What being immortal changes

These apply to player characters only, not to NPCs. "21+" means level 21
and above.

**Survival**
- **21+: you take no damage at all** from any source, including weapons,
  spells and poison (`damage()`, `src/fight.rs:296`). Attackers see the
  combat message's `god_msg` variant.
- 21+: death-trap rooms don't kill you, whether you walk or teleport in
  (`src/act_move.rs:59`, `src/magic.rs:201`).
- 21+: poisoned food has no effect on you.
- 21+: every saving throw you make succeeds (`src/spell_parser.rs:384`).
- 21+: hunger, thirst and drunkenness are set to -1 (never change) whenever
  you gain a level (`src/limits.rs:157`). If yours aren't -1, use
  `set <you> value full -1` (same for `thirst` and `drunk`).

**Movement and magic**
- 21+: moving costs no movement points.
- 21+: you can cast any spell regardless of class or level, warriors and
  thieves included, and you never run out of mana. Mana is still subtracted
  and can go negative (`src/spell_parser.rs:395`, `:569`).
- `summon` always fails on a target above level 20.

**Economy**
- 21+: receptionists store your items for free ("Well, since you're a God…").
- 22+: buying from a shop costs nothing (`src/shop.rs:131`).
- 22+: you can `give` any amount of gold without having it, and it isn't
  taken from you. Every gift of gold from a 21+ player to another player is
  logged.
- 22+: you can eat things that aren't food.

**Visibility and information**
- 21+: `where` with no argument lists every connected player and their room
  number. `where <name>` searches the whole world for characters and
  objects and shows room numbers.
- 23+: `who` also shows players you couldn't otherwise see.
- **Being immortal doesn't let you see in the dark or see invisible things.**
  `CAN_SEE` (`src/game.rs:322`) has no level check. Use the spells (`cast
  'detect invisibility'`) or carry a light. Many commands look things up
  "visibly" (`get_char_vis`, `get_obj_vis`), so in a dark room, or against
  an invisible target, they answer "No-one by that name".
- Thieves can't steal from 21+ players, and NPC thieves leave them alone.

**Idling**
- 22+: you're never pulled into the void for idling (`src/limits.rs:337`).
  At level 21 you still are, after 8 idle ticks (8 mud hours, about
  10 real minutes).

---

## 4. Command reference

Commands are matched by **prefix in table order**: the first command whose
name starts with what you typed wins (`find_command`,
`src/interpreter.rs:285`). So `s` is `south`, `sh` is `shout`, and `shutdow`
is a decoy so that you can't shut the game down by accident. Type `shutdown`
in full.

Typing a command above your level prints `Arglebargle, glop-glyf!?!`.

### 4.1 Full list

| Command      | Lvl | Position  | Summary |
|--------------|----:|-----------|---------|
| `wizhelp`    | 21  | sleeping  | List the immortal commands available to you |
| `goto`       | 21  | sleeping  | Teleport to a room, character or object |
| `at`         | 21  | dead      | Run one command somewhere else |
| `stat`       | 21  | dead      | Show internal data for a room, object or character |
| `echo`       | 21  | sleeping  | Send raw text to everyone else in the room |
| `snoop`      | 21  | dead      | Watch another player's session |
| `wiz` / `;`  | 21  | dead      | Immortal channel (to everyone above level 20) |
| `users`      | 21  | dead      | List connections with their source address |
| `where`      | 1*  | dead      | *With no argument, 21+ only.* Locate players, mobs, objects |
| `force`      | 22  | sleeping  | Make someone (or `all`) run a command |
| `transfer`   | 22  | sleeping  | Pull someone (or `all`) to your room |
| `load`       | 22  | dead      | Create a mobile or object from its prototype |
| `purge`      | 22  | dead      | Destroy a character or object, or clean the room |
| `restore`    | 22  | dead      | Restore hit points, mana and movement; more for immortals |
| `noshout`    | 22  | sleeping  | Toggle someone's ability to shout (or your own hearing) |
| `notell`     | 22  | dead      | Toggle someone's ability to `tell` (or your own) |
| `noemote`    | 22  | dead      | Toggle someone's ability to `emote` |
| `snowball`   | 22  | standing  | A social that only gods can use |
| `string`     | 23  | sleeping  | Edit names and descriptions of live mobiles and objects |
| `set`        | 23  | sleeping  | Change a character's stats or skills |
| `advance`    | 23  | dead      | Raise a level-0 character to 1; 24 can make immortals |
| `switch`     | 23  | dead      | Take over an NPC's body |
| `return`     | 0   | dead      | Go back to your own body after `switch` |
| `freeze`     | 23  | dead      | Toggle a player's frozen state |
| `shutdown`   | 24  | dead      | Save everyone and stop (or `shutdown reboot`) |
| `shutdow`    | 24  | dead      | Does nothing; guards against `shutdown` typed by accident |
| `reroll`     | 24  | dead      | Re-roll a character's abilities |
| `wizlock`    | 24  | dead      | Toggle refusing all new connections |
| `gol`        | 24  | dead      | Toggle the PLR_LOG flag (has no effect; hidden from `wizhelp`) |
| `kill`       | 24* | fighting  | *At 24 it slays instantly; otherwise a normal attack* |

### 4.2 Movement and location

**`goto <room vnum | name>`**: move to a room by virtual number, or to the
room of a character or object you can see. Bystanders at both ends see the
puff of smoke and the "ear-splitting bang". `goto` respects `PRIVATE` rooms,
but only when two or more characters are already there.

```
goto 3001          the Temple of Midgaard
goto fido          wherever the first visible "fido" is
goto 1202          the immortal "centre of the universe" room
```

**`at <room | name> <command>`**: move there invisibly, run the command, and
come back. It ignores `PRIVATE`. If the command moves you (for example
`at 3001 north`), you stay where it took you.

```
at 3014 look
at puff stat puff
at 3001 load obj 3022
```

**`transfer <name | all>`**: bring a character to your room. `all` brings
every connected player except you (link-dead characters are skipped).

**`where [name]`**: see [§3](#3-what-being-immortal-changes). When you're
switched into a mobile, the no-argument list shows "In body of …".

### 4.3 Information

**`stat room`**: the current room's zone index, virtual and real number,
sector, flags, special procedure (yes/no), extra description keywords,
occupants (PC/NPC/MOB), contents and every exit with its flags, key and
destination vnum.

**`stat <name>`**: **objects are searched before characters.** If an object
and a mobile share a keyword, you get the object. Use `2.name`, or `stat` a
more specific keyword.

- *Objects*: R- and V-number, type, wear and extra flags, the four values
  decoded by type, weight, cost, rent, timer, where it is, what it
  contains, and its `affected` slots.
- *Characters*: PC/NPC/MOB, R- and V-number, class, level, alignment, ages,
  abilities, current, maximum and regen of hit points, mana and movement, AC,
  gold, experience, hit and damage rolls, position, act flags, timer, saving
  throws, conditions, master and followers, `affected_by` bits and every
  spell affect with its remaining duration.

**`users`**: one line per connection, with the client's address in
brackets. Connections still at the login prompt show as `UNDEFINED`.

**`snoop <name>`**: see everything the target sees. You can only snoop
players below your own level, and only one immortal can snoop a given
target at a time. `snoop <yourself>` stops. The target isn't told.

### 4.4 Creating and destroying

**`load char <vnum>`** / **`load obj <vnum>`**: create a fresh copy of a
prototype in your room. You can abbreviate the type (`load c 3062`,
`load o 3022`). The game logs it as `"<you> loads <short desc> at <room>"`.
Loading **ignores the zone's maximums** but does raise the count the reset
commands check ([§10.3](#103-reset-commands)), so if you load extra copies,
zone resets stop creating that mob or object until the extras are gone.

**`purge`**: with no argument, destroys every NPC and every object in the
room ("The world seems a little cleaner."). Players stay.

**`purge <name>`**: destroys one character or object. Level 22 can only
reach characters in the same room. Level 23 can reach any character in the
world by name (`get_char`, which ignores visibility). Objects must be on the
floor of your room. You can't purge a player of your own level or higher.
**Purging a player closes their connection without saving.** Everything
they carried falls to the floor, and they lose any progress since their
last save. Use it only as a last resort.

### 4.5 Characters

**`restore <name>`**: sets hit points, mana and movement to maximum. Any
character in the world, visible or not.
- Target 21+: also sets every skill to 100% and recognised.
- Target 23+: also sets every ability to 25 (strength 18/100).

**`reroll <name>`** (24): rolls 4d6-drop-lowest five times and assigns the
results to the abilities in the order the class prefers, as at character
creation (`roll_abilities`, `src/act_wizard.rs:715`).

**`set`** (23). Type `set` with no arguments for the list of fields and
skills. The game logs every change.

```
set <name> value <field> <value>
set <name> skill '<skill name>' <learned 0-100> <y|n>
```

`value` and `skill` can be shortened to `v` and `s`. You must type the field
name **in full**. You can shorten the skill name, which goes in single
quotes. The last argument says whether the character "recognises" the skill.

| Field       | Range / values     | Notes |
|-------------|--------------------|-------|
| `age`       | 16–79              | Rewrites the birth time |
| `sex`       | `m` `f` `n`        | |
| `class`     | `m` `c` `w` `t`    | Magic User, Cleric, Warrior, Thief |
| `level`     | 0–24               | **No check against your own level**, see below |
| `height`    | 100–250            | cm |
| `weight`    | 100–250            | pounds |
| `str` `int` `wis` `dex` `con` | 1–18 | Sets the *permanent* ability |
| `stradd`    | 1–100              | Out-of-range values warn but are **still applied** |
| `gold`      | any                | |
| `exp`       | 1–7,000,000        | |
| `mana` `move` | -99–200          | Current points, not maximum |
| `hit`       | -9–30000           | Current points |
| `sessions`  | 0–100              | Practice sessions |
| `alignment` | -1000–1000         | |
| `thirst` `drunk` `full` | -1–100 | -1 means it never changes |

Skills you can name: armor, teleport, bless, blindness, burning hands, call
lightning, charm person, chill touch, clone, colour spray, control weather,
create food, create water, cure blind, cure critic, cure light, curse,
detect evil, detect invisibility, detect magic, detect poison, dispel evil,
earthquake, enchant weapon, energy drain, fireball, harm, heal,
invisibility, lightning bolt, locate object, magic missile, poison,
protection from evil, remove curse, sanctuary, shocking grasp, `zzzzz`
(sleep), strength, summon, ventriloquate, word of recall, remove poison,
sense life, sneak, hide, steal, backstab, pick lock, kick, bash, rescue.
(The help text that `set` prints leaves out "armor", but you can still set
it.)

> **Caution.** `set` never compares the target's level with yours. A
> level-23 god can `set` an Implementor, or set their own `level` to 24.
> Only promote people you trust to 23.

`set` changes the *permanent* abilities. The working values used in play
are recalculated (`affect_total`) when the character next wears or removes
something, gains or loses a spell effect, or logs in again.

**`string`** (23): edit the text of a live mobile, player or object. The
change is in memory only and is lost at the next reboot. Change the data
files for anything permanent.

```
string char <name> <field> [text]
string obj  <name> <field> [text]
```

Fields can be shortened. If you leave out the text, the game opens the line
editor: type lines, and finish with `@` (anything after `@` on that line is
dropped).

| Field | char | obj | Max length |
|-------|------|-----|-----------:|
| `name` | NPCs; players only at level 24 (with a warning) | keyword list | 15 |
| `short` | NPCs only | short description | 60 |
| `long` | NPCs only | room description | 256 |
| `description` | look-at text | **extra description**: `string obj <o> desc <keyword>` | 240 / 4096 |
| `title` | players only | – | 60 |
| `delete-description` | – | removes the extra description with that keyword | – |

Lengths are counted in characters. Each line you enter in the editor also
counts its line break. For mobile `long` descriptions, end the text with a
newline, as in the data file, or the next line of the room display runs on.

**`advance <name> <level>`**: see the next section.

**`switch <mob>`** / **`return`**: take over an NPC (any mob in the world,
found by name regardless of visibility). You can't switch into a player or
into a body someone else is already using, or while you're snooping or being
snooped. Your own body stays where it is, defenceless and link-less. `return`
brings you back.

### 4.6 `advance`

```
advance <name> <level>
```

The target must be **in your room** and must be a player.

- **Level 0 → 1**: any level-23 god can do this (no level argument needed).
  It runs character creation (`do_start`). Normal new characters are
  started automatically when they first enter the game, so you rarely need
  this.
- **To 21–24**: Implementors (level 24) only. You can only raise a level,
  never lower it, and only to 21, 22, 23 or 24. The game sets the target's
  experience to the threshold for the new level and runs `advance_level`
  for each step, then logs the change.
- To lower a level, use `set <name> value level <n>` (23+).

### 4.7 Communication

| Command | Effect |
|---------|--------|
| `wiz <text>` or `; <text>` | `::Yourname::text` to everyone above level 20 |
| `echo <text>` | Raw text, no name attached, to everyone else in your room |
| `emote <text>` | `Yourname <text>` to the room (open to everyone) |
| `force <name> <command>` | The target sees `X has forced you to '…'` and runs it |
| `force all <command>` | Every connected player except you; anyone above your level refuses |

`force` works on NPCs of any level, and on players of your level or below.
A forced command goes through the normal interpreter, so it must be at or
below the target's level. Forcing a mob is how you make Puff speak, or test
a special procedure.

---

## 5. Moderation and running the game

### 5.1 Player flags

All of these toggle: run the command again to undo it. They only work on
players at or below your level (the `gol` log flag requires a level strictly
below yours). Players keep these flags in their save files.

| Command | Flag | Effect |
|---------|------|--------|
| `noshout <name>` | `PLR_NOSHOUT` | Can't shout (and doesn't hear shouts) |
| `noshout` (alone) | – | Toggle *your own* hearing of shouts |
| `notell <name>` | `PLR_NOTELL` | Can't use `tell`; tells to them fail |
| `notell` (alone) | – | Stop receiving tells yourself |
| `noemote <name>` | `PLR_NOEMOTE` | Can't `emote` |
| `freeze <name>` | `PLR_FREEZE` | Every command answers "You're totally frozen!" |
| `gol <name>` | `PLR_LOG` | Sets the flag. **This port doesn't act on it**: nothing gets logged |

### 5.2 Wizlock

`wizlock` (24) makes the server refuse every new WebSocket connection with
`503 The game is wizlocked...`. **That includes immortals.** Only people
already connected can type `wizlock` again to lift it. The setting isn't
saved, so a restart also clears it.

### 5.3 Shutting down and rebooting

- `shutdown`: announces "Shutdown by X.", saves every player online and
  exits.
- `shutdown reboot`: the same, but logged as a reboot.

How the game comes back up depends on how it runs: Docker Compose and the
AWS deployment restart the container by themselves (`restart:
unless-stopped`), and `cargo run` doesn't. See the README. **`docker stop`
or killing the process doesn't save anyone.** Always `shutdown` first.

**Daily automatic reboot.** If a file named `reboot` exists in the data
directory, the game warns players at 09:30 and 09:40 **UTC**, then reboots
after 09:50 UTC (`check_reboot`, `src/modify.rs:281`). If the file isn't
empty, the game first runs it with `sh`. When the script succeeds the file
is renamed `reboot.SUCCEEDED`. If it fails, the reboot is cancelled and the
file is renamed `reboot.FAILED`.

### 5.4 Logs

The server logs to stderr (`docker compose logs`). Look there for:

- `X loads <thing> at <room>.`, `X purges Y`, `X restored Y`,
  `X rerolled Y`, `X sets Y's field to N.`, `X changes Y's skill…`,
  `X advances Y to level N.`, `X gives Y N gold coins.`
- `Shutdown by X.`, `Game has been wizlocked by X.`
- `SYSERR …`: the code tried to use a character or object after it was
  destroyed. The game loop catches it and keeps running. Report it as a bug.
- Boot messages: `Room N does not exist in database`, `assign_mobiles:
  mobile N does not exist`, `Undefd cmd in reset table`. See
  [§14](#14-building-workflow-and-checklist).

Players' `bug`, `idea` and `typo` reports are added to `lib/bugs`,
`lib/ideas` and `lib/typos` with the room number they were written in.

---

## 6. How the world is put together

### 6.1 Files

All world data is plain text in the data directory (`lib/` by default, or
`-d <dir>`):

| File | Holds | Loaded by |
|------|-------|-----------|
| `tinyworld.zon` | Zones: name, room range, lifespan, reset commands | `boot_zones` |
| `tinyworld.wld` | Rooms | `boot_world` |
| `tinyworld.mob` | Mobile prototypes | `boot_mobiles` |
| `tinyworld.obj` | Object prototypes | `boot_objects` |
| `tinyworld.shp` | Shops | `boot_the_shops` |
| `help_table`, `help`, `info`, `news`, `motd`, `credits`, `wizlist` | Text | `boot_db` |
| `actions`, `poses`, `messages` | Socials, poses, combat messages | various |

**The world files are read only at boot.** `load`, `string`, `set` and
`purge` change the running game. They never write back to these files.
Edit the files, then restart. With Docker, edit `lib/` in the repository
and rebuild the image: on each start the container copies the image's game
data over the copies in the volume.

### 6.2 Virtual and real numbers

Every room, mobile prototype and object prototype has a **virtual number
(vnum)**: the `#1234` in the file, which builders use everywhere. In memory
they're stored in arrays, and the **real number (rnum)** is the array index.
`stat` shows both.

`real_room`, `real_mobile` and `real_object` (`src/db.rs:972`) convert vnums
to rnums with a **binary search**. So:

> **Rooms, mobiles and objects must each appear in strictly ascending vnum
> order in their files.** The game checks this for rooms, against the zone
> ranges. It **doesn't check mobiles or objects**: one out of order makes
> its neighbours silently impossible to find (`load` says "There is no
> monster with that number", and resets skip them).

### 6.3 Zones own rooms by number range

Zones are listed in `tinyworld.zon` in ascending order of their **top room
number**. Zone *n* owns every room from (previous zone's top + 1) up to its
own top. The zone number written in each room record is **ignored**: a room
belongs to whichever zone's range its vnum falls in. If a room's vnum is
above the last zone's top, the server logs `Room N is outside of any zone.`
and won't start.

Current zones:

| # | Top  | Life (min) | Mode | Name |
|--:|-----:|-----:|:----:|------|
| 0 | 3    | 1    | 0 | LIMBO |
| 1 | 3099 | 30   | 2 | Northern Midgaard Main City (includes god rooms 1200–1202) |
| 2 | 3199 | 40   | 2 | Southern part of Midgaard |
| 3 | 3299 | 40   | 0 | River of Midgaard |
| 4 | 3399 | 120  | 0 | Buildings in Midgaard |
| 5 | 3499 | 40   | 2 | The Graveyard |
| 6 | 3699 | 40   | 2 | The Graveyard |
| 7 | 4099 | 20   | 2 | Moria level 1-2 |
| 8 | 4199 | 40   | 2 | Moria level 3-4 |
| 9 | 5500 | 240  | 2 | Plains north of City of Midgaard |
| 10 | 5550 | 60  | 2 | City of Ofcol |
| 11 | 6099 | 30  | 2 | HAON-DOR-LIGHT |
| 12 | 6199 | 30  | 2 | HAON-DOR-DARK |
| 13 | 6999 | 30  | 2 | QUIFAEL-CUSTOM-ROOMS |
| 14 | 7099 | 15  | 2 | Sewer |
| 15 | 7199 | 20  | 2 | SecondSewer |
| 16 | 7399 | 25  | 2 | SEWERMAZE |
| 17 | 7799 | 25  | 2 | The tunnels |
| 18 | 7899 | 3   | 2 | Redferne's fridge and larder |
| 19 | 7999 | 100 | 2 | Redferne's Residence |
| 20 | 8099 | 10  | 2 | Main matrix |
| 21 | 8899 | 5   | 1 | Main_matrix_auxiliary |
| 22 | 8999 | 10  | 1 | The_holy_grove |

Rooms with special meaning: **0–1** The Void (where idle players go),
**2** Limbo (Puff), **1200–1202** private god rooms, **3001** the Temple of
Midgaard (new players and players whose saved room no longer exists start
here).

### 6.4 Text file conventions

- **Strings** end with `~`. They can span several lines. The `~` must be the
  last character on its line apart from trailing spaces, so you can't
  have text after it. An empty string is a line holding just `~`.
- **Numbers** are separated by any whitespace, including newlines.
- **Bit vectors** (flags) are written as the **decimal sum** of the bit
  values. Example: `DARK | INDOORS` = 1 + 8 = `9`.
- Carriage returns are removed when the files are read, so files with
  Windows line endings load fine.
- Each file ends with a `$~` record (`#99999` then `$~` in `.wld`, `$` in
  `.mob`/`.obj`, `$~` in `.zon` and `.shp`).

---

## 7. Rooms: `tinyworld.wld`

```
#<vnum>
<name>~
<description, any number of lines>
~
<zone (ignored)> <room flags> <sector type>
{ D<dir>
  <exit description>~
  <door keywords>~
  <door flag> <key vnum> <to-room vnum>
| E
  <keywords>~
  <extra description>
  ~
}*
S
```

Example (`lib/tinyworld.wld`, room 0):

```
#0
The Void~
  You step out into ......
  You don't think that you are not floating in nothing.
~
0 8 1
D4
void~
~
0 -1 3001
S
```

**Name**: shown as the room title. **Description**: by convention it starts
with three spaces and wraps at 79 columns.

**Room flags** (sum the ones you want):

| Bit | Name | Effect |
|----:|------|--------|
| 1   | `DARK` | Dark unless someone has a light |
| 2   | `DEATH` | Death trap: mortals who enter die at once, with no corpse; everything they carried is left on the floor |
| 4   | `NO_MOB` | Wandering mobiles won't enter |
| 8   | `INDOORS` | No weather messages |
| 16  | `LAWFULL` | (unused) |
| 32  | `NEUTRAL` | (unused) |
| 64  | `CHAOTIC` | (unused) |
| 128 | `NO_MAGIC` | **Not enforced**: the flag shows in `stat room`, but nothing checks it |
| 256 | `TUNNEL` | (unused in this code) |
| 512 | `PRIVATE` | `goto` and `summon` blocked when 2+ people are present |

Wandering mobiles also avoid `DEATH` rooms.

**Sector types**: 0 inside, 1 city, 2 field, 3 forest, 4 hills, 5 mountain,
6 water (swimmable), 7 water (needs a boat). The sector sets the movement
cost.

**Exits** (`D0`–`D5` = north, east, south, west, up, down):

- *exit description*: what `look <direction>` shows. It can be empty (`~`).
- *door keywords*: the names used with `open`, `close`, `lock` and so on
  (e.g. `door gate~`). Empty for an open passage.
- *door flag*: `0` no door, `1` door, `2` pickproof door.
- *key vnum*: the object vnum of the key, or `-1` if none (it can't be
  locked).
- *to-room*: destination room vnum, or `-1` for an exit that leads nowhere.

Exits are **one-way**. To make a passage you can walk both ways, give each
room an exit to the other. A door is two independent records, one per side,
and the zone file must close and lock both
([§10.3](#103-reset-commands)). Doors start **open** at boot unless a `D`
reset command closes them. If the destination vnum doesn't exist, the boot
log says `Room N does not exist in database` and the exit leads nowhere.

**Extra descriptions** (`E`): `look <keyword>` shows the text. Several
keywords can share a description: `E` / `notice sign~` / text / `~`.

---

## 8. Mobiles: `tinyworld.mob`

Almost every mob (148 of 150) uses the **simple ("S") format**:

```
#<vnum>
<keywords>~
<short description>~
<long description (shown in the room)>
~
<look-at description>
~
<act flags> <affected_by flags> <alignment> S
<level> <THAC0> <AC> <hp: XdY+Z> <bare-hand damage: XdY+Z>
<gold> <experience>
<position> <default position> <sex>
```

Example (Puff, `#1`):

```
#1
Puff~
Puff~
Puff the Fractal Dragon is here, contemplating a higher reality.
~
~
1 0 1000 S
26 1 -1 5d10+550 4d6+3
10000 155000
8 8 2
```

- **keywords**: what players type to refer to it (`kill puff`). Put the
  most specific first: `stat` and `fname()` use the first word.
- **short description**: used in action messages ("**the Baker** hits you").
  It starts lower case unless it's a proper name.
- **long description**: the line shown when the mob stands in a room at its
  default position. It must end with a newline before the `~` (as above).
- **THAC0**: stored as `hitroll = 20 - THAC0`, so lower is better.
- **AC**: stored as `10 × AC`, from 10 (unarmoured) down to -10.
- **hp dice**: rolled again for every copy created. `1d1+30000` gives an
  effectively unkillable shopkeeper.
- **bare-hand damage**: used when the mob wields nothing. `Z` becomes the
  mob's damroll and also applies with a weapon.
- **position / default position**: usually `8 8` (standing). A mob whose
  current position differs from its default is described from its position
  ("is sitting here") instead of its long description.
- **sex**: 0 neutral, 1 male, 2 female.
- S-format mobs get 11 in every ability, 10 mana, 50 movement, never get
  hungry or thirsty, and have saving throws of `max(20 - level, 2)`.

The **detailed format** has some other letter in place of `S`. It gives
abilities, hit point range, mana, movement, class, height and weight,
conditions and saving throws explicitly. Read `parse_mobile`
(`src/db.rs:543`) for the exact field order. No new mob needs it.

**Act flags** (`specials.act`; the game adds `ISNPC` itself):

| Bit | Name | Effect |
|----:|------|--------|
| 1   | `SPEC` | Call the mob's special procedure every mobile tick (10 s) |
| 2   | `SENTINEL` | Never wanders |
| 4   | `SCAVENGER` | Picks up the most valuable item in the room now and then |
| 16  | `NICE_THIEF` | When it catches a player stealing, it shouts "X is a bloody thief." instead of attacking |
| 32  | `AGGRESSIVE` | Attacks visible players on sight |
| 64  | `STAY_ZONE` | Wanders only within its zone |
| 128 | `WIMPY` | If aggressive, only attacks sleeping players |

A mob with `SPEC` (1) set but no special procedure attached logs
`Non-Existing MOB[N] SPEC procedure` and has the flag cleared. Command
interception (shopkeepers, guildmasters) works **without** `SPEC`. `SPEC`
only adds the periodic call. See [§12](#12-special-procedures).

**Affected_by flags**: permanent spell effects. The ones that are useful on
mobs: 2 `INVISIBLE`, 8 `DETECT_INVISIBLE`, 32 `SENSE_LIFE`, 128
`SANCTUARY`, 524288 `SNEAK`, 1048576 `HIDE`. The full list is in
[§16](#16-reference-tables).

**Alignment**: -1000 (evil) to 1000 (good). Below -350 counts as evil and
above 350 as good, which affects `ITEM_ANTI_*` gear, `detect evil` and
`dispel evil`.

**Mobile ticks**: every 10 seconds each awake mob that isn't fighting gets
a chance to act. A non-sentinel standing mob picks a random number from 0
to 45; if it's an exit number (0–5) and the move is allowed, it moves. It
won't go straight back the way it came.

---

## 9. Objects: `tinyworld.obj`

```
#<vnum>
<keywords>~
<short description>~
<long description (on the ground)>~
<action description>~
<type> <extra flags> <wear flags>
<value0> <value1> <value2> <value3>
<weight> <cost> <rent per day>
{ E
  <keywords>~
  <description>
  ~
| A
  <location> <modifier>
}*
```

Example (long sword, `#3022`):

```
#3022
sword long~
a long sword~
A long sword has been left here.~
slash~
5 0 8193
0 1 8 3
8 600 200
```

- **long description**: one line, `~` on the same line.
- **action description**: rarely used. Notes (`ITEM_NOTE`) keep their
  written text here.
- **wear flags**: `8193` = `TAKE` (1) + `WIELD` (8192). **Without `TAKE`
  (1) the object can't be picked up.**
- **cost**: shop base price. Shops won't trade items with a cost below 1.
  **Rent**: coins per day to store it at an inn. Negative rent counts as 0.
  Receptionists store at most 15 items per player.
- **`A` affects**: at most **2**. Extra `A` lines are ignored. Location
  codes are under [Apply types](#apply-types-a-lines--affect-locations).

### 9.1 Item types and their values

| # | Type | v0 | v1 | v2 | v3 |
|--:|------|----|----|----|----|
| 1 | LIGHT | – | – | hours (0 = burnt out, **negative = eternal**) | – |
| 2 | SCROLL | spell level | spell 1 | spell 2 | spell 3 (unused = -1) |
| 3 | WAND | spell level | max charges | charges left | spell |
| 4 | STAFF | spell level | max charges | charges left | spell |
| 5 | WEAPON | – | damage dice count | damage dice size | weapon type ↓ |
| 6 | FIREWEAPON | – | – | – | – |
| 7 | MISSILE | – | – | – | – |
| 8 | TREASURE | – | – | – | – |
| 9 | ARMOR | AC bonus (positive is better) | – | – | – |
| 10 | POTION | spell level | spell 1 | spell 2 | spell 3 |
| 11 | WORN | – | – | – | – |
| 12 | OTHER | – | – | – | – |
| 13 | TRASH | – | – | – | – |
| 14 | TRAP | – | – | – | – |
| 15 | CONTAINER | max weight | flags: 1 closeable, 2 pickproof, 4 closed, 8 locked | key vnum (-1 none) | 1 = corpse (internal) |
| 16 | NOTE | language (unused) | – | – | – |
| 17 | DRINKCON | capacity | current amount | liquid ↓ | non-zero = poisoned |
| 18 | KEY | key type | – | – | – |
| 19 | FOOD | hours of fullness | – | – | non-zero = poisoned |
| 20 | MONEY | number of coins | – | – | – |
| 21 | PEN | – | – | – | – |
| 22 | BOAT | – | – | – | – (just carrying one lets you cross sector 7) |

**Weapon types** (v3), as `hit()` interprets them (`src/fight.rs:468`):
0–2 whip, 3 slash, 4–6 crush, 7 pound (bludgeon), 8–11 pierce, anything
else "hit". **Thieves can only backstab with piercing weapons (type 11).**

**Liquids** (v2): 0 water, 1 beer, 2 wine, 3 ale, 4 dark ale, 5 whisky,
6 lemonade, 7 firebreather, 8 local speciality, 9 slime mold juice, 10 milk,
11 tea, 12 coffee, 13 blood, 14 salt water, 15 coke. The effects on
drunkenness, fullness and thirst are tabulated in `doc/values.doc`.

**Spells in scrolls, potions, wands and staves**: use the spell numbers in
[§16](#spell-numbers). Keep the level at the spell's design level. Damage
spells are balanced against the level they're first learned at, and
`doc/values.doc` warns that some low-level spells are meant only for that
level. It also has a table of which spells are valid in which item types.

### 9.2 Extra flags

| Bit | Name | Effect |
|----:|------|--------|
| 1 | `GLOW` | "(glowing)" |
| 2 | `HUM` | "(humming)" |
| 4 | `DARK` | (cosmetic) |
| 8 | `LOCK` | (unused) |
| 16 | `EVIL` | Shows red to `detect evil` |
| 32 | `INVISIBLE` | Needs detect invisible to see |
| 64 | `MAGIC` | Shows blue to `detect magic` |
| 128 | `NODROP` | Cursed: can't be dropped |
| 256 | `BLESS` | Blessed |
| 512 | `ANTI_GOOD` | Zaps good-aligned wearers |
| 1024 | `ANTI_EVIL` | Zaps evil-aligned wearers |
| 2048 | `ANTI_NEUTRAL` | Zaps neutral wearers |

### 9.3 Wear flags

`1 TAKE`, `2 FINGER`, `4 NECK`, `8 BODY`, `16 HEAD`, `32 LEGS`, `64 FEET`,
`128 HANDS`, `256 ARMS`, `512 SHIELD`, `1024 ABOUT` (body), `2048 WAIST`,
`4096 WRIST`, `8192 WIELD`, `16384 HOLD`, `32768 THROW`.

Light sources are equipped with `hold`, which puts them in the light
position.

---

## 10. Zones and resets: `tinyworld.zon`

### 10.1 Format

```
#<zone number>
<name>~
<top room vnum> <lifespan in minutes> <reset mode>
<reset commands, one per line>
S
```

The final record is `#99999` / `$~`. The zone number after `#` is
informational. Zones must be in ascending order of top room vnum.

### 10.2 Lifespan and reset modes

A zone tick runs every 60 real seconds (`PULSE_ZONE` = 240 pulses at 4
pulses a second). Each tick, every zone whose reset mode isn't 0 ages one
minute. When a zone's age passes its lifespan it joins the reset queue.

| Mode | Behaviour |
|-----:|-----------|
| 0 | Never resets after boot |
| 1 | Resets only when no player is in the zone |
| 2 | Resets as soon as its turn comes, players present or not |

**Only one zone resets per tick**, the first in the queue that's ready. A
mode-1 zone with players in it waits in the queue without blocking zones
behind it. All zones reset once at boot, regardless of mode.

### 10.3 Reset commands

Each line is `<cmd> <if-flag> <arg1> <arg2> [<arg3>]`. Anything after the
last number is a comment. Lines starting with `*` are comments.

| Cmd | Meaning | arg1 | arg2 | arg3 |
|:---:|---------|------|------|------|
| `M` | Load a mobile into a room | mob vnum | max in world | room vnum |
| `O` | Load an object onto a room floor | obj vnum | max in world | room vnum |
| `G` | Give an object to the last mobile loaded | obj vnum | max in world | – |
| `E` | Equip the last mobile loaded with an object | obj vnum | max in world | wear position |
| `P` | Put an object inside another object | obj vnum | max in world | container obj vnum |
| `D` | Set a door's state | room vnum | direction 0–5 | 0 open, 1 closed, 2 closed and locked |
| `S` | End of this zone's commands | | | |

**"max in world"** counts **every copy that exists right now**: in rooms,
carried or worn by players, inside containers. Copies stored at an inn
don't count while their owner is logged out. If the count is already at
`max`, the command does nothing and counts as **failed**.

**Extra rules** (`reset_zone`, `src/db.rs:771`):
- `O` also does nothing if a copy is already on that room's floor.
- `P` puts the object into the **first copy of the container found
  anywhere in the world** (newest first), not necessarily the one you just
  loaded. If no container exists, the new object is destroyed.
- `E` destroys the new object if that wear slot is already filled.
- `D` changes only the one side named. Write two `D` lines, one per room,
  for each door.

**The if-flag** chains commands together. With if-flag `1`, the command
runs only if the **previous command succeeded**. A skipped command also
counts as failed, so the failure carries down the chain. Always use `1` on
`G`, `E` and `P` that belong to an `M` or `O`:

```
M 0 3003 1 3011       The weaponsmith (one in the world, in room 3011)
G 1 3020 100          ...give him a dagger, only if he was just loaded
E 1 3022 100 16       ...and have him wield a long sword (position 16)
```

If you use `0` there and the `M` failed because the weaponsmith already
exists, the `G` gives the item to **whatever mob the last successful `M`
in this zone loaded**, or to nobody. The "last mob" isn't reset between
`M` lines.

**Wear positions for `E`**:
0 light, 1 right finger, 2 left finger, 3 neck 1, 4 neck 2, 5 body, 6 head,
7 legs, 8 feet, 9 hands, 10 arms, 11 shield, 12 about body, 13 waist,
14 right wrist, 15 left wrist, 16 wield, 17 hold.

### 10.4 Patterns

```
* Keys that should always be in a safe: load the safe, then the key into it
O 0 3005 1 3044       safe
P 1 3302 1 3005       key into the safe

* A locked door between 3110 (east exit) and 3111 (west exit)
D 0 3110 1 2
D 0 3111 3 2

* A pack of five rats sharing one room
M 0 7001 5 7025
M 0 7001 5 7025
M 0 7001 5 7025
```

Each `M` adds at most one copy, so for a group you need one line per mob,
all with the same max.

---

## 11. Shops: `tinyworld.shp`

A shop is a shopkeeper mob, a room, and a record in `tinyworld.shp`. The
keeper intercepts `buy`, `sell`, `value` and `list` (commands 56–59).

```
#<shop number>~
<producing 1>        object vnums the shop never runs out of
<producing 2>        (six lines; -1 = unused)
<producing 3>
<producing 4>
<producing 5>
<producing 6>
<profit_buy>         float >= 1.0: player pays cost × this
<profit_sell>        float <= 1.0: player receives cost × this
<type 1>             item types the shop buys (five lines, 0 = unused)
<type 2>
<type 3>
<type 4>
<type 5>
<no such item (buy)>~         %s = player name
<no such item (sell)>~
<shop doesn't trade that>~
<shop can't afford item>~
<player can't afford item>~
<buy message>~                %s name, %d price
<sell message>~               %s name, %d price
<temper1>            when the player can't pay: 0 spit, 1 smoke, other nothing
<temper2>            when attacked: 0 keeper strikes first, 1 keeper can't be hurt, other nothing
<keeper mob vnum>
<with who>           unused, 0
<shop room vnum>     the keeper only trades while in this room
<open1>              first opening period (mud hours 0–23;
<close1>               0 and 28 means always open)
<open2>              second period, used after close1
<close2>
```

> `doc/shops.doc` lists **five** producing slots. The data and the code
> (`MAX_PROD = 6`, `src/shop.rs:17`) use **six**. Follow the real file.

Example, shop `#0` (the shop at room 3009, whose keeper is mob 3001,
producing 3010 and 3011):

```
#0~
-1
-1
-1
-1
3010
3011
1.1
1.0
0
0
0
0
0
%s Haven't got that on storage - try list!~
...
0
0
3001
0
3009
0
28
0
0
```

Shopkeepers are attached to the `shop_keeper` special procedure when the
game boots (`assign_the_shopkeepers`). Items listed as **producing** are
never sold out: after each sale the keeper makes a fresh copy. Everything
else is the keeper's real inventory, stocked by the zone's `G` commands and
by what players sell. The keeper's gold is limited, so a shop can run out
of money to buy with.

Shop messages must start with `%s`: the keeper delivers each one as
`tell <message>`, so the player's name has to come first. `%d` marks the
price. The keeper won't trade with anyone they can't see.

---

## 12. Special procedures

A special procedure ("spec proc") is a Rust function attached to a mobile,
object or room prototype that can intercept commands and, for mobiles, act
on its own.

```rust
pub type SpecFn = fn(w: &mut World, ch: CharId, cmd: i32, arg: &str) -> bool;
```

### 12.1 When they run

1. **Command interception** (`special()`, `src/interpreter.rs:396`): before
   any command runs, the game offers it to, in order: the room's procedure,
   each equipped object, each carried object, each mob in the room, then
   each object on the floor. `ch` is **the character typing the command**,
   not the owner. `cmd` is the command number and `arg` the rest of the
   line. Return `true` to swallow the command, or `false` to let it run
   normally.
2. **Mobile ticks** (`mobile_activity`, every 10 s): mobs with act flag
   `SPEC` (1) get called with `cmd == 0`, and here `ch` **is** the mob.
   Return `true` to skip its normal wandering and aggression this tick.

Starting the server with `-s` turns all special procedures off (no shops,
guilds, receptionists or mob AI). It's useful for isolating a crash.

### 12.2 What's attached (`src/spec_assign.rs`)

| Kind | Vnum | Procedure | What it does |
|------|------|-----------|--------------|
| mob | 1 | `puff` | Says random lines |
| mob | 3060, 3067 | `cityguard` | Attacks evil fighters |
| mob | 3061 | `janitor` | Picks up litter |
| mob | 3062, 3066 | `fido` | Eats corpses |
| mob | 3005, 5007 | `receptionist` | `rent` and `offer` at inns |
| mob | 3020–3023 | `guild` | `practice` (guildmasters) |
| mob | 3024–3027 | `guild_guard` | Block other classes from entering guilds |
| mob | 3143 | `mayor` | Walks a route, opens and closes the gates |
| mob | 4000, 4001, 4053, 4102, 4158, 7006, 6113, 6114 | `snake` | Poisonous bite |
| mob | 4103, 4155 | `thief` | Steals gold |
| mob | 4100, 4157, 6910 | `magic_user` | Casts spells in combat |
| mob | 4150–4154, 7040 | `bat_*` | Breath weapons |
| mob | shopkeepers | `shop_keeper` | Assigned from the shop file |
| obj | 3099 | `board` | Bulletin board (`write`, `read`, `remove`, `look board`) |
| room | 3030 | `dump` | Destroys dropped items and pays a little |
| room | 3031 | `pet_shops` | Buy pets |
| room | 3054 | `pray_for_items` | Prayer room |
| room | 5039 | `worm_ritual` | The worm ritual |
| room | 8001 | `mar_gate` | Main-matrix gate |

### 12.3 Writing one

1. Write the function in `src/spec_procs.rs`:

   ```rust
   /// A fountain that refills drink containers when you `fill` (cmd n).
   pub fn healing_fountain(w: &mut World, ch: CharId, cmd: i32, _arg: &str) -> bool {
       if cmd != 0 {           // only react to the mob tick
           return false;
       }
       // `ch` is the mob here; heal everyone in its room a little
       let room = w[ch].in_room;
       for p in w.room(room).people.clone() {
           if w.ch_exists(p) && !w.is_npc(p) {
               let max = crate::limits::hit_limit(w, p);
               w[p].points.hit = (w[p].points.hit + 5).min(max);
           }
       }
       false                   // keep normal behaviour too
   }
   ```

2. Attach it to a vnum in `src/spec_assign.rs`:

   ```rust
   mob(w, 3070, healing_fountain, "healing_fountain");
   ```

   Objects go in `assign_objects` and rooms in `assign_rooms`.

3. For mob tick behaviour, set act flag `SPEC` (1) on the mob in
   `tinyworld.mob`.

Rules of the arena:
- Characters and objects are **handles** (`CharId`, `ObjId`) into
  generational arenas in `World`. After anything that can kill or destroy
  (`hit`, `damage`, `extract_char`, `extract_obj`, casting), check
  `w.ch_exists(id)` / `w.obj_exists(id)` before using the handle again.
  Using a dead handle panics, the game loop logs a `SYSERR`, and the pulse
  is abandoned.
- Iterate over `.clone()` copies of `people`, `contents` and `carrying`
  when the loop body might change them.
- Command numbers are positions in `CMD_INFO`. Look them up there (for
  example 1–6 directions, 15 look, 56 buy, 84 cast) and don't hardcode
  guesses.
- When intercepting, the procedure must find its owner itself. The room's
  `people` list and the mob's `nr` (rnum) are the usual route, as
  `shop_keeper` and `guild` do.
- A procedure attached to an object runs when the object is worn, carried
  or on the floor of the room the command is typed in.

---

## 13. Help, socials and other text files

**`help_table`**: entries are separated by a line starting with `#`, and
the file ends with `#~`. The first line of an entry holds its keywords,
which can be multi-word in double quotes (`"PICK LOCKS"`), and is also
displayed. `help <word>` matches by prefix, case-insensitive. Keywords are
sorted at boot, so entries can be in any order.

```
"PICK LOCKS"

Thieves only.
For picking locks. ...
#
```

**`actions`** (socials): each record starts with `<command number> <hide>
<min victim position>` and is followed by the message lines described in
`doc/actions.doc`. The command must already exist in `CMD_INFO` with
`do_action` as its function. Messages use `act()` codes ([§15.3](#153-act-codes)).

**`poses`**, **`messages`** (combat messages per attack type, including a
`god_msg` line used when the victim is immortal): see the files themselves
and `doc/`.

**`motd`**, **`news`**, **`info`**, **`credits`**, **`wizlist`**, **`help`**
(the screen for `help` with no argument): plain text, read at boot. After
editing, restart.

---

## 14. Building workflow and checklist

1. **Claim a number block.** Choose an unused range, normally 100 numbers
   (e.g. 9000–9099), for rooms, mobs and objects alike. Use the same
   numbers in all three files so a vnum tells you which zone it belongs to.
2. **Add the zone** to `tinyworld.zon` in order of top vnum (after
   `#22 The_holy_grove` for 9099). Start with mode 2 and a lifespan of
   15–30.
3. **Write rooms** in `tinyworld.wld`, in vnum order, before the final
   `#99999`. Make every passage two-way unless you mean it to be one-way.
   Connect the area to the existing world from a room in another zone.
4. **Write mobiles and objects** in vnum order.
5. **Write resets**: `M` with `G`/`E` (if-flag 1), `O`, `P`, and `D` for
   both sides of each door.
6. **Restart** and read the boot log:
   - `Room N is below zone Z` / `Room N is outside of any zone`: ordering
     or zone range problem. The server stops.
   - `Room N does not exist in database`: an exit or reset points at a
     missing room.
   - `fread_str: unexpected EOF`: a missing `~`.
   - `expected number near byte N`: a number field holds something else
     (count your lines).
   - The `Boot db -- DONE.` totals should rise by the amounts you added.
7. **Test in game:**
   ```
   goto 9000
   stat room
   load char 9001
   stat <mob>
   load obj 9010
   stat <obj>
   where <keyword>
   ```
   To test resets, leave the area and wait out the lifespan. Then check
   that the mobs carry their gear and the doors are shut and locked.

**Balance checklist**
- Mob experience fits its level. Compare with mobs of the same level
  elsewhere.
- Weapon dice match the rest of the world: a long sword is 1d8.
- AC on armour: body armour runs 1–5, smaller items 1–2.
- Shop items have `TAKE` and a sensible cost. Valuable items have rent.
- Death traps (`DEATH`) are mean. Give players a warning in the
  description of the room before.
- `NO_MOB` on rooms that wanderers shouldn't flood (shops, inns, temples).

---

## 15. For code implementers

### 15.1 Architecture in one screen

- `World` (`src/game.rs`) holds everything: `chars` and `objs` (generational
  arenas), `character_list` and `object_list`, `world` (rooms by rnum),
  `mob_index` and `obj_index` (prototypes, live counts and spec procs),
  `zone_table`, `shops`, `descriptors` and global state such as `wizlock`
  and `shutdown`. Index it with `w[ch]` or `w[obj]`.
- The game loop (`src/net.rs`) runs **4 pulses a second**: zones every 240
  (60 s), mobiles every 40 (10 s), violence every 12 (3 s), and every 300
  pulses (75 s, one mud hour) weather, affects and regeneration
  (`point_update`).
- One mud day is 24 mud hours (30 real minutes). Months have 35 days, and
  years 17 months.
- `handler.rs` has the list operations: `char_to_room`, `char_from_room`,
  `obj_to_char`, `equip_char`, `extract_char`, `extract_obj` and lookups
  (`get_char_vis`, `get_obj_vis`, `get_char`, `generic_find`). Use them
  and don't edit fields directly, because they keep weight, light and
  counts consistent.
- Player files are text, one per player, in `lib/plrs/` (`name.plr` and
  `name.objs`). `save.rs` reads and writes them.

### 15.2 Adding a command

1. Write `pub fn do_thing(w: &mut World, ch: CharId, argument: &str, cmd: i32)`.
2. Append a `Cmd { name, func, min_pos, min_level }` row to `CMD_INFO`.
   - **Append it, don't insert.** Command numbers are table indexes, and
     `lib/actions`, `lib/poses` and spec procs refer to them by number.
   - Prefix matching follows table order, so a new command at the end
     never takes over an existing abbreviation. It also means a short
     prefix of the new name may already belong to an older command.
   - A `min_level` of 21 or more makes it an immortal command, and
     `wizhelp` lists it automatically.
3. For an immortal command, start with `if w.is_npc(ch) { return; }`
   (switched immortals are NPCs), and `log()` anything that changes game
   state.
4. Add a `help_table` entry.

### 15.3 `act()` codes

`act(w, "text", hide_invisible, ch, obj, vict, to)` with `to` one of
`TO_ROOM`, `TO_VICT`, `TO_NOTVICT`, `TO_CHAR`.

| Code | Becomes | Code | Becomes |
|------|---------|------|---------|
| `$n` | ch's name (or "someone") | `$N` | victim's name |
| `$m` | him/her/it (ch) | `$M` | him/her/it (victim) |
| `$s` | his/her/its (ch) | `$S` | his/her/its (victim) |
| `$e` | he/she/it (ch) | `$E` | he/she/it (victim) |
| `$o` | obj's first keyword | `$O` | second obj's keyword |
| `$p` | obj's short description | `$P` | second obj's short description |
| `$a` | "a"/"an" for obj | `$A` | "a"/"an" for second obj |
| `$T` | text argument | `$F` | first word of text argument |
| `$$` | `$` | | |

### 15.4 Porting rules

The port keeps the original's behaviour, quirks included. Fix a bug only on
purpose, and list it under "Bugs fixed" in `README.md` when you do. Keep
command numbers, message texts and formulas unchanged unless that's the
point of the change.

---

## 16. Reference tables

### Apply types (`A` lines / affect locations)

| # | Apply | # | Apply |
|--:|-------|--:|-------|
| 0 | none | 13 | max hit points |
| 1 | STR | 14 | max movement |
| 2 | DEX | 15 | gold |
| 3 | INT | 16 | experience |
| 4 | WIS | 17 | AC (negative is better) |
| 5 | CON | 18 | hitroll |
| 6 | sex | 19 | damroll |
| 7 | class | 20 | save vs paralysis |
| 8 | level | 21 | save vs rod |
| 9 | age | 22 | save vs petrification |
| 10 | weight | 23 | save vs breath |
| 11 | height | 24 | save vs spell |
| 12 | max mana | | |

### Affected_by bits

1 blind, 2 invisible, 4 detect evil, 8 detect invisible, 16 detect magic,
32 sense life, 64 hold, 128 sanctuary, 256 group, 1024 curse, 2048 flaming,
4096 poison, 8192 protection from evil, 16384 paralysis, 32768 Morden's
sword, 65536 flaming sword, 131072 sleep, 262144 dodge, 524288 sneak,
1048576 hide, 2097152 fear, 4194304 charm, 8388608 follow.

### Spell numbers

| # | Spell | # | Spell | # | Spell |
|--:|-------|--:|-------|--:|-------|
| 1 | armor | 16 | cure light | 31 | locate object |
| 2 | teleport | 17 | curse | 32 | magic missile |
| 3 | bless | 18 | detect evil | 33 | poison |
| 4 | blindness | 19 | detect invisibility | 34 | protection from evil |
| 5 | burning hands | 20 | detect magic | 35 | remove curse |
| 6 | call lightning | 21 | detect poison | 36 | sanctuary |
| 7 | charm person | 22 | dispel evil | 37 | shocking grasp |
| 8 | chill touch | 23 | earthquake | 38 | sleep |
| 9 | clone | 24 | enchant weapon | 39 | strength |
| 10 | colour spray | 25 | energy drain | 40 | summon |
| 11 | control weather | 26 | fireball | 41 | ventriloquate |
| 12 | create food | 27 | harm | 42 | word of recall |
| 13 | create water | 28 | heal | 43 | remove poison |
| 14 | cure blind | 29 | invisibility | 44 | sense life |
| 15 | cure critic | 30 | lightning bolt | 53 | identify (scrolls only) |

Skills: 45 sneak, 46 hide, 47 steal, 48 backstab, 49 pick lock, 50 kick,
51 bash, 52 rescue.

### Positions

0 dead, 1 mortally wounded, 2 incapacitated, 3 stunned, 4 sleeping,
5 resting, 6 sitting, 7 fighting, 8 standing.

### NPC classes (detailed-format mobs)

0 other, 1 undead, 2 humanoid, 3 animal, 4 dragon, 5 giant.

### Further reading in `doc/`

`database.doc` and `dbsup.doc` (the original file format docs),
`values.doc` (item values and spell/item validity), `spell_info.doc`
(spell damage), `shops.doc`, `actions.doc`, `comm.doc` (`act()`),
`handler.doc`, `defs.doc` (the original world-building rules), `time.doc`.
Where they disagree with this guide, this guide follows the current code.
