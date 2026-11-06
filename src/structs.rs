//! Central data structures (structs.h).
//!
//! The C original links everything together with raw pointers and intrusive
//! linked lists.  Here characters and objects live in generational arenas
//! inside [`crate::game::World`] and refer to each other through the
//! copyable handles [`CharId`] and [`ObjId`].  Linked lists became `Vec`s;
//! where the C code prepended to a list we insert at index 0 so iteration
//! order (and therefore game behaviour) is preserved.

#![allow(dead_code)]

use crate::game::SpecFn;

pub const PULSE_ZONE: i64 = 240;
pub const PULSE_MOBILE: i64 = 40;
pub const PULSE_VIOLENCE: i64 = 12;
pub const WAIT_SEC: i32 = 4;
pub const WAIT_ROUND: i32 = 4;

pub const MAX_STRING_LENGTH: usize = 4096;
pub const MAX_INPUT_LENGTH: usize = 80;
pub const MAX_MESSAGES: usize = 60;
pub const MAX_ITEMS: usize = 153;

pub const MESS_ATTACKER: i32 = 1;
pub const MESS_VICTIM: i32 = 2;
pub const MESS_ROOM: i32 = 3;

pub const SECS_PER_REAL_MIN: i64 = 60;
pub const SECS_PER_REAL_HOUR: i64 = 60 * SECS_PER_REAL_MIN;
pub const SECS_PER_REAL_DAY: i64 = 24 * SECS_PER_REAL_HOUR;
pub const SECS_PER_REAL_YEAR: i64 = 365 * SECS_PER_REAL_DAY;

pub const SECS_PER_MUD_HOUR: i64 = 75;
pub const SECS_PER_MUD_DAY: i64 = 24 * SECS_PER_MUD_HOUR;
pub const SECS_PER_MUD_MONTH: i64 = 35 * SECS_PER_MUD_DAY;
pub const SECS_PER_MUD_YEAR: i64 = 17 * SECS_PER_MUD_MONTH;

/* item types */
pub const ITEM_LIGHT: i32 = 1;
pub const ITEM_SCROLL: i32 = 2;
pub const ITEM_WAND: i32 = 3;
pub const ITEM_STAFF: i32 = 4;
pub const ITEM_WEAPON: i32 = 5;
pub const ITEM_FIREWEAPON: i32 = 6;
pub const ITEM_MISSILE: i32 = 7;
pub const ITEM_TREASURE: i32 = 8;
pub const ITEM_ARMOR: i32 = 9;
pub const ITEM_POTION: i32 = 10;
pub const ITEM_WORN: i32 = 11;
pub const ITEM_OTHER: i32 = 12;
pub const ITEM_TRASH: i32 = 13;
pub const ITEM_TRAP: i32 = 14;
pub const ITEM_CONTAINER: i32 = 15;
pub const ITEM_NOTE: i32 = 16;
pub const ITEM_DRINKCON: i32 = 17;
pub const ITEM_KEY: i32 = 18;
pub const ITEM_FOOD: i32 = 19;
pub const ITEM_MONEY: i32 = 20;
pub const ITEM_PEN: i32 = 21;
pub const ITEM_BOAT: i32 = 22;

/* wear flags */
pub const ITEM_TAKE: i32 = 1;
pub const ITEM_WEAR_FINGER: i32 = 2;
pub const ITEM_WEAR_NECK: i32 = 4;
pub const ITEM_WEAR_BODY: i32 = 8;
pub const ITEM_WEAR_HEAD: i32 = 16;
pub const ITEM_WEAR_LEGS: i32 = 32;
pub const ITEM_WEAR_FEET: i32 = 64;
pub const ITEM_WEAR_HANDS: i32 = 128;
pub const ITEM_WEAR_ARMS: i32 = 256;
pub const ITEM_WEAR_SHIELD: i32 = 512;
pub const ITEM_WEAR_ABOUT: i32 = 1024;
pub const ITEM_WEAR_WAISTE: i32 = 2048;
pub const ITEM_WEAR_WRIST: i32 = 4096;
pub const ITEM_WIELD: i32 = 8192;
pub const ITEM_HOLD: i32 = 16384;
pub const ITEM_THROW: i32 = 32768;

