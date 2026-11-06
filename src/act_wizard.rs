//! Immortal commands (act.wizard.c).

use crate::act_info::do_look;
use crate::comm::*;
use crate::constants::*;
use crate::db::{read_mobile, read_object, real_mobile, real_object, REAL};
use crate::fight::update_pos;
use crate::game::World;
use crate::handler::*;
use crate::interpreter::{argument_interpreter, command_interpreter, half_chop, is_abbrev, one_argument};
use crate::limits::*;
use crate::spell_parser::SPELLS;
use crate::spells::*;
use crate::structs::*;
use crate::utils::*;

pub fn do_emote(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        return;
    }
    if w[ch].specials.act & PLR_NOEMOTE != 0 {
        send_to_char(w, "You can't show your emotions!!\n\r", ch);
        return;
    }
    let a = argument.trim_start_matches(' ');
    if a.is_empty() {
        send_to_char(w, "Yes.. But what?\n\r", ch);
    } else {
        let buf = format!("$n {}", a);
        act(w, &buf, false, ch, None, Vict::None, TO_ROOM);
        send_to_char(w, "Ok.\n\r", ch);
    }
}

pub fn do_echo(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        return;
    }
    let a = argument.trim_start_matches(' ');
    if a.is_empty() {
        send_to_char(w, "That must be a mistake...\n\r", ch);
    } else {
        let buf = format!("{}\n\r", a);
        let r = w[ch].in_room;
        send_to_room_except(w, &buf, r, ch);
        send_to_char(w, "Ok.\n\r", ch);
    }
}

fn transfer(w: &mut World, ch: CharId, victim: CharId) {
    act(w, "$n disappears in a mushroom cloud.", false, victim, None, Vict::None, TO_ROOM);
    let target = w[ch].in_room;
    char_from_room(w, victim);
    char_to_room(w, victim, target);
    act(w, "$n arrives from a puff of smoke.", false, victim, None, Vict::None, TO_ROOM);
    act(w, "$n has transferred you!", false, ch, None, Vict::Ch(victim), TO_VICT);
    do_look(w, victim, "", 15);
}

pub fn do_trans(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        return;
    }
    let (buf, _) = one_argument(argument);
    if buf.is_empty() {
        send_to_char(w, "Who do you wich to transfer?\n\r", ch);
    } else if str_cmp("all", &buf) != 0 {
        match get_char_vis(w, ch, &buf) {
            None => send_to_char(w, "No-one by that name around.\n\r", ch),
            Some(v) => {
                transfer(w, ch, v);
                send_to_char(w, "Ok.\n\r", ch);
            }
        }
    } else {
        let targets: Vec<CharId> = w
            .descriptors
            .iter()
            .filter(|d| d.connected == CON_PLYNG)
            .filter_map(|d| d.character)
            .filter(|&c| c != ch)
            .collect();
        for v in targets {
            if w.ch_exists(v) && w[v].in_room >= 0 {
                transfer(w, ch, v);
            }
        }
        send_to_char(w, "Ok.\n\r", ch);
    }
}

/// Resolve a room number / character / object name to a room index.
fn find_location(w: &mut World, ch: CharId, loc: &str) -> Option<i32> {
    if loc.as_bytes().first().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        let nr = atoi(loc);
        match w.world.iter().position(|r| r.number == nr) {
            Some(p) => Some(p as i32),
            None => {
                send_to_char(w, "No room exists with that number.\n\r", ch);
                None
            }
        }
    } else if let Some(m) = get_char_vis(w, ch, loc) {
        Some(w[m].in_room)
    } else if let Some(o) = get_obj_vis(w, ch, loc) {
        if w[o].in_room != NOWHERE {
            Some(w[o].in_room)
        } else {
            send_to_char(w, "The object is not available.\n\r", ch);
            None
        }
    } else {
        send_to_char(w, "No such creature or object around.\n\r", ch);
        None
    }
}

pub fn do_at(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        return;
    }
    let (loc_str, command) = half_chop(argument);
    if loc_str.is_empty() {
        send_to_char(w, "You must supply a room number or a name.\n\r", ch);
        return;
    }
    let Some(location) = find_location(w, ch, &loc_str) else { return };
    if location < 0 {
        return;
    }
    let original_loc = w[ch].in_room;
    char_from_room(w, ch);
    char_to_room(w, ch, location);
    command_interpreter(w, ch, &command);
    if w.ch_exists(ch) && w[ch].in_room == location {
        char_from_room(w, ch);
        char_to_room(w, ch, original_loc);
    }
}

