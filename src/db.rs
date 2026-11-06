//! Booting the world, loading prototypes and resetting zones (db.c).

#![allow(dead_code)]

use std::fs;

use crate::comm::{act, Vict, TO_ROOM};
use crate::game::*;
use crate::handler::*;
use crate::structs::*;
use crate::utils::*;

pub const WORLD_FILE: &str = "tinyworld.wld";
pub const MOB_FILE: &str = "tinyworld.mob";
pub const OBJ_FILE: &str = "tinyworld.obj";
pub const ZONE_FILE: &str = "tinyworld.zon";
pub const CREDITS_FILE: &str = "credits";
pub const NEWS_FILE: &str = "news";
pub const MOTD_FILE: &str = "motd";
pub const IDEA_FILE: &str = "ideas";
pub const TYPO_FILE: &str = "typos";
pub const BUG_FILE: &str = "bugs";
pub const MESS_FILE: &str = "messages";
pub const SOCMESS_FILE: &str = "actions";
pub const HELP_KWRD_FILE: &str = "help_table";
pub const HELP_PAGE_FILE: &str = "help";
pub const INFO_FILE: &str = "info";
pub const WIZLIST_FILE: &str = "wizlist";
pub const POSEMESS_FILE: &str = "poses";

pub const REAL: i32 = 0;
pub const VIRTUAL: i32 = 1;

pub const MENU: &str = "\n\rWelcome to DikuMUD\n\r\n\
0) Exit from DikuMud.\n\r\
1) Enter the game.\n\r\
2) Enter description.\n\r\
3) Read the background story\n\r\
4) Change password.\n\r\n\r   Make your choice: ";

pub const GREETINGS: &str = "\n\r\n\r                             DikuMUD I (GAMMA 0.0)\n\r\n\r                               Created by\n\r                   Hans Henrik Staerfeldt, Katja Nyboe,\n\r            Tom Madsen, Michael Seifert, and Sebastian Hammer\n\r\n\r";

pub const WELC_MESSG: &str = "\n\rWelcome to the land of DikuMUD. May your visit here be... Interesting.\n\r\n\r";

pub const STORY: &str = "This will soon be the background story of DIKU-MUD.\n\r\n\r";

/* ------------------------------------------------------------------ */
/* a small fscanf-alike over a whole file                              */
/* ------------------------------------------------------------------ */

pub struct Reader {
    data: Vec<u8>,
    pos: usize,
    name: String,
}