/* extra flags */
pub const ITEM_GLOW: i32 = 1;
pub const ITEM_HUM: i32 = 2;
pub const ITEM_DARK: i32 = 4;
pub const ITEM_LOCK: i32 = 8;
pub const ITEM_EVIL: i32 = 16;
pub const ITEM_INVISIBLE: i32 = 32;
pub const ITEM_MAGIC: i32 = 64;
pub const ITEM_NODROP: i32 = 128;
pub const ITEM_BLESS: i32 = 256;
pub const ITEM_ANTI_GOOD: i32 = 512;
pub const ITEM_ANTI_EVIL: i32 = 1024;
pub const ITEM_ANTI_NEUTRAL: i32 = 2048;

/* liquids */
pub const LIQ_WATER: i32 = 0;
pub const LIQ_BEER: i32 = 1;
pub const LIQ_WINE: i32 = 2;
pub const LIQ_ALE: i32 = 3;
pub const LIQ_DARKALE: i32 = 4;
pub const LIQ_WHISKY: i32 = 5;
pub const LIQ_LEMONADE: i32 = 6;
pub const LIQ_FIREBRT: i32 = 7;
pub const LIQ_LOCALSPC: i32 = 8;
pub const LIQ_SLIME: i32 = 9;
pub const LIQ_MILK: i32 = 10;
pub const LIQ_TEA: i32 = 11;
pub const LIQ_COFFE: i32 = 12;
pub const LIQ_BLOOD: i32 = 13;
pub const LIQ_SALTWATER: i32 = 14;
pub const LIQ_COKE: i32 = 15;

/* containers - value[1] */
pub const CONT_CLOSEABLE: i32 = 1;
pub const CONT_PICKPROOF: i32 = 2;
pub const CONT_CLOSED: i32 = 4;
pub const CONT_LOCKED: i32 = 8;

#[derive(Clone, Debug, Default)]
pub struct ExtraDescr {
    pub keyword: String,
    pub description: String,
}

pub const MAX_OBJ_AFFECT: usize = 2;
pub const OBJ_NOTIMER: i32 = -7000000;

#[derive(Clone, Debug, Default)]
pub struct ObjFlagData {
    pub value: [i32; 4],
    pub type_flag: i32,
    pub wear_flags: i32,
    pub extra_flags: i32,
    pub weight: i32,
    pub cost: i32,
    pub cost_per_day: i32,
    pub timer: i32,
    pub bitvector: i64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ObjAffect {
    pub location: i32,
    pub modifier: i32,
}

/// Handle to a character in the world arena.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CharId {
    pub idx: u32,
    pub gen: u32,
}

/// Handle to an object in the world arena.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ObjId {
    pub idx: u32,
    pub gen: u32,
}

/// Descriptor (connection) handle - a unique, never reused number.
pub type DescId = u64;

#[derive(Clone, Debug, Default)]
pub struct Obj {
    pub item_number: i32,
    pub in_room: i32,
    pub obj_flags: ObjFlagData,
    pub affected: [ObjAffect; MAX_OBJ_AFFECT],
    pub name: String,
    pub description: String,
    pub short_description: String,
    pub action_description: String,
    pub ex_description: Vec<ExtraDescr>,
    pub carried_by: Option<CharId>,
    pub in_obj: Option<ObjId>,
    pub contains: Vec<ObjId>,
}

pub const NOWHERE: i32 = -1;