pub fn do_goto(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        return;
    }
    let (buf, _) = one_argument(argument);
    if buf.is_empty() {
        send_to_char(w, "You must supply a room number or a name.\n\r", ch);
        return;
    }
    let Some(location) = find_location(w, ch, &buf) else { return };
    if location < 0 {
        return;
    }
    if w.room(location).room_flags & PRIVATE != 0 && w.room(location).people.len() > 1 {
        send_to_char(w, "There's a private conversation going on in that room.\n\r", ch);
        return;
    }
    act(w, "$n disappears in a puff of smoke.", false, ch, None, Vict::None, TO_ROOM);
    char_from_room(w, ch);
    char_to_room(w, ch, location);
    act(w, "$n appears with an ear-splitting bang.", false, ch, None, Vict::None, TO_ROOM);
    do_look(w, ch, "", 15);
}

pub fn do_stat(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        return;
    }
    let (arg1, _) = one_argument(argument);
    if arg1.is_empty() {
        send_to_char(w, "Stats on who or what?\n\r", ch);
        return;
    }
    let mut out = String::new();
    macro_rules! p { ($($t:tt)*) => {{ out.push_str(&format!($($t)*)); }} }

    if str_cmp("room", &arg1) == 0 {
        let ri = w[ch].in_room;
        let rm = w.room(ri);
        p!("Room name: {}, Of zone : {}. V-Number : {}, R-number : {}\n\r", rm.name, rm.zone, rm.number, ri);
        p!("Sector type : {}", sprinttype(rm.sector_type, SECTOR_TYPES));
        p!("Special procedure : {}", if rm.funct.is_some() { "Exists\n\r" } else { "No\n\r" });
        p!("Room flags: {}\n\r", sprintbit(rm.room_flags as i64, ROOM_BITS));
        p!("Description:\n\r{}", rm.description);
        p!("Extra description keywords(s): ");
        if !rm.ex_description.is_empty() {
            p!("\n\r");
            for d in &rm.ex_description {
                p!("{}\n\r", d.keyword);
            }
            p!("\n\r");
        } else {
            p!("None\n\r");
        }
        p!("------- Chars present -------\n\r");
        for &k in &rm.people {
            let t = if !w.is_npc(k) {
                "(PC)\n\r"
            } else if !w.is_mob(k) {
                "(NPC)\n\r"
            } else {
                "(MOB)\n\r"
            };
            p!("{}{}", w[k].player.name, t);
        }
        p!("\n\r--------- Contents ---------\n\r");
        for &j in &rm.contents {
            p!("{}\n\r", w[j].name);
        }
        p!("\n\r------- Exits defined -------\n\r");
        for i in 0..6 {
            if let Some(e) = &rm.dir_option[i] {
                p!("Direction {} . Keyword : {}\n\r", DIRS[i], e.keyword);
                p!("Description:\n\r  ");
                if !e.general_description.is_empty() {
                    p!("{}", e.general_description);
                } else {
                    p!("UNDEFINED\n\r");
                }
                let to = if e.to_room >= 0 { w.room(e.to_room).number } else { -1 };
                p!(
                    "Exit flag: {} \n\rKey no: {}\n\rTo room (V-Number): {}\n\r",
                    sprintbit(e.exit_info as i64, EXIT_BITS),
                    e.key,
                    to
                );
            }
        }
        send_to_char(w, &out, ch);
        return;
    }

    if let Some(j) = get_obj_vis(w, ch, &arg1) {
        let o = w[j].clone();
        let vnum = if o.item_number >= 0 { w.obj_index[o.item_number as usize].virtual_nr } else { 0 };
        p!(
            "Object name: [{}], R-number: [{}], V-number: [{}] Item type: {}\n\r",
            o.name,
            o.item_number,
            vnum,
            sprinttype(o.obj_flags.type_flag, ITEM_TYPES)
        );
        p!(
            "Short description: {}\n\rLong description:\n\r{}\n\r",
            if o.short_description.is_empty() { "None" } else { &o.short_description },
            if o.description.is_empty() { "None" } else { &o.description }
        );
        if !o.ex_description.is_empty() {
            p!("Extra description keyword(s):\n\r----------\n\r");
            for d in &o.ex_description {
                p!("{}\n\r", d.keyword);
            }
            p!("----------\n\r");
        } else {
            p!("Extra description keyword(s): None\n\r");
        }
        p!("Can be worn on :{}\n\r", sprintbit(o.obj_flags.wear_flags as i64, WEAR_BITS));
        p!("Set char bits  :{}\n\r", sprintbit(o.obj_flags.bitvector, AFFECTED_BITS));
        p!("Extra flags: {}\n\r", sprintbit(o.obj_flags.extra_flags as i64, EXTRA_BITS));
        p!(
            "Weight: {}, Value: {}, Cost/day: {}, Timer: {}\n\r",
            o.obj_flags.weight,
            o.obj_flags.cost,
            o.obj_flags.cost_per_day,
            o.obj_flags.timer
        );
        p!("In room: ");
        if o.in_room == NOWHERE {
            p!("Nowhere");
        } else {
            p!("{}", w.room(o.in_room).number);
        }
        p!(" ,In object: {}", o.in_obj.map(|io| fname(&w[io].name)).unwrap_or_else(|| "None".into()));
        p!(" ,Carried by:{}\n\r", o.carried_by.map(|c| w[c].player.name.clone()).unwrap_or_else(|| "Nobody".into()));
        let v = o.obj_flags.value;
        match o.obj_flags.type_flag {
            ITEM_LIGHT => p!("Colour : [{}]\n\rType : [{}]\n\rHours : [{}]", v[0], v[1], v[2]),
            ITEM_SCROLL | ITEM_POTION => p!("Spells : {}, {}, {}, {}", v[0], v[1], v[2], v[3]),
            ITEM_WAND | ITEM_STAFF => p!("Spell : {}\n\rMana : {}", v[0], v[1]),
            ITEM_WEAPON | ITEM_FIREWEAPON => p!("Tohit : {}\n\rTodam : {}D{}\n\rType : {}", v[0], v[1], v[2], v[3]),
            ITEM_MISSILE => p!("Tohit : {}\n\rTodam : {}\n\rType : {}", v[0], v[1], v[3]),
            ITEM_ARMOR => p!("AC-apply : [{}]", v[0]),
            ITEM_TRAP => p!("Spell : {}\n\r- Hitpoints : {}", v[0], v[1]),
            ITEM_CONTAINER => p!(
                "Max-contains : {}\n\rLocktype : {}\n\rCorpse : {}",
                v[0],
                v[1],
                if v[3] != 0 { "Yes" } else { "No" }
            ),
            ITEM_DRINKCON => p!(
                "Max-contains : {}\n\rContains : {}\n\rPoisoned : {}\n\rLiquid : {}",
                v[0],
                v[1],
                v[3],
                sprinttype(v[2], &DRINKS)
            ),
            ITEM_NOTE => p!("Tounge : {}", v[0]),
            ITEM_KEY => p!("Keytype : {}", v[0]),
            ITEM_FOOD => p!("Makes full : {}\n\rPoisoned : {}", v[0], v[3]),
            _ => p!("Values 0-3 : [{}] [{}] [{}] [{}]", v[0], v[1], v[2], v[3]),
        }
        p!("\n\rEquipment Status: ");
        match o.carried_by {
            None => p!("NONE"),
            Some(c) => match (0..MAX_WEAR).find(|&i| w[c].equipment[i] == Some(j)) {
                Some(i) => p!("{}", sprinttype(i as i32, EQUIPMENT_TYPES)),
                None => p!("Inventory"),
            },
        }
        let sp = o.item_number >= 0 && w.obj_index[o.item_number as usize].func.is_some();
        p!("\n\rSpecial procedure : {}", if sp { "exists\n\r" } else { "No\n\r" });
        // (the C code always ended up printing "Nothing" here)
        if o.contains.is_empty() {
            p!("Contains : Nothing\n\r");
        } else {
            p!("Contains :\n\r");
            for &c in &o.contains {
                p!("{}\n\r", fname(&w[c].name));
            }
        }
        p!("Can affect char :\n\r");
        for a in o.affected.iter() {
            p!("    Affects : {} By {}\n\r", sprinttype(a.location, APPLY_TYPES), a.modifier);
        }
        send_to_char(w, &out, ch);
        return;
    }

    let Some(k) = get_char_vis(w, ch, &arg1) else {
        send_to_char(w, "No mobile or object by that name in the world\n\r", ch);
        return;
    };
    let c = w[k].clone();
    p!(
        "{}",
        match c.player.sex {
            SEX_NEUTRAL => "NEUTRAL-SEX",
            SEX_MALE => "MALE",
            SEX_FEMALE => "FEMALE",
            _ => "ILLEGAL-SEX!!",
        }
    );
    let kind = if !w.is_npc(k) {
        "PC"
    } else if !w.is_mob(k) {
        "NPC"
    } else {
        "MOB"
    };
    let rnum = if c.in_room >= 0 { w.room(c.in_room).number } else { -1 };
    p!(" {} - Name : {} [R-Number{}], In room [{}]\n\r", kind, c.player.name, c.nr, rnum);
    if w.is_mob(k) {
        p!("V-Number [{}]\n\r", w.mob_index[c.nr as usize].virtual_nr);
    }
    p!("Short description: {}\n\r", if c.player.short_descr.is_empty() { "None" } else { &c.player.short_descr });
    p!("Title: {}\n\r", if c.player.title.is_empty() { "None" } else { &c.player.title });
    p!("Long description: {}\n\r", if c.player.long_descr.is_empty() { "None" } else { &c.player.long_descr });
    if w.is_npc(k) {
        p!("Monster Class: {}", sprinttype(c.player.class, NPC_CLASS_TYPES));
    } else {
        p!("Class: {}", sprinttype(c.player.class, PC_CLASS_TYPES));
    }
    p!("   Level [{}] Alignment[{}]\n\r", c.player.level, c.specials.alignment);
    p!(
        "Birth : [{}]secs, Logon[{}]secs, Played[{}]secs\n\r",
        c.player.time.birth,
        c.player.time.logon,
        c.player.time.played
    );
    let a = age(&c);
    p!("Age: [{}] Years,  [{}] Months,  [{}] Days,  [{}] Hours\n\r", a.year, a.month, a.day, a.hours);
    p!("Height [{}]cm  Weight [{}]pounds \n\r", c.player.height, c.player.weight);
    let t = &c.tmpabilities;
    p!(
        "Hometown[{}], Speaks[{}/{}/{}], (STL[{}]/per[{}]/NSTL[{}])\n\r",
        c.player.hometown,
        c.player.talks[0] as i32,
        c.player.talks[1] as i32,
        c.player.talks[2] as i32,
        c.specials.spells_to_learn,
        INT_APP[t.intel.clamp(0, 25) as usize],
        WIS_APP[t.wis.clamp(0, 25) as usize]
    );
    p!(
        "Str:[{}/{}]  Int:[{}]  Wis:[{}]  Dex:[{}]  Con:[{}]\n\r",
        t.str, t.str_add, t.intel, t.wis, t.dex, t.con
    );
    p!(
        "Mana p.:[{}/{}+{}]  Hit p.:[{}/{}+{}]  Move p.:[{}/{}+{}]\n\r",
        c.points.mana,
        mana_limit(w, k),
        mana_gain(w, k),
        c.points.hit,
        hit_limit(w, k),
        hit_gain_calc(w, k),
        c.points.mov,
        move_limit(w, k),
        move_gain(w, k)
    );
    p!(
        "AC:[{}/10], Coins: [{}], Exp: [{}], Hitroll: [{}], Damroll: [{}]\n\r",
        c.points.armor,
        c.points.gold,
        c.points.exp,
        c.points.hitroll,
        c.points.damroll
    );
    let fighting = c.specials.fighting.filter(|f| w.ch_exists(*f)).map(|f| w[f].player.name.clone());
    p!(
        "Position: {}, Fighting: {}",
        sprinttype(c.specials.position, POSITION_TYPES),
        fighting.as_deref().unwrap_or("Nobody")
    );
    if let Some(d) = c.desc.and_then(|d| w.desc(d)) {
        p!(", Connected: {}", sprinttype(d.connected, CONNECTED_TYPES));
    }
    p!("\n\r");
    p!("Default position: {}", sprinttype(c.specials.default_pos, POSITION_TYPES));
    if w.is_npc(k) {
        p!(",NPC flags: {}", sprintbit(c.specials.act as i64, ACTION_BITS));
    } else {
        p!(",PC flags: {}", sprintbit(c.specials.act as i64, PLAYER_BITS));
    }
    p!(",Timer [{}] \n\r", c.specials.timer);
    if w.is_mob(k) {
        let sp = w.mob_index[c.nr as usize].func.is_some();
        p!("\n\rMobile Special procedure : {}", if sp { "Exists\n\r" } else { "None\n\r" });
    }
    if w.is_npc(k) {
        p!("NPC Bare Hand Damage {}d{}.\n\r", c.specials.damnodice, c.specials.damsizedice);
    }
    p!("Carried weight: {}   Carried items: {}\n\r", c.specials.carry_weight, c.specials.carry_items);
    let neq = c.equipment.iter().filter(|e| e.is_some()).count();
    p!("Items in inventory: {}, Items in equipment: {}\n\r", c.carrying.len(), neq);
    let s = c.specials.apply_saving_throw;
    p!("Apply saving throws: [{}] [{}] [{}] [{}] [{}]\n\r", s[0], s[1], s[2], s[3], s[4]);
    p!(
        "Thirst: {}, Hunger: {}, Drunk: {}\n\r",
        c.specials.conditions[THIRST],
        c.specials.conditions[FULL],
        c.specials.conditions[DRUNK]
    );
    let master = c.master.filter(|m| w.ch_exists(*m)).map(|m| w[m].player.name.clone());
    p!("Master is '{}'\n\r", master.as_deref().unwrap_or("NOBODY"));
    p!("Followers are:\n\r");
    send_to_char(w, &out, ch);
    for f in c.followers.clone() {
        act(w, "    $N", false, ch, None, Vict::Ch(f), TO_CHAR);
    }
    let mut out = String::new();
    out.push_str("Affected by: ");
    out.push_str(&sprintbit(c.specials.affected_by, AFFECTED_BITS));
    out.push_str("\n\r");
    if !c.affected.is_empty() {
        out.push_str("\n\rAffecting Spells:\n\r--------------\n\r");
        for af in &c.affected {
            let sname = if af.type_ >= 1 && (af.type_ as usize) <= SPELLS.len() { SPELLS[(af.type_ - 1) as usize] } else { "UNKNOWN" };
            out.push_str(&format!("Spell : '{}'\n\r", sname));
            out.push_str(&format!("     Modifies {} by {} points\n\r", sprinttype(af.location, APPLY_TYPES), af.modifier));
            out.push_str(&format!("     Expires in {:3} hours, Bits set ", af.duration));
            out.push_str(&sprintbit(af.bitvector, AFFECTED_BITS));
            out.push_str("\n\r");
        }
    }
    send_to_char(w, &out, ch);
}