impl Reader {
    pub fn open(name: &str) -> std::io::Result<Reader> {
        let mut data = fs::read(name)?;
        data.retain(|&b| b != b'\r');
        Ok(Reader { data, pos: 0, name: name.to_string() })
    }
    pub fn eof(&self) -> bool {
        self.pos >= self.data.len()
    }
    pub fn peek(&self) -> Option<u8> {
        self.data.get(self.pos).copied()
    }
    pub fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_ascii_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }
    /// `%d`
    pub fn int(&mut self) -> i32 {
        self.skip_ws();
        let start = self.pos;
        if matches!(self.peek(), Some(b'-') | Some(b'+')) {
            self.pos += 1;
        }
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.pos += 1;
        }
        let s = String::from_utf8_lossy(&self.data[start..self.pos]).to_string();
        s.parse::<i64>().map(|v| v as i32).unwrap_or_else(|_| {
            log(&format!("{}: expected number near byte {}", self.name, start));
            0
        })
    }
    /// `%d` that leaves the input alone (like fscanf) when there is no number.
    pub fn try_int(&mut self) -> Option<i32> {
        self.skip_ws();
        let start = self.pos;
        let mut end = start;
        if matches!(self.data.get(end), Some(b'-') | Some(b'+')) {
            end += 1;
        }
        let digits = end;
        while matches!(self.data.get(end), Some(c) if c.is_ascii_digit()) {
            end += 1;
        }
        if end == digits {
            return None;
        }
        self.pos = end;
        String::from_utf8_lossy(&self.data[start..end]).parse::<i64>().ok().map(|v| v as i32)
    }
    /// `%c` after skipping blanks.
    pub fn letter(&mut self) -> char {
        self.skip_ws();
        let c = self.peek().unwrap_or(b'$');
        self.pos += 1;
        c as char
    }
    /// expect a specific character (after optional blanks)
    pub fn expect(&mut self, c: u8) -> bool {
        self.skip_ws();
        if self.peek() == Some(c) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    /// `%s`
    pub fn word(&mut self) -> String {
        self.skip_ws();
        let start = self.pos;
        while matches!(self.peek(), Some(c) if !c.is_ascii_whitespace()) {
            self.pos += 1;
        }
        String::from_utf8_lossy(&self.data[start..self.pos]).to_string()
    }
    /// fgets(): rest of the current line, without the newline.
    pub fn line(&mut self) -> Option<String> {
        if self.eof() {
            return None;
        }
        let start = self.pos;
        while let Some(c) = self.peek() {
            self.pos += 1;
            if c == b'\n' {
                return Some(String::from_utf8_lossy(&self.data[start..self.pos - 1]).to_string());
            }
        }
        Some(String::from_utf8_lossy(&self.data[start..self.pos]).to_string())
    }
    /// fread_string(): a `~`-terminated string; lines are joined with "\n\r".
    pub fn string(&mut self) -> String {
        let mut buf = String::new();
        loop {
            let Some(line) = self.line() else {
                log(&format!("fread_str: unexpected EOF in {}", self.name));
                return buf;
            };
            let trimmed = line.trim_end();
            if trimmed.ends_with('~') {
                let content = &trimmed[..trimmed.len() - 1];
                buf.push_str(content);
                return buf;
            }
            buf.push_str(&line);
            buf.push_str("\n\r");
        }
    }
}

/* ------------------------------------------------------------------ */
/* boot                                                                */
/* ------------------------------------------------------------------ */

pub fn file_to_string(name: &str) -> String {
    match fs::read_to_string(name) {
        Ok(s) => {
            let mut out = String::new();
            for line in s.replace('\r', "").split_inclusive('\n') {
                out.push_str(line);
                if line.ends_with('\n') {
                    out.push('\r');
                }
            }
            out
        }
        Err(e) => {
            log(&format!("file-to-string: {}: {}", name, e));
            String::new()
        }
    }
}

pub fn boot_db(w: &mut World) {
    log("Boot db -- BEGIN.");
    log("Resetting the game time:");
    reset_time(w);

    log("Reading newsfile, credits, help-page, info and motd.");
    w.news = file_to_string(NEWS_FILE);
    w.credits = file_to_string(CREDITS_FILE);
    w.motd = file_to_string(MOTD_FILE);
    w.help = file_to_string(HELP_PAGE_FILE);
    w.info = file_to_string(INFO_FILE);
    w.wizlist = file_to_string(WIZLIST_FILE);

    log("Loading help index.");
    build_help_index(w);

    log("Loading zone table.");
    boot_zones(w);

    log("Loading rooms.");
    boot_world(w);
    log("Renumbering rooms.");
    renum_world(w);

    log("Loading mobile and object prototypes.");
    boot_mobiles(w);
    boot_objects(w);

    log("Renumbering zone table.");
    renum_zone_table(w);

    log("Generating player index.");
    crate::save::build_player_index(w);

    log("Updating characters with saved items:");
    crate::reception::update_obj_file(w);

    log("Loading fight messages.");
    crate::fight::load_messages(w);

    log("Loading social messages.");
    crate::act_social::boot_social_messages(w);

    log("Loading pose messages.");
    crate::act_social::boot_pose_messages(w);

    log("Assigning function pointers:");
    if !w.no_specials {
        log("   Mobiles.");
        crate::spec_assign::assign_mobiles(w);
        log("   Objects.");
        crate::spec_assign::assign_objects(w);
        log("   Room.");
        crate::spec_assign::assign_rooms(w);
    }

    log("Loading the bulletin board.");
    crate::board::board_load(w);

    for i in 0..w.zone_table.len() {
        log(&format!(
            "Performing boot-time reset of {} (rooms {}-{}).",
            w.zone_table[i].name,
            if i > 0 { w.zone_table[i - 1].top + 1 } else { 0 },
            w.zone_table[i].top
        ));
        reset_zone(w, i);
    }
    w.reset_q.clear();
    log(&format!(
        "Boot db -- DONE. {} rooms, {} mobiles, {} objects, {} zones, {} shops, {} socials, {} help topics.",
        w.world.len(),
        w.mob_index.len(),
        w.obj_index.len(),
        w.zone_table.len(),
        w.shops.len(),
        w.soc_mess_list.len(),
        w.help_index.len()
    ));
}

