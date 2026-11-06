//! Constant tables (constants.c).

#![allow(dead_code)]

use crate::structs::*;

pub const SPELL_WEAR_OFF_MSG: &[&str] = &[
    "RESERVED DB.C",
    "You feel less proctected.",
    "!Teleport!",
    "You feel less righteous.",
    "You feel a cloak of blindness disolve.",
    "!Burning Hands!",
    "!Call Lightning",
    "You feel more self-confident.",
    "!Chill Touch!",
    "!Clone!",
    "!Color Spray!",
    "!Control Weather!",
    "!Create Food!",
    "!Create Water!",
    "!Cure Blind!",
    "!Cure Critic!",
    "!Cure Light!",
    "You feel better.",
    "You sense the red in your vision disappear.",
    "The detect invisible wears off.",
    "The detect magic wears off.",
    "The detect poison wears off.",
    "!Dispel Evil!",
    "!Earthquake!",
    "!Enchant Weapon!",
    "!Energy Drain!",
    "!Fireball!",
    "!Harm!",
    "!Heal",
    "You feel yourself exposed.",
    "!Lightning Bolt!",
    "!Locate object!",
    "!Magic Missile!",
    "You feel less sick.",
    "You feel less protected.",
    "!Remove Curse!",
    "The white aura around your body fades.",
    "!Shocking Grasp!",
    "You feel les tired.",
    "You feel weaker.",
    "!Summon!",
    "!Ventriloquate!",
    "!Word of Recall!",
    "!Remove Poison!",
    "You feel less aware of your suroundings.",
    "",
    "!Hide!",
    "!Steal!",
    "!Backstab!",
    "!Pick Lock!",
    "!Kick!",
    "!Bash!",
    "!Rescue!",
    "!UNUSED!",
];

pub const REV_DIR: [i32; 6] = [2, 3, 0, 1, 5, 4];

pub const MOVEMENT_LOSS: [i32; 8] = [1, 2, 2, 3, 4, 6, 4, 1];

pub const DIRS: &[&str] = &["north", "east", "south", "west", "up", "down"];

pub const WEEKDAYS: [&str; 7] = [
    "the Day of the Moon",
    "the Day of the Bull",
    "the Day of the Deception",
    "the Day of Thunder",
    "the Day of Freedom",
    "the day of the Great Gods",
    "the Day of the Sun",
];

pub const MONTH_NAME: [&str; 17] = [
    "Month of Winter",
    "Month of the Winter Wolf",
    "Month of the Frost Giant",
    "Month of the Old Forces",
    "Month of the Grand Struggle",
    "Month of the Spring",
    "Month of Nature",
    "Month of Futility",
    "Month of the Dragon",
    "Month of the Sun",
    "Month of the Heat",
    "Month of the Battle",
    "Month of the Dark Shades",
    "Month of the Shadows",
    "Month of the Long Shadows",
    "Month of the Ancient Darkness",
    "Month of the Great Evil",
];

pub const SHARP: [i32; 12] = [0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0];

pub const WHERE: [&str; MAX_WEAR] = [
    "<used as light>      ",
    "<worn on finger>     ",
    "<worn on finger>     ",
    "<worn around neck>   ",
    "<worn around neck>   ",
    "<worn on body>       ",
    "<worn on head>       ",
    "<worn on legs>       ",
    "<worn on feet>       ",
    "<worn on hands>      ",
    "<worn on arms>       ",
    "<worn as shield>     ",
    "<worn about body>    ",
    "<worn about waist>   ",
    "<worn around wrist>  ",
    "<worn around wrist>  ",
    "<wielded>            ",
    "<held>               ",
];

pub const DRINKS: [&str; 16] = [
    "water",
    "beer",
    "wine",
    "ale",
    "dark ale",
    "whisky",
    "lemonade",
    "firebreather",
    "local speciality",
    "slime mold juice",
    "milk",
    "tea",
    "coffee",
    "blood",
    "salt water",
    "coca cola",
];

pub const DRINKNAMES: [&str; 16] = [
    "water", "beer", "wine", "ale", "ale", "whisky", "lemonade", "firebreather", "local", "juice",
    "milk", "tea", "coffee", "blood", "salt", "cola",
];

pub const DRINK_AFF: [[i32; 3]; 16] = [
    [0, 1, 10],
    [3, 2, 5],
    [5, 2, 5],
    [2, 2, 5],
    [1, 2, 5],
    [6, 1, 4],
    [0, 1, 8],
    [10, 0, 0],
    [3, 3, 3],
    [0, 4, -8],
    [0, 3, 6],
    [0, 1, 6],
    [0, 1, 6],
    [0, 2, -1],
    [0, 1, -2],
    [0, 1, 5],
];