pub fn do_shutdow(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    send_to_char(w, "If you want to shut something down - say so!\n\r", ch);
}

pub fn do_shutdown(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        return;
    }
    let (arg, _) = one_argument(argument);
    if arg.is_empty() {
        let buf = format!("Shutdown by {}.", w[ch].player.name);
        send_to_all(w, &buf);
        log(&buf);
        w.shutdown = true;
    } else if str_cmp(&arg, "reboot") == 0 {
        let buf = format!("Reboot by {}.", w[ch].player.name);
        send_to_all(w, &buf);
        log(&buf);
        w.shutdown = true;
        w.reboot = true;
    } else {
        send_to_char(w, "Go shut down someone your own size.\n\r", ch);
    }
}

fn set_snoop_by(w: &mut World, c: CharId, by: Option<CharId>) {
    if let Some(d) = w[c].desc {
        if let Some(d) = w.desc_mut(d) {
            d.snoop.snoop_by = by;
        }
    }
}

pub fn do_snoop(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let Some(did) = w[ch].desc else { return };
    if w.is_npc(ch) {
        send_to_char(w, "Did you ever try this before?", ch);
        return;
    }
    let (arg, _) = one_argument(argument);
    if arg.is_empty() {
        send_to_char(w, "Snoop who ?\n\r", ch);
        return;
    }
    let Some(victim) = get_char_vis(w, ch, &arg) else {
        send_to_char(w, "No such person around.\n\r", ch);
        return;
    };
    let Some(vdid) = w[victim].desc else {
        send_to_char(w, "There's no link.. nothing to snoop.\n\r", ch);
        return;
    };
    let snooping = w.desc(did).and_then(|d| d.snoop.snooping);
    if victim == ch {
        send_to_char(w, "Ok, you just snoop yourself.\n\r", ch);
        if let Some(s) = snooping {
            if w.ch_exists(s) {
                set_snoop_by(w, s, None);
            }
            if let Some(d) = w.desc_mut(did) {
                d.snoop.snooping = None;
            }
        }
        return;
    }
    if w.desc(vdid).and_then(|d| d.snoop.snoop_by).is_some() {
        send_to_char(w, "Busy already. \n\r", ch);
        return;
    }
    if w.level(victim) >= w.level(ch) {
        send_to_char(w, "You failed.\n\r", ch);
        return;
    }
    send_to_char(w, "Ok. \n\r", ch);
    if let Some(s) = snooping {
        if w.ch_exists(s) {
            set_snoop_by(w, s, None);
        }
    }
    if let Some(d) = w.desc_mut(did) {
        d.snoop.snooping = Some(victim);
    }
    if let Some(d) = w.desc_mut(vdid) {
        d.snoop.snoop_by = Some(ch);
    }
}