/* room flags */
pub const DARK: i32 = 1;
pub const DEATH: i32 = 2;
pub const NO_MOB: i32 = 4;
pub const INDOORS: i32 = 8;
pub const LAWFULL: i32 = 16;
pub const NEUTRAL: i32 = 32;
pub const CHAOTIC: i32 = 64;
pub const NO_MAGIC: i32 = 128;
pub const TUNNEL: i32 = 256;
pub const PRIVATE: i32 = 512;

pub const NORTH: i32 = 0;
pub const EAST: i32 = 1;
pub const SOUTH: i32 = 2;
pub const WEST: i32 = 3;
pub const UP: i32 = 4;
pub const DOWN: i32 = 5;

pub const EX_ISDOOR: i32 = 1;
pub const EX_CLOSED: i32 = 2;
pub const EX_LOCKED: i32 = 4;
pub const EX_RSCLOSED: i32 = 8;
pub const EX_RSLOCKED: i32 = 16;
pub const EX_PICKPROOF: i32 = 32;

pub const SECT_INSIDE: i32 = 0;
pub const SECT_CITY: i32 = 1;
pub const SECT_FIELD: i32 = 2;
pub const SECT_FOREST: i32 = 3;
pub const SECT_HILLS: i32 = 4;
pub const SECT_MOUNTAIN: i32 = 5;
pub const SECT_WATER_SWIM: i32 = 6;
pub const SECT_WATER_NOSWIM: i32 = 7;

#[derive(Clone, Debug, Default)]
pub struct RoomDirection {
    pub general_description: String,
    pub keyword: String,
    pub exit_info: i32,
    pub key: i32,
    pub to_room: i32,
}

#[derive(Clone, Default)]
pub struct Room {
    pub number: i32,
    pub zone: i32,
    pub sector_type: i32,
    pub name: String,
    pub description: String,
    pub ex_description: Vec<ExtraDescr>,
    pub dir_option: [Option<RoomDirection>; 6],
    pub room_flags: i32,
    pub light: i32,
    pub funct: Option<SpecFn>,
    pub contents: Vec<ObjId>,
    pub people: Vec<CharId>,
}

/* equipment positions */
pub const WEAR_LIGHT: usize = 0;
pub const WEAR_FINGER_R: usize = 1;
pub const WEAR_FINGER_L: usize = 2;
pub const WEAR_NECK_1: usize = 3;
pub const WEAR_NECK_2: usize = 4;
pub const WEAR_BODY: usize = 5;
pub const WEAR_HEAD: usize = 6;
pub const WEAR_LEGS: usize = 7;
pub const WEAR_FEET: usize = 8;
pub const WEAR_HANDS: usize = 9;
pub const WEAR_ARMS: usize = 10;
pub const WEAR_SHIELD: usize = 11;
pub const WEAR_ABOUT: usize = 12;
pub const WEAR_WAISTE: usize = 13;
pub const WEAR_WRIST_R: usize = 14;
pub const WEAR_WRIST_L: usize = 15;
pub const WIELD: usize = 16;
pub const HOLD: usize = 17;

pub const MAX_TOUNGE: usize = 3;
pub const MAX_SKILLS: usize = 53;
pub const MAX_WEAR: usize = 18;
pub const MAX_AFFECT: usize = 25;

pub const DRUNK: usize = 0;
pub const FULL: usize = 1;
pub const THIRST: usize = 2;