pub const COLOR_LIQUID: [&str; 16] = [
    "clear",
    "brown",
    "clear",
    "brown",
    "dark",
    "golden",
    "red",
    "green",
    "clear",
    "light green",
    "white",
    "brown",
    "black",
    "red",
    "clear",
    "black",
];

pub const FULLNESS: [&str; 4] = ["less than half ", "about half ", "more than half ", ""];

macro_rules! t {
    ($m:expr, $f:expr, $e:expr) => {
        TitleType { title_m: $m, title_f: $f, exp: $e }
    };
}

pub const TITLES: [[TitleType; 25]; 4] = [
    [
        t!("the Man", "the Woman", 0),
        t!("the Apprentice of Magic", "the Apprentice of Magic", 1),
        t!("the Spell Student", "the Spell Student", 2500),
        t!("the Scholar of Magic", "the Scholar of Magic", 5000),
        t!("the Delver in Spells", "the Delveress in Spells", 10000),
        t!("the Medium of Magic", "the Medium of Magic", 20000),
        t!("the Scribe of Magic", "the Scribess of Magic", 40000),
        t!("the Seer", "the Seeress", 60000),
        t!("the Sage", "the Sage", 90000),
        t!("the Illusionist", "the Illusionist", 135000),
        t!("the Abjurer", "the Abjuress", 250000),
        t!("the Invoker", "the Invoker", 375000),
        t!("the Enchanter", "the Enchantress", 750000),
        t!("the Conjurer", "the Conjuress", 1125000),
        t!("the Magician", "the Witch", 1500000),
        t!("the Creator", "the Creator", 1875000),
        t!("the Savant", "the Savant", 2250000),
        t!("the Magus", "the Craftess", 2625000),
        t!("the Wizard", "the Wizard", 3000000),
        t!("the Warlock", "the War Witch", 3375000),
        t!("the Sorcerer", "the Sorceress", 3750000),
        t!("the Immortal Warlock", "the Immortal Enchantress", 4000000),
        t!("the Avatar of Magic", "the Empress of Magic", 5000000),
        t!("the God of magic", "the Goddess of magic", 6000000),
        t!("the Implementator", "the Implementress", 7000000),
    ],
    [
        t!("the Man", "the Woman", 0),
        t!("the Believer", "the Believer", 1),
        t!("the Attendant", "the Attendant", 1500),
        t!("the Acolyte", "the Acolyte", 3000),
        t!("the Novice", "the Novice", 6000),
        t!("the Missionary", "the Missionary", 13000),
        t!("the Adept", "the Adept", 27500),
        t!("the Deacon", "the Deaconess", 55000),
        t!("the Vicar", "the Vicaress", 110000),
        t!("the Priest", "the Priestess", 225000),
        t!("the Minister", "the Lady Minister", 450000),
        t!("the Canon", "the Canon", 675000),
        t!("the Levite", "the Levitess", 900000),
        t!("the Curate", "the Curess", 1125000),
        t!("the Monk", "the Nunne", 1350000),
        t!("the Healer", "the Healess", 1575000),
        t!("the Chaplain", "the Chaplain", 1800000),
        t!("the Expositor", "the Expositress", 2025000),
        t!("the Bishop", "the Bishop", 2250000),
        t!("the Arch Bishop", "the Arch Lady of the Church", 2475000),
        t!("the Patriarch", "the Matriarch", 2700000),
        t!("the Immortal Cardinal", "the Immortal Priestess", 3000000),
        t!("the Inquisitor", "the Inquisitress", 5000000),
        t!("the God of good and evil", "the Goddess of good and evil", 6000000),
        t!("the Implementator", "the Implementress", 7000000),
    ],
    [
        t!("the Man", "the Woman", 0),
        t!("the Pilferer", "the Pilferess", 1),
        t!("the Footpad", "the Footpad", 1250),
        t!("the Filcher", "the Filcheress", 2500),
        t!("the Pick-Pocket", "the Pick-Pocket", 5000),
        t!("the Sneak", "the Sneak", 10000),
        t!("the Pincher", "the Pincheress", 20000),
        t!("the Cut-Purse", "the Cut-Purse", 30000),
        t!("the Snatcher", "the Snatcheress", 70000),
        t!("the Sharper", "the Sharpress", 110000),
        t!("the Rogue", "the Rogue", 160000),
        t!("the Robber", "the Robber", 220000),
        t!("the Magsman", "the Magswoman", 440000),
        t!("the Highwayman", "the Highwaywoman", 660000),
        t!("the Burglar", "the Burglaress", 880000),
        t!("the Thief", "the Thief", 1100000),
        t!("the Knifer", "the Knifer", 1320000),
        t!("the Quick-Blade", "the Quick-Blade", 1540000),
        t!("the Killer", "the Murderess", 1760000),
        t!("the Brigand", "the Brigand", 1980000),
        t!("the Cut-Throat", "the Cut-Throat", 2200000),
        t!("the Immortal Assasin", "the Immortal Assasin", 2500000),
        t!("the Demi God of thieves", "the Demi Goddess of thieves", 5000000),
        t!("the God of thieves and tradesmen", "the Goddess of thieves and tradesmen", 6000000),
        t!("the Implementator", "the Implementress", 7000000),
    ],
    [
        t!("the Man", "the Woman", 0),
        t!("the Swordpupil", "the Swordpupil", 1),
        t!("the Recruit", "the Recruit", 2000),
        t!("the Sentry", "the Sentress", 4000),
        t!("the Fighter", "the Fighter", 8000),
        t!("the Soldier", "the Soldier", 16000),
        t!("the Warrior", "the Warrior", 32000),
        t!("the Veteran", "the Veteran", 64000),
        t!("the Swordsman", "the Swordswoman", 125000),
        t!("the Fencer", "the Fenceress", 250000),
        t!("the Combatant", "the Combatess", 500000),
        t!("the Hero", "the Heroine", 750000),
        t!("the Myrmidon", "the Myrmidon", 1000000),
        t!("the Swashbuckler", "the Swashbuckleress", 1250000),
        t!("the Mercenary", "the Mercenaress", 1500000),
        t!("the Swordmaster", "the Swordmistress", 1750000),
        t!("the Lieutenant", "the Lieutenant", 2000000),
        t!("the Champion", "the Lady Champion", 2250000),
        t!("the Dragoon", "the Lady Dragoon", 2500000),
        t!("the Cavalier", "the Cavalier", 2750000),
        t!("the Knight", "the Lady Knight", 3000000),
        t!("the Immortal Warlord", "the Immortal Lady of War", 3250000),
        t!("the Extirpator", "the Queen of Destruction", 5000000),
        t!("the God of war", "the Goddess of war", 6000000),
        t!("the Implementator", "the Implementress", 7000000),
    ],
];