pub fn do_switch(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        return;
    }
    let (arg, _) = one_argument(argument);
    if arg.is_empty() {
        send_to_char(w, "Switch with who?\n\r", ch);
        return;
    }
    let Some(victim) = get_char(w, &arg) else {
        send_to_char(w, "They aren't here.\n\r", ch);
        return;
    };
    if ch == victim {
        send_to_char(w, "He he he... We are jolly funny today, eh?\n\r", ch);
        return;
    }
    let did = w[ch].desc;
    let snoopy = did.and_then(|d| w.desc(d)).map(|d| d.snoop.snoop_by.is_some() || d.snoop.snooping.is_some());
    let Some(did) = did.filter(|_| snoopy == Some(false)) else {
        send_to_char(w, "Mixing snoop & switch is bad for your health.\n\r", ch);
        return;
    };
    if w[victim].desc.is_some() || !w.is_npc(victim) {
        send_to_char(w, "You can't do that, the body is already in use!\n\r", ch);
        return;
    }
    send_to_char(w, "Ok.\n\r", ch);
    if let Some(d) = w.desc_mut(did) {
        d.character = Some(victim);
        d.original = Some(ch);
    }
    w[victim].desc = Some(did);
    w[ch].desc = None;
}

pub fn do_return(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    let Some(did) = w[ch].desc else { return };
    let orig = w.desc(did).and_then(|d| d.original);
    let Some(orig) = orig else {
        send_to_char(w, "Arglebargle, glop-glyf!?!\n\r", ch);
        return;
    };
    send_to_char(w, "You return to your originaly body.\n\r", ch);
    if let Some(d) = w.desc_mut(did) {
        d.character = Some(orig);
        d.original = None;
    }
    if w.ch_exists(orig) {
        w[orig].desc = Some(did);
    }
    w[ch].desc = None;
}

