//! Player and rent files.
//!
//! The C original wrote raw `struct char_file_u` / `struct obj_file_u`
//! records into two binary files, which is neither portable between
//! machines nor between compilers.  This port stores one plain-text file per
//! player in `lib/plrs/` (`<name>.plr` for the character, `<name>.objs`
//! for items left at the receptionist).  The *contents* mirror the old
//! records field for field.

#![allow(dead_code)]

use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use crate::game::World;
use crate::handler::*;
use crate::structs::*;
use crate::utils::*;

pub const PLAYER_DIR: &str = "plrs";

fn plr_path(name: &str, ext: &str) -> PathBuf {
    PathBuf::from(PLAYER_DIR).join(format!("{}.{}", name.to_ascii_lowercase(), ext))
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\n', "\\n").replace('\r', "\\r")
}

fn unesc(s: &str) -> String {
    let mut out = String::new();
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some(o) => out.push(o),
                None => {}
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// In-memory equivalent of `struct char_file_u`.
#[derive(Clone, Debug, Default)]
pub struct CharFile {
    pub name: String,
    pub pwd: String,
    pub sex: i32,
    pub class: i32,
    pub level: i32,
    pub birth: i64,
    pub played: i64,
    pub weight: i32,
    pub height: i32,
    pub title: String,
    pub hometown: i32,
    pub description: String,
    pub talks: [bool; MAX_TOUNGE],
    pub load_room: i32,
    pub abilities: Abilities,
    pub points: Points,
    pub skills: Vec<Skill>,
    pub affected: Vec<Affect>,
    pub spells_to_learn: i32,
    pub alignment: i32,
    pub last_logon: i64,
    pub act: i32,
    pub apply_saving_throw: [i32; 5],
    pub conditions: [i32; 3],
}

impl CharFile {
    fn serialize(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(s, "name {}", self.name);
        let _ = writeln!(s, "pwd {}", self.pwd);
        let _ = writeln!(s, "sex {}", self.sex);
        let _ = writeln!(s, "class {}", self.class);
        let _ = writeln!(s, "level {}", self.level);
        let _ = writeln!(s, "birth {}", self.birth);
        let _ = writeln!(s, "played {}", self.played);
        let _ = writeln!(s, "weight {}", self.weight);
        let _ = writeln!(s, "height {}", self.height);
        let _ = writeln!(s, "title {}", esc(&self.title));
        let _ = writeln!(s, "hometown {}", self.hometown);
        let _ = writeln!(s, "description {}", esc(&self.description));
        let _ = writeln!(s, "talks {} {} {}", self.talks[0] as i32, self.talks[1] as i32, self.talks[2] as i32);
        let _ = writeln!(s, "load_room {}", self.load_room);
        let a = &self.abilities;
        let _ = writeln!(s, "abilities {} {} {} {} {} {}", a.str, a.str_add, a.intel, a.wis, a.dex, a.con);
        let p = &self.points;
        let _ = writeln!(
            s,
            "points {} {} {} {} {} {} {} {} {} {} {}",
            p.mana, p.max_mana, p.hit, p.max_hit, p.mov, p.max_move, p.armor, p.gold, p.exp, p.hitroll, p.damroll
        );
        for (i, sk) in self.skills.iter().enumerate() {
            if sk.learned != 0 || sk.recognise {
                let _ = writeln!(s, "skill {} {} {}", i, sk.learned, sk.recognise as i32);
            }
        }
        for af in &self.affected {
            let _ = writeln!(s, "affect {} {} {} {} {}", af.type_, af.duration, af.modifier, af.location, af.bitvector);
        }
        let _ = writeln!(s, "spells_to_learn {}", self.spells_to_learn);
        let _ = writeln!(s, "alignment {}", self.alignment);
        let _ = writeln!(s, "last_logon {}", self.last_logon);
        let _ = writeln!(s, "act {}", self.act);
        let t = &self.apply_saving_throw;
        let _ = writeln!(s, "saving_throw {} {} {} {} {}", t[0], t[1], t[2], t[3], t[4]);
        let c = &self.conditions;
        let _ = writeln!(s, "conditions {} {} {}", c[0], c[1], c[2]);
        s
    }

    fn parse(text: &str) -> CharFile {
        let mut st = CharFile { skills: vec![Skill::default(); MAX_SKILLS], load_room: NOWHERE, ..Default::default() };
        for line in text.lines() {
            let (key, val) = line.split_once(' ').unwrap_or((line, ""));
            let nums: Vec<i64> = val.split_whitespace().map(|x| x.parse().unwrap_or(0)).collect();
            let n = |i: usize| nums.get(i).copied().unwrap_or(0) as i32;
            match key {
                "name" => st.name = val.to_string(),
                "pwd" => st.pwd = val.to_string(),
                "sex" => st.sex = n(0),
                "class" => st.class = n(0),
                "level" => st.level = n(0),
                "birth" => st.birth = nums.first().copied().unwrap_or(0),
                "played" => st.played = nums.first().copied().unwrap_or(0),
                "weight" => st.weight = n(0),
                "height" => st.height = n(0),
                "title" => st.title = unesc(val),
                "hometown" => st.hometown = n(0),
                "description" => st.description = unesc(val),
                "talks" => st.talks = [n(0) != 0, n(1) != 0, n(2) != 0],
                "load_room" => st.load_room = n(0),
                "abilities" => {
                    st.abilities = Abilities { str: n(0), str_add: n(1), intel: n(2), wis: n(3), dex: n(4), con: n(5) }
                }
                "points" => {
                    st.points = Points {
                        mana: n(0),
                        max_mana: n(1),
                        hit: n(2),
                        max_hit: n(3),
                        mov: n(4),
                        max_move: n(5),
                        armor: n(6),
                        gold: n(7),
                        exp: n(8),
                        hitroll: n(9),
                        damroll: n(10),
                    }
                }
                "skill" => {
                    let i = n(0) as usize;
                    if i < MAX_SKILLS {
                        st.skills[i] = Skill { learned: n(1), recognise: n(2) != 0 };
                    }
                }
                "affect" => st.affected.push(Affect {
                    type_: n(0),
                    duration: n(1),
                    modifier: n(2),
                    location: n(3),
                    bitvector: nums.get(4).copied().unwrap_or(0),
                }),
                "spells_to_learn" => st.spells_to_learn = n(0),
                "alignment" => st.alignment = n(0),
                "last_logon" => st.last_logon = nums.first().copied().unwrap_or(0),
                "act" => st.act = n(0),
                "saving_throw" => st.apply_saving_throw = [n(0), n(1), n(2), n(3), n(4)],
                "conditions" => st.conditions = [n(0), n(1), n(2)],
                _ => {}
            }
        }
        st
    }
}

pub fn build_player_index(w: &mut World) {
    let _ = fs::create_dir_all(PLAYER_DIR);
    w.player_table.clear();
    if let Ok(rd) = fs::read_dir(PLAYER_DIR) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) == Some("plr") {
                if let Some(stem) = p.file_stem().and_then(|x| x.to_str()) {
                    w.player_table.push(stem.to_ascii_lowercase());
                }
            }
        }
    }
    w.player_table.sort();
}