pub fn reset_time(w: &mut World) {
    let beginning_of_time: i64 = 650336715;
    w.time_info = mud_time_passed(now(), beginning_of_time);
    w.weather_info.sunlight = match w.time_info.hours {
        0..=4 => SUN_DARK,
        5 => SUN_RISE,
        6..=20 => SUN_LIGHT,
        21 => SUN_SET,
        _ => SUN_DARK,
    };
    log(&format!(
        "   Current Gametime: {}H {}D {}M {}Y.",
        w.time_info.hours, w.time_info.day, w.time_info.month, w.time_info.year
    ));
    w.weather_info.pressure = 960;
    if w.time_info.month >= 7 && w.time_info.month <= 12 {
        w.weather_info.pressure += dice(1, 50);
    } else {
        w.weather_info.pressure += dice(1, 80);
    }
    w.weather_info.change = 0;
    w.weather_info.sky = if w.weather_info.pressure <= 980 {
        SKY_LIGHTNING
    } else if w.weather_info.pressure <= 1000 {
        SKY_RAINING
    } else if w.weather_info.pressure <= 1020 {
        SKY_CLOUDY
    } else {
        SKY_CLOUDLESS
    };
}

fn build_help_index(w: &mut World) {
    let Ok(text) = fs::read_to_string(HELP_KWRD_FILE) else {
        log("   Could not open help file.");
        return;
    };
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    let mut list: Vec<(String, String)> = Vec::new();
    while i < lines.len() {
        let kwline = lines[i];
        // collect the entry text (keyword line included, as in the C version)
        let mut body = String::new();
        let mut j = i;
        while j < lines.len() && !lines[j].starts_with('#') {
            body.push_str(lines[j].trim_end_matches('\r'));
            body.push_str("\n\r");
            j += 1;
        }
        for kw in split_words_quoted(kwline) {
            list.push((kw, body.clone()));
        }
        if j >= lines.len() || lines[j].starts_with("#~") {
            break;
        }
        i = j + 1;
    }
    list.sort_by(|a, b| str_cmp(&a.0, &b.0).cmp(&0));
    w.help_index = list;
}

/// one_word() repeatedly: words, or "quoted phrases", lower-cased.
pub fn split_words_quoted(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= b.len() {
            break;
        }
        let mut w = String::new();
        if b[i] == b'"' {
            i += 1;
            while i < b.len() && b[i] >= b' ' && b[i] != b'"' {
                w.push(b[i].to_ascii_lowercase() as char);
                i += 1;
            }
            if i < b.len() && b[i] == b'"' {
                i += 1;
            }
        } else {
            while i < b.len() && b[i] > b' ' {
                w.push(b[i].to_ascii_lowercase() as char);
                i += 1;
            }
        }
        if !w.is_empty() {
            out.push(w);
        }
    }
    out
}

fn boot_zones(w: &mut World) {
    let mut f = Reader::open(ZONE_FILE).unwrap_or_else(|e| {
        log(&format!("boot_zones: {}", e));
        std::process::exit(1);
    });
    loop {
        f.skip_ws();
        if f.peek() == Some(b'#') {
            f.line();
        }
        let check = f.string();
        if check.starts_with('$') || f.eof() {
            break;
        }
        let top = f.int();
        let lifespan = f.int();
        let reset_mode = f.int();
        let mut cmds = Vec::new();
        loop {
            let c = f.letter();
            if c == 'S' || f.eof() {
                cmds.push(ResetCom { command: 'S', if_flag: false, arg1: 0, arg2: 0, arg3: 0 });
                f.line();
                break;
            }
            if c == '*' {
                f.line();
                continue;
            }
            let if_flag = f.int() != 0;
            let arg1 = f.int();
            let arg2 = f.int();
            let arg3 = if matches!(c, 'M' | 'O' | 'E' | 'P' | 'D') { f.int() } else { 0 };
            f.line();
            cmds.push(ResetCom { command: c, if_flag, arg1, arg2, arg3 });
        }
        w.zone_table.push(Zone { name: check, lifespan, age: 0, top, reset_mode, cmd: cmds });
    }
}