pub const ITEM_TYPES: &[&str] = &[
    "UNDEFINED",
    "LIGHT",
    "SCROLL",
    "WAND",
    "STAFF",
    "WEAPON",
    "FIRE WEAPON",
    "MISSILE",
    "TREASURE",
    "ARMOR",
    "POTION",
    "WORN",
    "OTHER",
    "TRASH",
    "TRAP",
    "CONTAINER",
    "NOTE",
    "LIQUID CONTAINER",
    "KEY",
    "FOOD",
    "MONEY",
    "PEN",
    "BOAT",
];

pub const WEAR_BITS: &[&str] = &[
    "TAKE", "FINGER", "NECK", "BODY", "HEAD", "LEGS", "FEET", "HANDS", "ARMS", "SHIELD", "ABOUT",
    "WAISTE", "WRIST", "WIELD", "HOLD", "THROW", "LIGHT-SOURCE",
];

pub const EXTRA_BITS: &[&str] = &[
    "GLOW",
    "HUM",
    "DARK",
    "LOCK",
    "EVIL",
    "INVISIBLE",
    "MAGIC",
    "NODROP",
    "BLESS",
    "ANTI-GOOD",
    "ANTI-EVIL",
    "ANTI-NEUTRAL",
];

pub const ROOM_BITS: &[&str] = &[
    "DARK", "DEATH", "NO_MOB", "INDOORS", "LAWFULL", "NEUTRAL", "CHAOTOC", "NO_MAGIC", "TUNNEL",
    "PRIVATE",
];

pub const EXIT_BITS: &[&str] = &["IS-DOOR", "CLOSED", "LOCKED"];

pub const SECTOR_TYPES: &[&str] = &[
    "Inside",
    "City",
    "Field",
    "Forest",
    "Hills",
    "Mountains",
    "Water Swim",
    "Water NoSwim",
];