pub fn find_name(w: &World, name: &str) -> Option<usize> {
    w.player_table.iter().position(|n| str_cmp(n, name) == 0)
}

pub fn create_entry(w: &mut World, name: &str) {
    if find_name(w, name).is_none() {
        w.player_table.push(name.to_ascii_lowercase());
    }
}

pub fn load_char(w: &World, name: &str) -> Option<CharFile> {
    find_name(w, name)?;
    let text = fs::read_to_string(plr_path(name, "plr")).ok()?;
    Some(CharFile::parse(&text))
}

/// Copy data from the file structure to a character.
pub fn store_to_char(w: &mut World, st: &CharFile, ch: CharId) {
    {
        let c = &mut w[ch];
        c.player.sex = st.sex;
        c.player.class = st.class;
        c.player.level = st.level;
        c.player.short_descr.clear();
        c.player.long_descr.clear();
        c.player.title = st.title.clone();
        c.player.description = st.description.clone();
        c.player.hometown = st.hometown;
        c.player.time.birth = st.birth;
        c.player.time.played = st.played;
        c.player.time.logon = now();
        c.player.talks = st.talks;
        c.player.weight = st.weight;
        c.player.height = st.height;
        c.abilities = st.abilities;
        c.tmpabilities = st.abilities;
        c.points = st.points;
        for i in 0..MAX_SKILLS {
            c.skills[i] = st.skills.get(i).copied().unwrap_or_default();
        }
        c.specials.spells_to_learn = st.spells_to_learn;
        c.specials.alignment = st.alignment;
        c.specials.act = st.act;
        c.specials.carry_weight = 0;
        c.specials.carry_items = 0;
        c.points.armor = 100;
        c.points.hitroll = 0;
        c.points.damroll = 0;
        c.player.name = st.name.clone();
        c.specials.apply_saving_throw = st.apply_saving_throw;
        c.specials.conditions = st.conditions;
    }
    for af in st.affected.iter().take(MAX_AFFECT) {
        if af.type_ != 0 {
            affect_to_char(w, ch, af);
        }
    }
    w[ch].in_room = st.load_room;
    affect_total(w, ch);
}