fn boot_world(w: &mut World) {
    let mut f = Reader::open(WORLD_FILE).unwrap_or_else(|e| {
        log(&format!("boot_world: could not open world file: {}", e));
        std::process::exit(1);
    });
    let mut zone = 0usize;
    loop {
        f.skip_ws();
        if !f.expect(b'#') {
            break;
        }
        let virtual_nr = f.int();
        f.line();
        let name = f.string();
        if name.starts_with('$') {
            break;
        }
        let mut room = Room { number: virtual_nr, name, ..Default::default() };
        room.description = f.string();
        f.int(); // zone number in file is ignored
        if virtual_nr <= if zone > 0 { w.zone_table[zone - 1].top } else { -1 } {
            log(&format!("Room nr {} is below zone {}.", virtual_nr, zone));
            std::process::exit(1);
        }
        while virtual_nr > w.zone_table[zone].top {
            zone += 1;
            if zone >= w.zone_table.len() {
                log(&format!("Room {} is outside of any zone.", virtual_nr));
                std::process::exit(1);
            }
        }
        room.zone = zone as i32;
        room.room_flags = f.int();
        room.sector_type = f.int();
        loop {
            let chk = f.word();
            if let Some(rest) = chk.strip_prefix('D') {
                let dir = atoi(rest).clamp(0, 5) as usize;
                f.line();
                let general_description = f.string();
                let keyword = f.string();
                let tmp = f.int();
                let exit_info = match tmp {
                    1 => EX_ISDOOR,
                    2 => EX_ISDOOR | EX_PICKPROOF,
                    _ => 0,
                };
                let key = f.int();
                let to_room = f.int();
                room.dir_option[dir] = Some(RoomDirection { general_description, keyword, exit_info, key, to_room });
            } else if chk.starts_with('E') {
                f.line();
                let keyword = f.string();
                let description = f.string();
                room.ex_description.insert(0, ExtraDescr { keyword, description });
            } else if chk.starts_with('S') || chk.is_empty() {
                break;
            }
        }
        w.world.push(room);
    }
}

fn renum_world(w: &mut World) {
    for r in 0..w.world.len() {
        for d in 0..6 {
            let to = match &w.world[r].dir_option[d] {
                Some(e) if e.to_room != NOWHERE => e.to_room,
                _ => continue,
            };
            let real = real_room(w, to);
            w.world[r].dir_option[d].as_mut().unwrap().to_room = real;
        }
    }
}

fn renum_zone_table(w: &mut World) {
    for z in 0..w.zone_table.len() {
        for c in 0..w.zone_table[z].cmd.len() {
            let cmd = w.zone_table[z].cmd[c].clone();
            let mut n = cmd.clone();
            match cmd.command {
                'M' => {
                    n.arg1 = real_mobile(w, cmd.arg1);
                    n.arg3 = real_room(w, cmd.arg3);
                }
                'O' => {
                    n.arg1 = real_object(w, cmd.arg1);
                    if cmd.arg3 != NOWHERE {
                        n.arg3 = real_room(w, cmd.arg3);
                    }
                }
                'G' | 'E' => n.arg1 = real_object(w, cmd.arg1),
                'P' => {
                    n.arg1 = real_object(w, cmd.arg1);
                    n.arg3 = real_object(w, cmd.arg3);
                }
                'D' => n.arg1 = real_room(w, cmd.arg1),
                _ => {}
            }
            w.zone_table[z].cmd[c] = n;
        }
    }
}