pub fn do_force(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        return;
    }
    let (name, to_force) = half_chop(argument);
    if name.is_empty() || to_force.is_empty() {
        send_to_char(w, "Who do you wish to force to do what?\n\r", ch);
    } else if str_cmp("all", &name) != 0 {
        match get_char_vis(w, ch, &name) {
            None => send_to_char(w, "No-one by that name here..\n\r", ch),
            Some(v) => {
                if w.level(ch) < w.level(v) && !w.is_npc(v) {
                    send_to_char(w, "Oh no you don't!!\n\r", ch);
                } else {
                    let buf = format!("$n has forced you to '{}'.", to_force);
                    act(w, &buf, false, ch, None, Vict::Ch(v), TO_VICT);
                    send_to_char(w, "Ok.\n\r", ch);
                    command_interpreter(w, v, &to_force);
                }
            }
        }
    } else {
        let targets: Vec<CharId> = w
            .descriptors
            .iter()
            .filter(|d| d.connected == CON_PLYNG)
            .filter_map(|d| d.character)
            .filter(|&c| c != ch)
            .collect();
        for v in targets {
            if !w.ch_exists(v) || !w.ch_exists(ch) {
                continue;
            }
            if w.level(ch) < w.level(v) {
                send_to_char(w, "Oh no you don't!!\n\r", ch);
            } else {
                let buf = format!("$n has forced you to '{}'.", to_force);
                act(w, &buf, false, ch, None, Vict::Ch(v), TO_VICT);
                command_interpreter(w, v, &to_force);
            }
        }
        if w.ch_exists(ch) {
            send_to_char(w, "Ok.\n\r", ch);
        }
    }
}

