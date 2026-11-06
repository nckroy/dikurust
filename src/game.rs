//! The world: every piece of global state from the C sources, gathered
//! into one struct that is passed around as `&mut World`.

#![allow(dead_code)]

use std::collections::VecDeque;
use std::ops::{Index, IndexMut};

use crate::constants::STR_APP;
use crate::structs::*;

/// Special procedure: `int (*func)(struct char_data *ch, int cmd, char *arg)`.
pub type SpecFn = fn(&mut World, CharId, i32, &str) -> bool;

/* ------------------------------------------------------------------ */
/* generational arena                                                  */
/* ------------------------------------------------------------------ */

pub trait ArenaId: Copy {
    fn new(idx: u32, gen: u32) -> Self;
    fn idx(self) -> u32;
    fn gen(self) -> u32;
}

impl ArenaId for CharId {
    fn new(idx: u32, gen: u32) -> Self {
        CharId { idx, gen }
    }
    fn idx(self) -> u32 {
        self.idx
    }
    fn gen(self) -> u32 {
        self.gen
    }
}

impl ArenaId for ObjId {
    fn new(idx: u32, gen: u32) -> Self {
        ObjId { idx, gen }
    }
    fn idx(self) -> u32 {
        self.idx
    }
    fn gen(self) -> u32 {
        self.gen
    }
}

struct Slot<T> {
    gen: u32,
    val: Option<T>,
}

pub struct Arena<I: ArenaId, T> {
    slots: Vec<Slot<T>>,
    free: Vec<u32>,
    _p: std::marker::PhantomData<I>,
}

impl<I: ArenaId, T> Default for Arena<I, T> {
    fn default() -> Self {
        Arena { slots: Vec::new(), free: Vec::new(), _p: std::marker::PhantomData }
    }
}

impl<I: ArenaId, T> Arena<I, T> {
    pub fn insert(&mut self, v: T) -> I {
        if let Some(idx) = self.free.pop() {
            let s = &mut self.slots[idx as usize];
            s.gen = s.gen.wrapping_add(1);
            s.val = Some(v);
            I::new(idx, s.gen)
        } else {
            self.slots.push(Slot { gen: 0, val: Some(v) });
            I::new(self.slots.len() as u32 - 1, 0)
        }
    }
    pub fn remove(&mut self, id: I) -> Option<T> {
        let s = self.slots.get_mut(id.idx() as usize)?;
        if s.gen != id.gen() || s.val.is_none() {
            return None;
        }
        self.free.push(id.idx());
        s.val.take()
    }
    pub fn get(&self, id: I) -> Option<&T> {
        let s = self.slots.get(id.idx() as usize)?;
        if s.gen == id.gen() {
            s.val.as_ref()
        } else {
            None
        }
    }
    pub fn get_mut(&mut self, id: I) -> Option<&mut T> {
        let s = self.slots.get_mut(id.idx() as usize)?;
        if s.gen == id.gen() {
            s.val.as_mut()
        } else {
            None
        }
    }
    pub fn contains(&self, id: I) -> bool {
        self.get(id).is_some()
    }
}

/* ------------------------------------------------------------------ */
/* database tables                                                     */
/* ------------------------------------------------------------------ */

#[derive(Clone, Debug)]
pub struct ResetCom {
    pub command: char,
    pub if_flag: bool,
    pub arg1: i32,
    pub arg2: i32,
    pub arg3: i32,
}

#[derive(Clone, Debug)]
pub struct Zone {
    pub name: String,
    pub lifespan: i32,
    pub age: i32,
    pub top: i32,
    pub reset_mode: i32,
    pub cmd: Vec<ResetCom>,
}

/// How a mobile's hit points are rolled each time it is loaded.
#[derive(Clone, Copy, Debug)]
pub enum HpRoll {
    Dice(i32, i32, i32),
    Range(i32, i32),
}

#[derive(Clone)]
pub struct MobIndex {
    pub virtual_nr: i32,
    pub number: i32,
    pub func: Option<SpecFn>,
    /// name of the special procedure (function pointers can't be compared reliably)
    pub spec: &'static str,
    pub proto: Char,
    pub hp: HpRoll,
}

#[derive(Clone)]
pub struct ObjIndex {
    pub virtual_nr: i32,
    pub number: i32,
    pub func: Option<SpecFn>,
    pub proto: Obj,
}

#[derive(Clone, Debug, Default)]
pub struct SocialMessg {
    pub act_nr: i32,
    pub hide: bool,
    pub min_victim_position: i32,
    pub char_no_arg: String,
    pub others_no_arg: String,
    pub char_found: String,
    pub others_found: String,
    pub vict_found: String,
    pub not_found: String,
    pub char_auto: String,
    pub others_auto: String,
}

#[derive(Clone, Debug, Default)]
pub struct PoseType {
    pub level: i32,
    pub poser_msg: [String; 4],
    pub room_msg: [String; 4],
}

pub struct World {
    pub world: Vec<Room>,
    pub chars: Arena<CharId, Char>,
    pub objs: Arena<ObjId, Obj>,
    pub character_list: Vec<CharId>,
    pub object_list: Vec<ObjId>,
    pub combat_list: Vec<CharId>,