fn boot_mobiles(w: &mut World) {
    let mut f = Reader::open(MOB_FILE).unwrap_or_else(|e| {
        log(&format!("boot: {}: {}", MOB_FILE, e));
        std::process::exit(1);
    });
    loop {
        f.skip_ws();
        if f.peek() == Some(b'$') || f.eof() {
            break;
        }
        if !f.expect(b'#') {
            log("boot_mobiles: format error");
            break;
        }
        let vnum = f.int();
        f.line();
        f.skip_ws();
        if f.peek() == Some(b'$') {
            break;
        }
        let (proto, hp) = parse_mobile(&mut f);
        w.mob_index.push(MobIndex { virtual_nr: vnum, number: 0, func: None, spec: "", proto, hp });
    }
}

fn parse_mobile(f: &mut Reader) -> (Char, HpRoll) {
    let mut mob = Char::default();
    mob.player.name = f.string();
    mob.player.short_descr = f.string();
    mob.player.long_descr = f.string();
    mob.player.description = f.string();

    mob.specials.act = f.int() | ACT_ISNPC;
    mob.specials.affected_by = f.int() as i64;
    mob.specials.alignment = f.int();
    let letter = f.letter();
    let hp;
    if letter == 'S' {
        mob.abilities = Abilities { str: 11, str_add: 0, intel: 11, wis: 11, dex: 11, con: 11 };
        mob.player.level = f.int();
        mob.points.hitroll = 20 - f.int();
        mob.points.armor = 10 * f.int();
        let a = f.int();
        f.expect(b'd');
        let b = f.int();
        f.expect(b'+');
        let c = f.int();
        hp = HpRoll::Dice(a, b, c);
        let a = f.int();
        f.expect(b'd');
        let b = f.int();
        f.expect(b'+');
        let c = f.int();
        mob.points.damroll = c;
        mob.specials.damnodice = a;
        mob.specials.damsizedice = b;
        mob.points.mana = 10;
        mob.points.max_mana = 10;
        mob.points.mov = 50;
        mob.points.max_move = 50;
        mob.points.gold = f.int();
        mob.points.exp = f.int();
        mob.specials.position = f.int();
        mob.specials.default_pos = f.int();
        mob.player.sex = f.int();
        mob.player.class = 0;
        mob.player.weight = 200;
        mob.player.height = 198;
        mob.specials.conditions = [-1, -1, -1];
        let st = (20 - mob.player.level).max(2);
        mob.specials.apply_saving_throw = [st; 5];
    } else {
        mob.abilities.str = f.int();
        mob.abilities.intel = f.int();
        mob.abilities.wis = f.int();
        mob.abilities.dex = f.int();
        mob.abilities.con = f.int();
        let lo = f.int();
        let hi = f.int();
        hp = HpRoll::Range(lo, hi);
        mob.points.armor = 10 * f.int();
        let m = f.int();
        mob.points.mana = m;
        mob.points.max_mana = m;
        let mv = f.int();
        mob.points.mov = mv;
        mob.points.max_move = mv;
        mob.points.gold = f.int();
        mob.points.exp = f.int();
        mob.specials.position = f.int();
        mob.specials.default_pos = f.int();
        mob.player.sex = f.int();
        mob.player.class = f.int();
        mob.player.level = f.int();
        f.int(); // age (ignored)
        mob.player.weight = f.int();
        mob.player.height = f.int();
        for i in 0..3 {
            mob.specials.conditions[i] = f.int();
        }
        for i in 0..5 {
            mob.specials.apply_saving_throw[i] = f.int();
        }
        mob.points.damroll = 0;
        mob.specials.damnodice = 1;
        mob.specials.damsizedice = 6;
        mob.points.hitroll = (mob.player.level - 3).max(1);
    }
    mob.tmpabilities = mob.abilities;
    (mob, hp)
}