pub fn do_load(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        return;
    }
    let (typ, num) = argument_interpreter(argument);
    if typ.is_empty() || num.is_empty() || !num.as_bytes()[0].is_ascii_digit() {
        send_to_char(w, "Syntax:\n\rload <'char' | 'obj'> <number>.\n\r", ch);
        return;
    }
    let number = atoi(&num);
    if number < 0 {
        send_to_char(w, "A NEGATIVE number??\n\r", ch);
        return;
    }
    let room = w[ch].in_room;
    if is_abbrev(&typ, "char") {
        let r = real_mobile(w, number);
        if r < 0 {
            send_to_char(w, "There is no monster with that number.\n\r", ch);
            return;
        }
        let Some(mob) = read_mobile(w, r, REAL) else { return };
        char_to_room(w, mob, room);
        act(w, "$n makes a quaint, magical gesture with one hand.", true, ch, None, Vict::None, TO_ROOM);
        act(w, "$n has created $N!", false, ch, None, Vict::Ch(mob), TO_ROOM);
        send_to_char(w, "Done.\n\r", ch);
        log(&format!("{} loads {} at {}.", w[ch].player.name, w[mob].player.short_descr, w.room(room).name));
    } else if is_abbrev(&typ, "obj") {
        let r = real_object(w, number);
        if r < 0 {
            send_to_char(w, "There is no object with that number.\n\r", ch);
            return;
        }
        let Some(obj) = read_object(w, r, REAL) else { return };
        obj_to_room(w, obj, room);
        act(w, "$n makes a strange magical gesture.", true, ch, None, Vict::None, TO_ROOM);
        act(w, "$n has created $p!", false, ch, Some(obj), Vict::None, TO_ROOM);
        send_to_char(w, "Ok.\n\r", ch);
        log(&format!("{} loads {} at {}.", w[ch].player.name, w[obj].short_description, w.room(room).name));
    } else {
        send_to_char(w, "That'll have to be either 'char' or 'obj'.\n\r", ch);
    }
}

pub fn do_purge(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        return;
    }
    let (name, _) = one_argument(argument);
    let room = w[ch].in_room;
    if !name.is_empty() {
        let mut vict = get_char_room_vis(w, ch, &name);
        if vict.is_none() && w.level(ch) >= 23 {
            vict = get_char(w, &name);
        }
        if let Some(v) = vict {
            if !w.is_npc(v) && w.level(ch) <= w.level(v) {
                send_to_char(w, "Fuuuuuuuuu!\n\r", ch);
                return;
            }
            act(w, "$n disintegrates $N.", false, ch, None, Vict::Ch(v), TO_NOTVICT);
            if !w.is_npc(v) {
                if let Some(d) = w[v].desc {
                    log(&format!("{} purges {} at {}.", w[ch].player.name, w[v].player.name, w.room(room).name));
                    crate::net::close_socket(w, d);
                    if w.ch_exists(v) {
                        w[v].desc = None;
                    }
                }
            }
            if w.ch_exists(v) {
                extract_char(w, v);
            }
        } else if let Some(o) = get_obj_in_list_vis(w, ch, &name, &w.room(room).contents) {
            act(w, "$n destroys $p.", false, ch, Some(o), Vict::None, TO_ROOM);
            extract_obj(w, o);
        } else {
            send_to_char(w, "I don't know anyone or anything by that name.\n\r", ch);
            return;
        }
        send_to_char(w, "Ok.\n\r", ch);
    } else {
        act(w, "$n gestures... You are surrounded by scorching flames!", false, ch, None, Vict::None, TO_ROOM);
        send_to_room(w, "The world seems a little cleaner.\n\r", room);
        for v in w.room(room).people.clone() {
            if w.ch_exists(v) && w.is_npc(v) {
                extract_char(w, v);
            }
        }
        for o in w.room(room).contents.clone() {
            if w.obj_exists(o) {
                extract_obj(w, o);
            }
        }
    }
}