/// Copy the vital data of a character to the file structure.
pub fn char_to_store(w: &mut World, ch: CharId) -> CharFile {
    let mut char_eq: [Option<ObjId>; MAX_WEAR] = [None; MAX_WEAR];
    for i in 0..MAX_WEAR {
        if w[ch].equipment[i].is_some() {
            char_eq[i] = unequip_char(w, ch, i);
        }
    }
    let affs: Vec<Affect> = w[ch].affected.iter().take(MAX_AFFECT).copied().collect();
    if w[ch].affected.len() > MAX_AFFECT {
        log("WARNING: OUT OF STORE ROOM FOR AFFECTED TYPES!!!");
    }
    for af in &affs {
        affect_modify(w, ch, af.location, af.modifier, af.bitvector, false);
    }
    let abil = w[ch].abilities;
    w[ch].tmpabilities = abil;

    let t = now();
    let c = &w[ch];
    let played = c.player.time.played + (t - c.player.time.logon);
    let mut st = CharFile {
        name: c.player.name.clone(),
        pwd: String::new(),
        sex: c.player.sex,
        class: c.player.class,
        level: c.player.level,
        birth: c.player.time.birth,
        played,
        weight: c.player.weight,
        height: c.player.height,
        title: c.player.title.clone(),
        hometown: c.player.hometown,
        description: c.player.description.clone(),
        talks: c.player.talks,
        load_room: NOWHERE,
        abilities: c.abilities,
        points: c.points,
        skills: c.skills.to_vec(),
        affected: affs.clone(),
        spells_to_learn: c.specials.spells_to_learn,
        alignment: c.specials.alignment,
        last_logon: t,
        act: c.specials.act,
        apply_saving_throw: c.specials.apply_saving_throw,
        conditions: c.specials.conditions,
    };
    st.points.armor = 100;
    st.points.hitroll = 0;
    st.points.damroll = 0;

    w[ch].player.time.played = played;
    w[ch].player.time.logon = t;

    for af in &affs {
        affect_modify(w, ch, af.location, af.modifier, af.bitvector, true);
    }
    for i in 0..MAX_WEAR {
        if let Some(o) = char_eq[i] {
            equip_char(w, ch, o, i);
        }
    }
    affect_total(w, ch);
    st
}

/// Write the vital data of a player to the player file.
pub fn save_char(w: &mut World, ch: CharId, load_room: i32) {
    if w.is_npc(ch) {
        return;
    }
    let Some(did) = w[ch].desc else { return };
    let pwd = match w.desc(did) {
        Some(d) => d.pwd.clone(),
        None => return,
    };
    let mut st = char_to_store(w, ch);
    st.load_room = load_room;
    st.pwd = pwd;
    let _ = fs::create_dir_all(PLAYER_DIR);
    let path = plr_path(&st.name, "plr");
    let tmp = path.with_extension("plr.tmp");
    let res = fs::write(&tmp, st.serialize()).and_then(|_| fs::rename(&tmp, &path));
    if let Err(e) = res {
        log(&format!("save char {}: {}", st.name, e));
    }
    let name = st.name.clone();
    create_entry(w, &name);
}