fn boot_objects(w: &mut World) {
    let mut f = Reader::open(OBJ_FILE).unwrap_or_else(|e| {
        log(&format!("boot: {}: {}", OBJ_FILE, e));
        std::process::exit(1);
    });
    let mut pending: Option<i32> = None;
    loop {
        let vnum = match pending.take() {
            Some(v) => v,
            None => {
                f.skip_ws();
                if f.peek() == Some(b'$') || f.eof() {
                    break;
                }
                if !f.expect(b'#') {
                    log("boot_objects: format error");
                    break;
                }
                let v = f.int();
                f.line();
                v
            }
        };
        f.skip_ws();
        if f.peek() == Some(b'$') || f.eof() {
            break;
        }
        let mut obj = Obj { in_room: NOWHERE, ..Default::default() };
        obj.name = f.string();
        obj.short_description = f.string();
        obj.description = f.string();
        obj.action_description = f.string();
        obj.obj_flags.type_flag = f.int();
        obj.obj_flags.extra_flags = f.int();
        obj.obj_flags.wear_flags = f.int();
        for i in 0..4 {
            obj.obj_flags.value[i] = f.int();
        }
        obj.obj_flags.weight = f.int();
        obj.obj_flags.cost = f.int();
        obj.obj_flags.cost_per_day = f.int();
        // trailing E / A sections, then the next '#' record or '$'
        let mut naff = 0;
        loop {
            let chk = f.word();
            if chk.starts_with('E') {
                f.line();
                let keyword = f.string();
                let description = f.string();
                obj.ex_description.insert(0, ExtraDescr { keyword, description });
            } else if chk.starts_with('A') {
                let loc = f.int();
                let m = f.int();
                if naff < MAX_OBJ_AFFECT {
                    obj.affected[naff] = ObjAffect { location: loc, modifier: m };
                    naff += 1;
                }
            } else if let Some(n) = chk.strip_prefix('#') {
                pending = Some(atoi(n));
                f.line();
                break;
            } else {
                // '$' or EOF
                break;
            }
        }
        w.obj_index.push(ObjIndex { virtual_nr: vnum, number: 0, func: None, proto: obj });
        if pending.is_none() {
            break;
        }
    }
}

/* ------------------------------------------------------------------ */
/* instantiating prototypes                                            */
/* ------------------------------------------------------------------ */

pub fn read_mobile(w: &mut World, nr: i32, typ: i32) -> Option<CharId> {
    let nr = if typ == VIRTUAL { real_mobile(w, nr) } else { nr };
    if nr < 0 || nr as usize >= w.mob_index.len() {
        return None;
    }
    let idx = &w.mob_index[nr as usize];
    let mut mob = idx.proto.clone();
    let hp = match idx.hp {
        HpRoll::Dice(a, b, c) => dice(a, b) + c,
        HpRoll::Range(lo, hi) => number(lo, hi),
    };
    mob.points.max_hit = hp;
    mob.points.hit = hp;
    let t = now();
    mob.player.time.birth = t;
    mob.player.time.played = 0;
    mob.player.time.logon = t;
    mob.nr = nr;
    mob.desc = None;
    let id = w.chars.insert(mob);
    w.character_list.insert(0, id);
    w.mob_index[nr as usize].number += 1;
    Some(id)
}

pub fn read_object(w: &mut World, nr: i32, typ: i32) -> Option<ObjId> {
    let nr = if typ == VIRTUAL { real_object(w, nr) } else { nr };
    if nr < 0 || nr as usize >= w.obj_index.len() {
        return None;
    }
    let mut obj = w.obj_index[nr as usize].proto.clone();
    obj.in_room = NOWHERE;
    obj.carried_by = None;
    obj.in_obj = None;
    obj.contains.clear();
    obj.item_number = nr;
    let id = w.objs.insert(obj);
    w.object_list.insert(0, id);
    w.obj_index[nr as usize].number += 1;
    Some(id)
}

const ZO_DEAD: i32 = 999;

pub fn zone_update(w: &mut World) {
    for i in 0..w.zone_table.len() {
        let z = &mut w.zone_table[i];
        if z.age < z.lifespan && z.reset_mode != 0 {
            z.age += 1;
        } else if z.age < ZO_DEAD && z.reset_mode != 0 {
            w.reset_q.push_back(i);
            w.zone_table[i].age = ZO_DEAD;
        }
    }
    let q: Vec<usize> = w.reset_q.iter().copied().collect();
    for (pos, z) in q.into_iter().enumerate() {
        if w.zone_table[z].reset_mode == 2 || is_empty(w, z as i32) {
            reset_zone(w, z);
            w.reset_q.remove(pos);
            break;
        }
    }
}