pub const EQUIPMENT_TYPES: &[&str] = &[
    "Special",
    "Worn on right finger",
    "Worn on left finger",
    "First worn around Neck",
    "Second worn around Neck",
    "Worn on body",
    "Worn on head",
    "Worn on legs",
    "Worn on feet",
    "Worn on hands",
    "Worn on arms",
    "Worn as shield",
    "Worn about body",
    "Worn around waiste",
    "Worn around right wrist",
    "Worn around left wrist",
    "Wielded",
    "Held",
];

pub const AFFECTED_BITS: &[&str] = &[
    "BLIND",
    "INVISIBLE",
    "DETECT-EVIL",
    "DETECT-INVISIBLE",
    "DETECT-MAGIC",
    "SENCE-LIFE",
    "HOLD",
    "SANCTUARY",
    "GROUP",
    "UNUSED",
    "CURSE",
    "FLAMING-HANDS",
    "POISON",
    "PROTECT-EVIL",
    "PARALYSIS",
    "MORDENS-SWORD",
    "FLAMING-SWORD",
    "SLEEP",
    "DODGE",
    "SNEAK",
    "HIDE",
    "FEAR",
    "CHARM",
    "FOLLOW",
    "SAVED_OBJECTS",
];

pub const APPLY_TYPES: &[&str] = &[
    "NONE",
    "STR",
    "DEX",
    "INT",
    "WIS",
    "CON",
    "SEX",
    "CLASS",
    "LEVEL",
    "AGE",
    "CHAR_WEIGHT",
    "CHAR_HEIGHT",
    "MANA",
    "HIT",
    "MOVE",
    "GOLD",
    "EXP",
    "ARMOR",
    "HITROLL",
    "DAMROLL",
    "SAVING_PARA",
    "SAVING_ROD",
    "SAVING_PETRI",
    "SAVING_BREATH",
    "SAVING_SPELL",
];

pub const PC_CLASS_TYPES: &[&str] = &["UNDEFINED", "Magic User", "Cleric", "Thief", "Warrior"];

pub const NPC_CLASS_TYPES: &[&str] = &["Normal", "Undead"];

pub const ACTION_BITS: &[&str] = &[
    "SPEC",
    "SENTINEL",
    "SCAVENGER",
    "ISNPC",
    "NICE-THIEF",
    "AGGRESSIVE",
    "STAY-ZONE",
    "WIMPY",
];

pub const PLAYER_BITS: &[&str] = &["BRIEF", "NOSHOUT", "COMPACT", "DONTSET", "NOTELL", "NOEMOTE", "", "FREEZE"];

pub const POSITION_TYPES: &[&str] = &[
    "Dead",
    "Mortally wounded",
    "Incapasitated",
    "Stunned",
    "Sleeping",
    "Resting",
    "Sitting",
    "Fighting",
    "Standing",
];

pub const CONNECTED_TYPES: &[&str] = &[
    "Playing",
    "Get name",
    "Confirm name",
    "Read Password",
    "Get new password",
    "Confirm new password",
    "Get sex",
    "Read messages of today",
    "Read Menu",
    "Get extra description",
    "Get class",
];

pub const THACO: [[i32; 25]; 4] = [
    [100, 20, 20, 20, 19, 19, 19, 18, 18, 18, 17, 17, 17, 16, 16, 16, 15, 15, 15, 14, 14, 14, 13, 13, 13],
    [100, 20, 20, 20, 18, 18, 18, 16, 16, 16, 14, 14, 14, 12, 12, 12, 10, 10, 10, 8, 8, 8, 6, 6, 6],
    [100, 20, 20, 19, 19, 18, 18, 17, 17, 16, 16, 15, 15, 14, 13, 13, 12, 12, 11, 11, 10, 10, 9, 9, 8],
    [100, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 1, 1, 1, 1],
];

macro_rules! s {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        StrApp { tohit: $a, todam: $b, carry_w: $c, wield_w: $d }
    };
}

pub const STR_APP: [StrApp; 31] = [
    s!(-5, -4, 0, 0),
    s!(-5, -4, 3, 1),
    s!(-3, -2, 3, 2),
    s!(-3, -1, 10, 3),
    s!(-2, -1, 25, 4),
    s!(-2, -1, 55, 5),
    s!(-1, 0, 80, 6),
    s!(-1, 0, 90, 7),
    s!(0, 0, 100, 8),
    s!(0, 0, 100, 9),
    s!(0, 0, 115, 10),
    s!(0, 0, 115, 11),
    s!(0, 0, 140, 12),
    s!(0, 0, 140, 13),
    s!(0, 0, 170, 14),
    s!(0, 0, 170, 15),
    s!(0, 1, 195, 16),
    s!(1, 1, 220, 18),
    s!(1, 2, 255, 20),
    s!(3, 7, 640, 40),
    s!(3, 8, 700, 40),
    s!(4, 9, 810, 40),
    s!(4, 10, 970, 40),
    s!(5, 11, 1130, 40),
    s!(6, 12, 1440, 40),
    s!(7, 14, 1750, 40),
    s!(1, 3, 280, 22),
    s!(2, 3, 305, 24),
    s!(2, 4, 330, 26),
    s!(2, 5, 380, 28),
    s!(3, 6, 480, 30),
];