    pub zone_table: Vec<Zone>,
    pub reset_q: VecDeque<usize>,
    pub mob_index: Vec<MobIndex>,
    pub obj_index: Vec<ObjIndex>,

    pub descriptors: Vec<Descriptor>,
    pub next_desc_id: DescId,
    /// connections closed by the game: (id, output still to be flushed)
    pub closed: Vec<(DescId, Vec<String>)>,

    pub fight_messages: Vec<MessageList>,
    pub soc_mess_list: Vec<SocialMessg>,
    pub pose_messages: Vec<PoseType>,
    pub help_index: Vec<(String, String)>,

    pub credits: String,
    pub news: String,
    pub motd: String,
    pub help: String,
    pub info: String,
    pub wizlist: String,

    pub time_info: TimeInfo,
    pub weather_info: Weather,

    /// lower-case names of all players that have a save file
    pub player_table: Vec<String>,

    pub wizlock: bool,
    pub shutdown: bool,
    pub reboot: bool,
    pub no_specials: bool,

    pub shops: Vec<crate::shop::Shop>,
    pub board: crate::board::Board,
}

impl World {
    pub fn new() -> World {
        World {
            world: Vec::new(),
            chars: Arena::default(),
            objs: Arena::default(),
            character_list: Vec::new(),
            object_list: Vec::new(),
            combat_list: Vec::new(),
            zone_table: Vec::new(),
            reset_q: VecDeque::new(),
            mob_index: Vec::new(),
            obj_index: Vec::new(),
            descriptors: Vec::new(),
            next_desc_id: 1,
            closed: Vec::new(),
            fight_messages: Vec::new(),
            soc_mess_list: Vec::new(),
            pose_messages: Vec::new(),
            help_index: Vec::new(),
            credits: String::new(),
            news: String::new(),
            motd: String::new(),
            help: String::new(),
            info: String::new(),
            wizlist: String::new(),
            time_info: TimeInfo::default(),
            weather_info: Weather::default(),
            player_table: Vec::new(),
            wizlock: false,
            shutdown: false,
            reboot: false,
            no_specials: false,
            shops: Vec::new(),
            board: crate::board::Board::default(),
        }
    }

    /* ---------------- arena access ---------------- */

    pub fn ch_exists(&self, c: CharId) -> bool {
        self.chars.contains(c)
    }
    pub fn obj_exists(&self, o: ObjId) -> bool {
        self.objs.contains(o)
    }

    pub fn room(&self, r: i32) -> &Room {
        &self.world[r as usize]
    }
    pub fn room_mut(&mut self, r: i32) -> &mut Room {
        &mut self.world[r as usize]
    }

    pub fn desc(&self, id: DescId) -> Option<&Descriptor> {
        self.descriptors.iter().find(|d| d.id == id)
    }
    pub fn desc_mut(&mut self, id: DescId) -> Option<&mut Descriptor> {
        self.descriptors.iter_mut().find(|d| d.id == id)
    }

    /* ---------------- utils.h macros ---------------- */