pub fn reset_zone(w: &mut World, zone: usize) {
    let mut last_cmd = true;
    let mut mob: Option<CharId> = None;
    let cmds = w.zone_table[zone].cmd.clone();
    for (cmd_no, c) in cmds.iter().enumerate() {
        if c.command == 'S' {
            break;
        }
        if !(last_cmd || !c.if_flag) {
            last_cmd = false;
            continue;
        }
        match c.command {
            'M' => {
                if c.arg1 >= 0 && c.arg3 >= 0 && w.mob_index[c.arg1 as usize].number < c.arg2 {
                    mob = read_mobile(w, c.arg1, REAL);
                    if let Some(m) = mob {
                        char_to_room(w, m, c.arg3);
                    }
                    last_cmd = true;
                } else {
                    last_cmd = false;
                }
            }
            'O' => {
                if c.arg1 >= 0 && w.obj_index[c.arg1 as usize].number < c.arg2 {
                    if c.arg3 >= 0 {
                        if get_obj_in_list_num(w, c.arg1, &w.room(c.arg3).contents).is_none() {
                            if let Some(o) = read_object(w, c.arg1, REAL) {
                                obj_to_room(w, o, c.arg3);
                            }
                            last_cmd = true;
                        } else {
                            last_cmd = false;
                        }
                    } else {
                        if let Some(o) = read_object(w, c.arg1, REAL) {
                            w[o].in_room = NOWHERE;
                        }
                        last_cmd = true;
                    }
                } else {
                    last_cmd = false;
                }
            }
            'P' => {
                if c.arg1 >= 0 && w.obj_index[c.arg1 as usize].number < c.arg2 {
                    let obj = read_object(w, c.arg1, REAL);
                    let to = get_obj_num(w, c.arg3);
                    match (obj, to) {
                        (Some(o), Some(t)) => obj_to_obj(w, o, t),
                        (Some(o), None) => extract_obj(w, o),
                        _ => {}
                    }
                    last_cmd = true;
                } else {
                    last_cmd = false;
                }
            }
            'G' => {
                if c.arg1 >= 0 && w.obj_index[c.arg1 as usize].number < c.arg2 {
                    if let (Some(o), Some(m)) = (read_object(w, c.arg1, REAL), mob) {
                        if w.ch_exists(m) {
                            obj_to_char(w, o, m);
                        }
                    }
                    last_cmd = true;
                } else {
                    last_cmd = false;
                }
            }
            'E' => {
                if c.arg1 >= 0 && w.obj_index[c.arg1 as usize].number < c.arg2 {
                    if let (Some(o), Some(m)) = (read_object(w, c.arg1, REAL), mob) {
                        if w.ch_exists(m) && (c.arg3 as usize) < MAX_WEAR && w[m].equipment[c.arg3 as usize].is_none() {
                            equip_char(w, m, o, c.arg3 as usize);
                        } else {
                            extract_obj(w, o);
                        }
                    }
                    last_cmd = true;
                } else {
                    last_cmd = false;
                }
            }
            'D' => {
                if c.arg1 >= 0 {
                    if let Some(e) = w.world[c.arg1 as usize].dir_option[c.arg2.clamp(0, 5) as usize].as_mut() {
                        match c.arg3 {
                            0 => e.exit_info &= !(EX_LOCKED | EX_CLOSED),
                            1 => {
                                e.exit_info |= EX_CLOSED;
                                e.exit_info &= !EX_LOCKED;
                            }
                            2 => e.exit_info |= EX_LOCKED | EX_CLOSED,
                            _ => {}
                        }
                    }
                }
                last_cmd = true;
            }
            _ => {
                log(&format!("Undefd cmd in reset table; zone {} cmd {}.", zone, cmd_no));
            }
        }
    }
    w.zone_table[zone].age = 0;
}