pub fn roll_abilities(w: &mut World, ch: CharId) {
    let mut table = [0i32; 5];
    for _ in 0..5 {
        let rools = [number(1, 6), number(1, 6), number(1, 6), number(1, 6)];
        let mut temp = rools.iter().sum::<i32>() - rools.iter().min().unwrap();
        for t in table.iter_mut() {
            if *t < temp {
                std::mem::swap(t, &mut temp);
            }
        }
    }
    let class = w[ch].player.class;
    let a = &mut w[ch].abilities;
    a.str_add = 0;
    match class {
        CLASS_MAGIC_USER => {
            a.intel = table[0];
            a.wis = table[1];
            a.dex = table[2];
            a.str = table[3];
            a.con = table[4];
        }
        CLASS_CLERIC => {
            a.wis = table[0];
            a.intel = table[1];
            a.str = table[2];
            a.dex = table[3];
            a.con = table[4];
        }
        CLASS_THIEF => {
            a.dex = table[0];
            a.str = table[1];
            a.con = table[2];
            a.intel = table[3];
            a.wis = table[4];
        }
        CLASS_WARRIOR => {
            a.str = table[0];
            a.dex = table[1];
            a.con = table[2];
            a.wis = table[3];
            a.intel = table[4];
            if a.str == 18 {
                a.str_add = number(0, 100);
            }
        }
        _ => {}
    }
    w[ch].tmpabilities = w[ch].abilities;
}

pub fn do_start(w: &mut World, ch: CharId) {
    send_to_char(
        w,
        "Welcome. This is now you character in DikuMud,\n\rYou can now earn XP, and lots more...\n\r",
        ch,
    );
    w[ch].player.level = 1;
    w[ch].points.exp = 1;
    set_title(w, ch);
    roll_abilities(w, ch);
    w[ch].points.max_hit = 10;
    if w[ch].player.class == CLASS_THIEF {
        let s = &mut w[ch].skills;
        s[SKILL_SNEAK as usize].learned = 10;
        s[SKILL_HIDE as usize].learned = 5;
        s[SKILL_STEAL as usize].learned = 15;
        s[SKILL_BACKSTAB as usize].learned = 10;
        s[SKILL_PICK_LOCK as usize].learned = 10;
    }
    advance_level(w, ch);
    let h = hit_limit(w, ch);
    let m = mana_limit(w, ch);
    let v = move_limit(w, ch);
    let c = &mut w[ch];
    c.points.hit = h;
    c.points.mana = m;
    c.points.mov = v;
    c.specials.conditions[THIRST] = 24;
    c.specials.conditions[FULL] = 24;
    c.specials.conditions[DRUNK] = 0;
    c.player.time.played = 0;
    c.player.time.logon = now();
}