    pub fn is_npc(&self, c: CharId) -> bool {
        self[c].specials.act & ACT_ISNPC != 0
    }
    pub fn is_mob(&self, c: CharId) -> bool {
        self.is_npc(c) && self[c].nr > -1
    }
    pub fn is_affected(&self, c: CharId, bit: i64) -> bool {
        self[c].specials.affected_by & bit != 0
    }
    pub fn get_pos(&self, c: CharId) -> i32 {
        self[c].specials.position
    }
    pub fn awake(&self, c: CharId) -> bool {
        self.get_pos(c) > POSITION_SLEEPING
    }
    pub fn level(&self, c: CharId) -> i32 {
        self[c].player.level
    }
    pub fn name(&self, c: CharId) -> &str {
        &self[c].player.name
    }
    pub fn in_room(&self, c: CharId) -> i32 {
        self[c].in_room
    }
    pub fn is_dark(&self, room: i32) -> bool {
        let r = self.room(room);
        r.light == 0 && r.room_flags & DARK != 0
    }
    pub fn is_light(&self, room: i32) -> bool {
        if room < 0 {
            return false;
        }
        let r = self.room(room);
        r.light != 0 || r.room_flags & DARK == 0
    }
    /// CAN_SEE(sub, obj)
    pub fn can_see(&self, sub: CharId, obj: CharId) -> bool {
        (!self.is_affected(obj, AFF_INVISIBLE) || self.is_affected(sub, AFF_DETECT_INVISIBLE))
            && !self.is_affected(sub, AFF_BLIND)
            && self.is_light(self[sub].in_room)
    }
    /// CAN_SEE_OBJ(sub, obj)
    pub fn can_see_obj(&self, sub: CharId, obj: ObjId) -> bool {
        (self[obj].obj_flags.extra_flags & ITEM_INVISIBLE == 0 || self.is_affected(sub, AFF_DETECT_INVISIBLE))
            && !self.is_affected(sub, AFF_BLIND)
            && self.is_light(self[sub].in_room)
    }
    /// PERS(ch, vict)
    pub fn pers(&self, ch: CharId, vict: CharId) -> String {
        if self.can_see(vict, ch) {
            if !self.is_npc(ch) {
                self[ch].player.name.clone()
            } else {
                self[ch].player.short_descr.clone()
            }
        } else {
            "someone".to_string()
        }
    }
    /// OBJS(obj, vict)
    pub fn objs(&self, obj: ObjId, vict: CharId) -> String {
        if self.can_see_obj(vict, obj) {
            self[obj].short_description.clone()
        } else {
            "something".to_string()
        }
    }
    /// OBJN(obj, vict)
    pub fn objn(&self, obj: ObjId, vict: CharId) -> String {
        if self.can_see_obj(vict, obj) {
            crate::handler::fname(&self[obj].name)
        } else {
            "something".to_string()
        }
    }
    pub fn hshr(&self, c: CharId) -> &'static str {
        match self[c].player.sex {
            0 => "its",
            1 => "his",
            _ => "her",
        }
    }
    pub fn hssh(&self, c: CharId) -> &'static str {
        match self[c].player.sex {
            0 => "it",
            1 => "he",
            _ => "she",
        }
    }
    pub fn hmhr(&self, c: CharId) -> &'static str {
        match self[c].player.sex {
            0 => "it",
            1 => "him",
            _ => "her",
        }
    }
    pub fn strength_apply_index(&self, c: CharId) -> usize {
        let a = &self[c].tmpabilities;
        let idx = if a.str_add == 0 || a.str != 18 {
            a.str
        } else if a.str_add <= 50 {
            26
        } else if a.str_add <= 75 {
            27
        } else if a.str_add <= 90 {
            28
        } else if a.str_add <= 99 {
            29
        } else {
            30
        };
        idx.clamp(0, 30) as usize
    }
    pub fn can_carry_w(&self, c: CharId) -> i32 {
        STR_APP[self.strength_apply_index(c)].carry_w
    }
    pub fn can_carry_n(&self, c: CharId) -> i32 {
        5 + self[c].tmpabilities.dex / 2 + self[c].player.level / 2
    }
    pub fn can_carry_obj(&self, c: CharId, o: ObjId) -> bool {
        self[c].specials.carry_weight + self[o].obj_flags.weight <= self.can_carry_w(c)
            && self[c].specials.carry_items + 1 <= self.can_carry_n(c)
    }
    pub fn can_get_obj(&self, c: CharId, o: ObjId) -> bool {
        self[o].obj_flags.wear_flags & ITEM_TAKE != 0 && self.can_carry_obj(c, o) && self.can_see_obj(c, o)
    }
    pub fn outside(&self, c: CharId) -> bool {
        self.room(self[c].in_room).room_flags & INDOORS == 0
    }
    pub fn exit(&self, c: CharId, door: usize) -> Option<&RoomDirection> {
        self.room(self[c].in_room).dir_option[door].as_ref()
    }
    pub fn exit_mut(&mut self, c: CharId, door: usize) -> Option<&mut RoomDirection> {
        let r = self[c].in_room;
        self.room_mut(r).dir_option[door].as_mut()
    }
    pub fn can_go(&self, c: CharId, door: usize) -> bool {
        match self.exit(c, door) {
            Some(e) => e.to_room != NOWHERE && e.exit_info & EX_CLOSED == 0,
            None => false,
        }
    }
    pub fn is_good(&self, c: CharId) -> bool {
        self[c].specials.alignment >= 350
    }
    pub fn is_evil(&self, c: CharId) -> bool {
        self[c].specials.alignment <= -350
    }
    pub fn is_neutral(&self, c: CharId) -> bool {
        !self.is_good(c) && !self.is_evil(c)
    }
    /// WAIT_STATE(ch, cycle)
    pub fn wait_state(&mut self, c: CharId, cycle: i32) {
        if let Some(d) = self[c].desc {
            if let Some(d) = self.desc_mut(d) {
                d.wait = cycle;
            }
        }
    }
    pub fn obj_type(&self, o: ObjId) -> i32 {
        self[o].obj_flags.type_flag
    }
    pub fn is_obj_stat(&self, o: ObjId, stat: i32) -> bool {
        self[o].obj_flags.extra_flags & stat != 0
    }
    pub fn can_wear(&self, o: ObjId, part: i32) -> bool {
        self[o].obj_flags.wear_flags & part != 0
    }
}

impl Index<CharId> for World {
    type Output = Char;
    fn index(&self, c: CharId) -> &Char {
        self.chars.get(c).expect("stale character handle")
    }
}
impl IndexMut<CharId> for World {
    fn index_mut(&mut self, c: CharId) -> &mut Char {
        self.chars.get_mut(c).expect("stale character handle")
    }
}
impl Index<ObjId> for World {
    type Output = Obj;
    fn index(&self, o: ObjId) -> &Obj {
        self.objs.get(o).expect("stale object handle")
    }
}
impl IndexMut<ObjId> for World {
    fn index_mut(&mut self, o: ObjId) -> &mut Obj {
        self.objs.get_mut(o).expect("stale object handle")
    }
}