/// Is the zone free of (connected, playing) players?
pub fn is_empty(w: &World, zone_nr: i32) -> bool {
    for d in &w.descriptors {
        if d.connected == CON_PLYNG {
            if let Some(c) = d.character {
                if w.ch_exists(c) && w[c].in_room >= 0 && w.room(w[c].in_room).zone == zone_nr {
                    return false;
                }
            }
        }
    }
    true
}

/* ------------------------------------------------------------------ */
/* character initialisation                                            */
/* ------------------------------------------------------------------ */

pub fn reset_char(w: &mut World, ch: CharId) {
    let c = &mut w[ch];
    c.equipment = [None; MAX_WEAR];
    c.followers.clear();
    c.master = None;
    c.carrying.clear();
    c.specials.fighting = None;
    c.specials.position = POSITION_STANDING;
    c.specials.default_pos = POSITION_STANDING;
    c.specials.carry_weight = 0;
    c.specials.carry_items = 0;
    if c.points.hit <= 0 {
        c.points.hit = 1;
    }
    if c.points.mov <= 0 {
        c.points.mov = 1;
    }
    if c.points.mana <= 0 {
        c.points.mana = 1;
    }
}

/// Initialise a brand new player character (class must be set).
pub fn init_char(w: &mut World, ch: CharId) {
    if w.player_table.is_empty() {
        // First player in the game becomes the implementor.
        w[ch].points.exp = 7000000;
        w[ch].player.level = 24;
    }
    crate::limits::set_title(w, ch);
    let t = now();
    {
        let c = &mut w[ch];
        c.player.short_descr.clear();
        c.player.long_descr.clear();
        c.player.description.clear();
        c.player.hometown = number(1, 4);
        c.player.time.birth = t;
        c.player.time.played = 0;
        c.player.time.logon = t;
        c.player.talks = [false; MAX_TOUNGE];
        c.tmpabilities.str = 9;
        c.tmpabilities.intel = 9;
        c.tmpabilities.wis = 9;
        c.tmpabilities.dex = 9;
        c.tmpabilities.con = 9;
        if c.player.sex == SEX_MALE {
            c.player.weight = number(120, 180);
            c.player.height = number(160, 200);
        } else {
            c.player.weight = number(100, 160);
            c.player.height = number(150, 180);
        }
    }
    let mm = crate::limits::mana_limit(w, ch);
    let mh = crate::limits::hit_limit(w, ch);
    let mv = crate::limits::move_limit(w, ch);
    let c = &mut w[ch];
    c.points.mana = mm;
    c.points.hit = mh;
    c.points.mov = mv;
    c.points.armor = 100;
    let god = c.player.level >= 24;
    for s in c.skills.iter_mut() {
        s.learned = if god { 100 } else { 0 };
        s.recognise = false;
    }
    c.specials.affected_by = 0;
    c.specials.spells_to_learn = 0;
    c.specials.apply_saving_throw = [0; 5];
    let cond = if god { -1 } else { 24 };
    c.specials.conditions = [cond; 3];
}

pub fn real_room(w: &World, vnum: i32) -> i32 {
    match w.world.binary_search_by(|r| r.number.cmp(&vnum)) {
        Ok(i) => i as i32,
        Err(_) => {
            log(&format!("Room {} does not exist in database", vnum));
            -1
        }
    }
}

pub fn real_mobile(w: &World, vnum: i32) -> i32 {
    match w.mob_index.binary_search_by(|m| m.virtual_nr.cmp(&vnum)) {
        Ok(i) => i as i32,
        Err(_) => -1,
    }
}

pub fn real_object(w: &World, vnum: i32) -> i32 {
    match w.obj_index.binary_search_by(|o| o.virtual_nr.cmp(&vnum)) {
        Ok(i) => i as i32,
        Err(_) => -1,
    }
}

/// Used by the "returned from linkdeath" code path in the game loop.
pub fn announce_return(w: &mut World, ch: CharId) {
    act(w, "$n has returned.", true, ch, None, Vict::None, TO_ROOM);
}