/* affected_by bits */
pub const AFF_BLIND: i64 = 1;
pub const AFF_INVISIBLE: i64 = 2;
pub const AFF_DETECT_EVIL: i64 = 4;
pub const AFF_DETECT_INVISIBLE: i64 = 8;
pub const AFF_DETECT_MAGIC: i64 = 16;
pub const AFF_SENSE_LIFE: i64 = 32;
pub const AFF_HOLD: i64 = 64;
pub const AFF_SANCTUARY: i64 = 128;
pub const AFF_GROUP: i64 = 256;
pub const AFF_CURSE: i64 = 1024;
pub const AFF_FLAMING: i64 = 2048;
pub const AFF_POISON: i64 = 4096;
pub const AFF_PROTECT_EVIL: i64 = 8192;
pub const AFF_PARALYSIS: i64 = 16384;
pub const AFF_MORDEN_SWORD: i64 = 32768;
pub const AFF_FLAMING_SWORD: i64 = 65536;
pub const AFF_SLEEP: i64 = 131072;
pub const AFF_DODGE: i64 = 262144;
pub const AFF_SNEAK: i64 = 524288;
pub const AFF_HIDE: i64 = 1048576;
pub const AFF_FEAR: i64 = 2097152;
pub const AFF_CHARM: i64 = 4194304;
pub const AFF_FOLLOW: i64 = 8388608;

/* apply types */
pub const APPLY_NONE: i32 = 0;
pub const APPLY_STR: i32 = 1;
pub const APPLY_DEX: i32 = 2;
pub const APPLY_INT: i32 = 3;
pub const APPLY_WIS: i32 = 4;
pub const APPLY_CON: i32 = 5;
pub const APPLY_SEX: i32 = 6;
pub const APPLY_CLASS: i32 = 7;
pub const APPLY_LEVEL: i32 = 8;
pub const APPLY_AGE: i32 = 9;
pub const APPLY_CHAR_WEIGHT: i32 = 10;
pub const APPLY_CHAR_HEIGHT: i32 = 11;
pub const APPLY_MANA: i32 = 12;
pub const APPLY_HIT: i32 = 13;
pub const APPLY_MOVE: i32 = 14;
pub const APPLY_GOLD: i32 = 15;
pub const APPLY_EXP: i32 = 16;
pub const APPLY_AC: i32 = 17;
pub const APPLY_ARMOR: i32 = 17;
pub const APPLY_HITROLL: i32 = 18;
pub const APPLY_DAMROLL: i32 = 19;
pub const APPLY_SAVING_PARA: i32 = 20;
pub const APPLY_SAVING_ROD: i32 = 21;
pub const APPLY_SAVING_PETRI: i32 = 22;
pub const APPLY_SAVING_BREATH: i32 = 23;
pub const APPLY_SAVING_SPELL: i32 = 24;

pub const CLASS_MAGIC_USER: i32 = 1;
pub const CLASS_CLERIC: i32 = 2;
pub const CLASS_THIEF: i32 = 3;
pub const CLASS_WARRIOR: i32 = 4;

pub const CLASS_OTHER: i32 = 0;
pub const CLASS_UNDEAD: i32 = 1;
pub const CLASS_HUMANOID: i32 = 2;
pub const CLASS_ANIMAL: i32 = 3;
pub const CLASS_DRAGON: i32 = 4;
pub const CLASS_GIANT: i32 = 5;

pub const SEX_NEUTRAL: i32 = 0;
pub const SEX_MALE: i32 = 1;
pub const SEX_FEMALE: i32 = 2;

pub const POSITION_DEAD: i32 = 0;
pub const POSITION_MORTALLYW: i32 = 1;
pub const POSITION_INCAP: i32 = 2;
pub const POSITION_STUNNED: i32 = 3;
pub const POSITION_SLEEPING: i32 = 4;
pub const POSITION_RESTING: i32 = 5;
pub const POSITION_SITTING: i32 = 6;
pub const POSITION_FIGHTING: i32 = 7;
pub const POSITION_STANDING: i32 = 8;

/* specials.act for mobiles */
pub const ACT_SPEC: i32 = 1;
pub const ACT_SENTINEL: i32 = 2;
pub const ACT_SCAVENGER: i32 = 4;
pub const ACT_ISNPC: i32 = 8;
pub const ACT_NICE_THIEF: i32 = 16;
pub const ACT_AGGRESSIVE: i32 = 32;
pub const ACT_STAY_ZONE: i32 = 64;
pub const ACT_WIMPY: i32 = 128;