/* ------------------------------------------------------------------ */
/* rent file (pcobjs.obj in the original)                              */
/* ------------------------------------------------------------------ */

#[derive(Clone, Debug, Default)]
pub struct ObjFileElem {
    pub item_number: i32,
    pub value: [i32; 4],
    pub extra_flags: i32,
    pub weight: i32,
    pub timer: i32,
    pub bitvector: i64,
    pub affected: [ObjAffect; MAX_OBJ_AFFECT],
}

#[derive(Clone, Debug, Default)]
pub struct ObjFile {
    pub owner: String,
    pub gold_left: i32,
    pub total_cost: i32,
    pub last_update: i64,
    pub objects: Vec<ObjFileElem>,
}

impl ObjFile {
    fn serialize(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(s, "owner {}", self.owner);
        let _ = writeln!(s, "gold_left {}", self.gold_left);
        let _ = writeln!(s, "total_cost {}", self.total_cost);
        let _ = writeln!(s, "last_update {}", self.last_update);
        for o in &self.objects {
            let _ = writeln!(
                s,
                "obj {} {} {} {} {} {} {} {} {} {} {} {} {}",
                o.item_number,
                o.value[0],
                o.value[1],
                o.value[2],
                o.value[3],
                o.extra_flags,
                o.weight,
                o.timer,
                o.bitvector,
                o.affected[0].location,
                o.affected[0].modifier,
                o.affected[1].location,
                o.affected[1].modifier
            );
        }
        s
    }
    fn parse(text: &str) -> ObjFile {
        let mut st = ObjFile::default();
        for line in text.lines() {
            let (key, val) = line.split_once(' ').unwrap_or((line, ""));
            let nums: Vec<i64> = val.split_whitespace().map(|x| x.parse().unwrap_or(0)).collect();
            let n = |i: usize| nums.get(i).copied().unwrap_or(0) as i32;
            match key {
                "owner" => st.owner = val.to_string(),
                "gold_left" => st.gold_left = n(0),
                "total_cost" => st.total_cost = n(0),
                "last_update" => st.last_update = nums.first().copied().unwrap_or(0),
                "obj" => st.objects.push(ObjFileElem {
                    item_number: n(0),
                    value: [n(1), n(2), n(3), n(4)],
                    extra_flags: n(5),
                    weight: n(6),
                    timer: n(7),
                    bitvector: nums.get(8).copied().unwrap_or(0),
                    affected: [
                        ObjAffect { location: n(9), modifier: n(10) },
                        ObjAffect { location: n(11), modifier: n(12) },
                    ],
                }),
                _ => {}
            }
        }
        st
    }
}

pub fn read_obj_file(name: &str) -> Option<ObjFile> {
    fs::read_to_string(plr_path(name, "objs")).ok().map(|t| ObjFile::parse(&t))
}

pub fn write_obj_file(st: &ObjFile) {
    let _ = fs::create_dir_all(PLAYER_DIR);
    if let Err(e) = fs::write(plr_path(&st.owner, "objs"), st.serialize()) {
        log(&format!("saving PC's objects: {}", e));
    }
}

pub fn remove_obj_file(name: &str) {
    let _ = fs::remove_file(plr_path(name, "objs"));
}

/// Update a stored player record in place (used at boot for rent).
pub fn modify_stored_char<F: FnOnce(&mut CharFile)>(name: &str, f: F) {
    let path = plr_path(name, "plr");
    if let Ok(text) = fs::read_to_string(&path) {
        let mut st = CharFile::parse(&text);
        f(&mut st);
        let _ = fs::write(&path, st.serialize());
    }
}

/// All rent files, for `update_obj_file()` at boot.
pub fn all_obj_files() -> Vec<ObjFile> {
    let mut v = Vec::new();
    if let Ok(rd) = fs::read_dir(PLAYER_DIR) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) == Some("objs") {
                if let Ok(t) = fs::read_to_string(&p) {
                    v.push(ObjFile::parse(&t));
                }
            }
        }
    }
    v
}