pub fn do_advance(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        return;
    }
    let (name, level) = argument_interpreter(argument);
    if name.is_empty() {
        send_to_char(w, "Advance who?\n\r", ch);
        return;
    }
    let Some(victim) = get_char_room_vis(w, ch, &name) else {
        send_to_char(w, "That player is not here.\n\r", ch);
        return;
    };
    if w.is_npc(victim) {
        send_to_char(w, "NO! Not on NPC's.\n\r", ch);
        return;
    }
    let vl = w.level(victim);
    let adv;
    let mut newlevel = 1;
    if vl == 0 {
        adv = 1;
    } else if level.is_empty() {
        send_to_char(w, "You must supply a level number.\n\r", ch);
        return;
    } else {
        if !level.as_bytes()[0].is_ascii_digit() {
            send_to_char(w, "Second argument must be a positive integer.\n\r", ch);
            return;
        }
        newlevel = atoi(&level);
        if newlevel <= vl {
            send_to_char(w, "Can't dimish a players status (yet).\n\r", ch);
            return;
        }
        adv = newlevel - vl;
    }
    if adv + vl > 1 && w.level(ch) < 24 {
        send_to_char(w, "Thou art not godly enough.\n\r", ch);
        return;
    }
    if adv + vl > 24 {
        send_to_char(w, "24 is the highest possible level.\n\r", ch);
        return;
    }
    if adv + vl < 21 && adv + vl != 1 {
        send_to_char(w, "21 is the lowest possible level.\n\r", ch);
        return;
    }
    send_to_char(w, "You feel generous.\n\r", ch);
    act(
        w,
        "$n makes some strange gestures.\n\rA strange feeling comes uppon you,\n\rLike a giant hand, light comes down from\n\rabove, grabbing your body, that begins\n\rto pulse with coloured lights from inside.\n\rYour head seems to be filled with deamons\n\rfrom another plane as your body dissolves\n\rto the elements of time and space itself.\n\rSuddenly a silent explosion of light snaps\n\ryou back to reality. You feel slightly\n\rdifferent.",
        false,
        ch,
        None,
        Vict::Ch(victim),
        TO_VICT,
    );
    log(&format!("{} advances {} to level {}.", w[ch].player.name, w[victim].player.name, newlevel));
    if vl == 0 {
        do_start(w, victim);
    } else if vl < 24 {
        let cls = (w[victim].player.class - 1).clamp(0, 3) as usize;
        let target = TITLES[cls][(vl + adv).clamp(0, 24) as usize].exp;
        let e = w[victim].points.exp;
        gain_exp_regardless(w, victim, target - e);
        send_to_char(w, "WARNING! Was not level 0.\n\r", ch);
    } else {
        send_to_char(w, "Some idiot just tried to advance your level.\n\r", victim);
        send_to_char(w, "IMPOSSIBLE! IDIOTIC!\n\r", ch);
    }
}

pub fn do_reroll(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        return;
    }
    let (buf, _) = one_argument(argument);
    if buf.is_empty() {
        send_to_char(w, "Who do you wich to reroll?\n\r", ch);
        return;
    }
    match get_char(w, &buf) {
        None => send_to_char(w, "No-one by that name in the world.\n\r", ch),
        Some(v) => {
            send_to_char(w, "Rerolled...\n\r", ch);
            roll_abilities(w, v);
            log(&format!("{} rerolled {}.", w[ch].player.name, w[v].player.name));
        }
    }
}

pub fn do_restore(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        return;
    }
    let (buf, _) = one_argument(argument);
    if buf.is_empty() {
        send_to_char(w, "Who do you wich to restore?\n\r", ch);
        return;
    }
    let Some(v) = get_char(w, &buf) else {
        send_to_char(w, "No-one by that name in the world.\n\r", ch);
        return;
    };
    let (m, h, mv) = (mana_limit(w, v), hit_limit(w, v), move_limit(w, v));
    w[v].points.mana = m;
    w[v].points.hit = h;
    w[v].points.mov = mv;
    log(&format!("{} restored {}.", w[ch].player.name, w[v].player.name));
    if w.level(v) >= 21 {
        for s in w[v].skills.iter_mut() {
            s.learned = 100;
            s.recognise = true;
        }
        if w.level(v) >= 23 {
            w[v].abilities = Abilities { str: 25, str_add: 100, intel: 25, wis: 25, dex: 25, con: 25 };
        }
        w[v].tmpabilities = w[v].abilities;
    }
    update_pos(w, v);
    send_to_char(w, "Done.\n\r", ch);
    act(w, "You have been fully healed by $N!", false, v, None, Vict::Ch(ch), TO_CHAR);
}

pub fn do_noshout(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        return;
    }
    let (buf, _) = one_argument(argument);
    if buf.is_empty() {
        if w[ch].specials.act & PLR_NOSHOUT != 0 {
            send_to_char(w, "You can now hear shouts again.\n\r", ch);
            w[ch].specials.act &= !PLR_NOSHOUT;
        } else {
            send_to_char(w, "From now on, you won't hear shouts.\n\r", ch);
            w[ch].specials.act |= PLR_NOSHOUT;
        }
        return;
    }
    let (bits, vict, _) = generic_find(w, argument, FIND_CHAR_WORLD, ch);
    let Some(vict) = vict.filter(|_| bits != 0) else {
        send_to_char(w, "Couldn't find any such creature.\n\r", ch);
        return;
    };
    if w.is_npc(vict) {
        send_to_char(w, "Can't do that to a beast.\n\r", ch);
    } else if w.level(vict) > w.level(ch) {
        act(w, "$E might object to that.. better not.", false, ch, None, Vict::Ch(vict), TO_CHAR);
    } else if w[vict].specials.act & PLR_NOSHOUT != 0 {
        send_to_char(w, "You can shout again.\n\r", vict);
        send_to_char(w, "NOSHOUT removed.\n\r", ch);
        w[vict].specials.act &= !PLR_NOSHOUT;
    } else {
        send_to_char(w, "The gods take away your ability to shout!\n\r", vict);
        send_to_char(w, "NOSHOUT set.\n\r", ch);
        w[vict].specials.act |= PLR_NOSHOUT;
    }
}