macro_rules! d {
    ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr) => {
        DexSkill { p_pocket: $a, p_locks: $b, traps: $c, sneak: $d, hide: $e }
    };
}

pub const DEX_APP_SKILL: [DexSkill; 26] = [
    d!(-99, -99, -90, -99, -60),
    d!(-90, -90, -60, -90, -50),
    d!(-80, -80, -40, -80, -45),
    d!(-70, -70, -30, -70, -40),
    d!(-60, -60, -30, -60, -35),
    d!(-50, -50, -20, -50, -30),
    d!(-40, -40, -20, -40, -25),
    d!(-30, -30, -15, -30, -20),
    d!(-20, -20, -15, -20, -15),
    d!(-15, -10, -10, -20, -10),
    d!(-10, -5, -10, -15, -5),
    d!(-5, 0, -5, -10, 0),
    d!(0, 0, 0, -5, 0),
    d!(0, 0, 0, 0, 0),
    d!(0, 0, 0, 0, 0),
    d!(0, 0, 0, 0, 0),
    d!(0, 5, 0, 0, 0),
    d!(5, 10, 0, 5, 5),
    d!(10, 15, 5, 10, 10),
    d!(15, 20, 10, 15, 15),
    d!(15, 20, 10, 15, 15),
    d!(20, 25, 10, 15, 20),
    d!(20, 25, 15, 20, 20),
    d!(25, 25, 15, 20, 20),
    d!(25, 30, 15, 25, 25),
    d!(25, 30, 15, 25, 25),
];

pub const BACKSTAB_MULT: [i32; 25] = [1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5];

macro_rules! x {
    ($a:expr, $b:expr, $c:expr) => {
        DexApp { reaction: $a, miss_att: $b, defensive: $c }
    };
}

pub const DEX_APP: [DexApp; 26] = [
    x!(-7, -7, 6),
    x!(-6, -6, 5),
    x!(-4, -4, 5),
    x!(-3, -3, 4),
    x!(-2, -2, 3),
    x!(-1, -1, 2),
    x!(0, 0, 1),
    x!(0, 0, 0),
    x!(0, 0, 0),
    x!(0, 0, 0),
    x!(0, 0, 0),
    x!(0, 0, 0),
    x!(0, 0, 0),
    x!(0, 0, 0),
    x!(0, 0, 0),
    x!(0, 0, -1),
    x!(1, 1, -2),
    x!(2, 2, -3),
    x!(2, 2, -4),
    x!(3, 3, -4),
    x!(3, 3, -4),
    x!(4, 4, -5),
    x!(4, 4, -5),
    x!(4, 4, -5),
    x!(5, 5, -6),
    x!(5, 5, -6),
];

macro_rules! c {
    ($a:expr, $b:expr) => {
        ConApp { hitp: $a, shock: $b }
    };
}

pub const CON_APP: [ConApp; 26] = [
    c!(-4, 20),
    c!(-3, 25),
    c!(-2, 30),
    c!(-2, 35),
    c!(-1, 40),
    c!(-1, 45),
    c!(-1, 50),
    c!(0, 55),
    c!(0, 60),
    c!(0, 65),
    c!(0, 70),
    c!(0, 75),
    c!(0, 80),
    c!(0, 85),
    c!(0, 88),
    c!(1, 90),
    c!(2, 95),
    c!(2, 97),
    c!(3, 99),
    c!(3, 99),
    c!(4, 99),
    c!(5, 99),
    c!(5, 99),
    c!(5, 99),
    c!(6, 99),
    c!(7, 100),
];

/// int_app[].learn
pub const INT_APP: [i32; 26] = [3, 5, 7, 8, 9, 10, 11, 12, 13, 15, 17, 19, 22, 25, 30, 35, 40, 45, 50, 53, 55, 56, 60, 70, 80, 99];

/// wis_app[].bonus
pub const WIS_APP: [i32; 26] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 3, 3, 3, 4, 5, 6, 6, 6, 6, 6, 6, 6];