/* specials.act for players */
pub const PLR_BRIEF: i32 = 1;
pub const PLR_NOSHOUT: i32 = 2;
pub const PLR_COMPACT: i32 = 4;
pub const PLR_DONTSET: i32 = 8;
pub const PLR_NOTELL: i32 = 16;
pub const PLR_NOEMOTE: i32 = 32;
pub const PLR_LOG: i32 = 64;
pub const PLR_FREEZE: i32 = 128;

#[derive(Clone, Copy, Debug, Default)]
pub struct TimeInfo {
    pub hours: i32,
    pub day: i32,
    pub month: i32,
    pub year: i32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TimeData {
    pub birth: i64,
    pub logon: i64,
    pub played: i64,
}

#[derive(Clone, Debug, Default)]
pub struct PlayerData {
    pub name: String,
    pub short_descr: String,
    pub long_descr: String,
    pub description: String,
    pub title: String,
    pub sex: i32,
    pub class: i32,
    pub level: i32,
    pub hometown: i32,
    pub talks: [bool; MAX_TOUNGE],
    pub time: TimeData,
    pub weight: i32,
    pub height: i32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Abilities {
    pub str: i32,
    pub str_add: i32,
    pub intel: i32,
    pub wis: i32,
    pub dex: i32,
    pub con: i32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Points {
    pub mana: i32,
    pub max_mana: i32,
    pub hit: i32,
    pub max_hit: i32,
    pub mov: i32,
    pub max_move: i32,
    pub armor: i32,
    pub gold: i32,
    pub exp: i32,
    pub hitroll: i32,
    pub damroll: i32,
}

#[derive(Clone, Debug, Default)]
pub struct Specials {
    pub fighting: Option<CharId>,
    pub hunting: Option<CharId>,
    pub affected_by: i64,
    pub position: i32,
    pub default_pos: i32,
    pub act: i32,
    pub spells_to_learn: i32,
    pub carry_weight: i32,
    pub carry_items: i32,
    pub timer: i32,
    pub was_in_room: i32,
    pub apply_saving_throw: [i32; 5],
    pub conditions: [i32; 3],
    pub damnodice: i32,
    pub damsizedice: i32,
    pub last_direction: i32,
    pub attack_type: i32,
    pub alignment: i32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Skill {
    pub learned: i32,
    pub recognise: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Affect {
    pub type_: i32,
    pub duration: i32,
    pub modifier: i32,
    pub location: i32,
    pub bitvector: i64,
}

#[derive(Clone, Debug)]
pub struct Char {
    pub nr: i32,
    pub in_room: i32,
    pub player: PlayerData,
    pub abilities: Abilities,
    pub tmpabilities: Abilities,
    pub points: Points,
    pub specials: Specials,
    pub skills: [Skill; MAX_SKILLS],
    pub affected: Vec<Affect>,
    pub equipment: [Option<ObjId>; MAX_WEAR],
    pub carrying: Vec<ObjId>,
    pub desc: Option<DescId>,
    pub followers: Vec<CharId>,
    pub master: Option<CharId>,
}

impl Default for Char {
    /// Equivalent of `clear_char()`.
    fn default() -> Self {
        let mut specials = Specials::default();
        specials.was_in_room = NOWHERE;
        specials.position = POSITION_STANDING;
        specials.default_pos = POSITION_STANDING;
        let mut points = Points::default();
        points.armor = 100;
        Char {
            nr: -1,
            in_room: NOWHERE,
            player: PlayerData::default(),
            abilities: Abilities::default(),
            tmpabilities: Abilities::default(),
            points,
            specials,
            skills: [Skill::default(); MAX_SKILLS],
            affected: Vec::new(),
            equipment: [None; MAX_WEAR],
            carrying: Vec::new(),
            desc: None,
            followers: Vec::new(),
            master: None,
        }
    }
}

pub const SUN_DARK: i32 = 0;
pub const SUN_RISE: i32 = 1;
pub const SUN_LIGHT: i32 = 2;
pub const SUN_SET: i32 = 3;

pub const SKY_CLOUDLESS: i32 = 0;
pub const SKY_CLOUDY: i32 = 1;
pub const SKY_RAINING: i32 = 2;
pub const SKY_LIGHTNING: i32 = 3;

#[derive(Clone, Copy, Debug, Default)]
pub struct Weather {
    pub pressure: i32,
    pub change: i32,
    pub sky: i32,
    pub sunlight: i32,
}

pub const MAX_OBJ_SAVE: usize = 15;

/* connection states */
pub const CON_PLYNG: i32 = 0;
pub const CON_NME: i32 = 1;
pub const CON_NMECNF: i32 = 2;
pub const CON_PWDNRM: i32 = 3;
pub const CON_PWDGET: i32 = 4;
pub const CON_PWDCNF: i32 = 5;
pub const CON_QSEX: i32 = 6;
pub const CON_RMOTD: i32 = 7;
pub const CON_SLCT: i32 = 8;
pub const CON_EXDSCR: i32 = 9;
pub const CON_QCLASS: i32 = 10;
pub const CON_LDEAD: i32 = 11;
pub const CON_PWDNEW: i32 = 12;
pub const CON_PWDNCNF: i32 = 13;

/// Where the string editor (`modify.c`) writes its result.
#[derive(Clone, Debug)]
pub enum StrTarget {
    CharDescription(CharId),
    CharTitle(CharId),
    CharName(CharId),
    CharShort(CharId),
    CharLong(CharId),
    ObjName(ObjId),
    ObjShort(ObjId),
    ObjLong(ObjId),
    ObjAction(ObjId),
    ObjExtra(ObjId, String),
    /// Note written with a pen: action_description of the paper.
    Note(ObjId),
    /// Bulletin board message being composed.
    Board(usize),
}

#[derive(Clone, Debug, Default)]
pub struct Snoop {
    pub snooping: Option<CharId>,
    pub snoop_by: Option<CharId>,
}

pub struct Descriptor {
    pub id: DescId,
    pub host: String,
    pub pwd: String,
    pub connected: i32,
    pub wait: i32,
    pub showstr: Option<(String, usize)>,
    pub str_target: Option<StrTarget>,
    pub str_buf: String,
    pub max_str: usize,
    pub prompt_mode: bool,
    pub last_input: String,
    pub output: Vec<String>,
    pub input: std::collections::VecDeque<String>,
    pub character: Option<CharId>,
    pub original: Option<CharId>,
    pub snoop: Snoop,
    /// false while the client should hide what the user types (passwords)
    pub echo: bool,
}

#[derive(Clone, Debug, Default)]
pub struct MsgType {
    pub attacker_msg: String,
    pub victim_msg: String,
    pub room_msg: String,
}

#[derive(Clone, Debug, Default)]
pub struct MessageType {
    pub die_msg: MsgType,
    pub miss_msg: MsgType,
    pub hit_msg: MsgType,
    pub sanctuary_msg: MsgType,
    pub god_msg: MsgType,
}

#[derive(Clone, Debug, Default)]
pub struct MessageList {
    pub a_type: i32,
    pub msg: Vec<MessageType>,
}

#[derive(Clone, Copy, Debug)]
pub struct DexSkill {
    pub p_pocket: i32,
    pub p_locks: i32,
    pub traps: i32,
    pub sneak: i32,
    pub hide: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct DexApp {
    pub reaction: i32,
    pub miss_att: i32,
    pub defensive: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct StrApp {
    pub tohit: i32,
    pub todam: i32,
    pub carry_w: i32,
    pub wield_w: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct ConApp {
    pub hitp: i32,
    pub shock: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct TitleType {
    pub title_m: &'static str,
    pub title_f: &'static str,
    pub exp: i32,
}
