//! Moving characters and objects around (handler.c).

#![allow(dead_code)]

use crate::comm::{act, send_to_char, Vict, TO_CHAR, TO_ROOM};
use crate::game::World;
use crate::structs::*;
use crate::utils::*;

pub const FIND_CHAR_ROOM: i32 = 1;
pub const FIND_CHAR_WORLD: i32 = 2;
pub const FIND_OBJ_INV: i32 = 4;
pub const FIND_OBJ_ROOM: i32 = 8;
pub const FIND_OBJ_WORLD: i32 = 16;
pub const FIND_OBJ_EQUIP: i32 = 32;

/// First name in a name list (only the leading alphabetic run).
pub fn fname(namelist: &str) -> String {
    namelist.chars().take_while(|c| c.is_ascii_alphabetic()).take(29).collect()
}

/// Is `s` one of the names in `namelist`?  (prefix-free, case-insensitive)
pub fn isname(s: &str, namelist: &str) -> bool {
    let st = s.as_bytes();
    let nl = namelist.as_bytes();
    let mut cur = 0usize;
    loop {
        let mut i = 0usize;
        loop {
            let cs = st.get(i).copied().unwrap_or(0);
            let cn = nl.get(cur + i).copied().unwrap_or(0);
            if cs == 0 && !cn.is_ascii_alphabetic() {
                return true;
            }
            if cn == 0 {
                return false;
            }
            if cs == 0 || cn == b' ' {
                break;
            }
            if cs.to_ascii_lowercase() != cn.to_ascii_lowercase() {
                break;
            }
            i += 1;
        }
        cur += i;
        while cur < nl.len() && nl[cur].is_ascii_alphabetic() {
            cur += 1;
        }
        if cur >= nl.len() {
            return false;
        }
        cur += 1;
    }
}

pub fn affect_modify(w: &mut World, ch: CharId, loc: i32, modifier: i32, bitv: i64, add: bool) {
    let c = &mut w[ch];
    let mut m = modifier;
    if add {
        c.specials.affected_by |= bitv;
    } else {
        c.specials.affected_by &= !bitv;
        m = -m;
    }
    match loc {
        APPLY_NONE => {}
        APPLY_STR => c.tmpabilities.str += m,
        APPLY_DEX => c.tmpabilities.dex += m,
        APPLY_INT => c.tmpabilities.intel += m,
        APPLY_WIS => c.tmpabilities.wis += m,
        APPLY_CON => c.tmpabilities.con += m,
        APPLY_SEX | APPLY_CLASS | APPLY_LEVEL => {}
        APPLY_AGE => c.player.time.birth -= SECS_PER_MUD_YEAR * m as i64,
        APPLY_CHAR_WEIGHT => c.player.weight += m,
        APPLY_CHAR_HEIGHT => c.player.height += m,
        APPLY_MANA => {}
        APPLY_HIT => c.points.max_hit += m,
        APPLY_MOVE | APPLY_GOLD | APPLY_EXP => {}
        APPLY_AC => c.points.armor += m,
        APPLY_HITROLL => c.points.hitroll += m,
        APPLY_DAMROLL => c.points.damroll += m,
        APPLY_SAVING_PARA => c.specials.apply_saving_throw[0] += m,
        APPLY_SAVING_ROD => c.specials.apply_saving_throw[1] += m,
        APPLY_SAVING_PETRI => c.specials.apply_saving_throw[2] += m,
        APPLY_SAVING_BREATH => c.specials.apply_saving_throw[3] += m,
        APPLY_SAVING_SPELL => c.specials.apply_saving_throw[4] += m,
        _ => log("Unknown apply adjust attempt (handler.c, affect_modify)."),
    }
}

fn eq_affects(w: &World, ch: CharId) -> Vec<(i32, i32, i64)> {
    let mut v = Vec::new();
    for i in 0..MAX_WEAR {
        if let Some(o) = w[ch].equipment[i] {
            for j in 0..MAX_OBJ_AFFECT {
                let a = w[o].affected[j];
                v.push((a.location, a.modifier, w[o].obj_flags.bitvector));
            }
        }
    }
    v
}

/// Subtract all affects, restore original abilities, re-apply everything.
pub fn affect_total(w: &mut World, ch: CharId) {
    let eq = eq_affects(w, ch);
    let afs: Vec<Affect> = w[ch].affected.clone();
    for &(l, m, b) in &eq {
        affect_modify(w, ch, l, m, b, false);
    }
    for af in &afs {
        affect_modify(w, ch, af.location, af.modifier, af.bitvector, false);
    }
    let abil = w[ch].abilities;
    w[ch].tmpabilities = abil;
    for &(l, m, b) in &eq {
        affect_modify(w, ch, l, m, b, true);
    }
    for af in &afs {
        affect_modify(w, ch, af.location, af.modifier, af.bitvector, true);
    }
    let npc = w.is_npc(ch);
    let i = if npc { 25 } else { 18 };
    let t = &mut w[ch].tmpabilities;
    t.dex = t.dex.min(i).max(0);
    t.intel = t.intel.min(i).max(0);
    t.wis = t.wis.min(i).max(0);
    t.con = t.con.min(i).max(0);
    t.str = t.str.max(0);
    if npc {
        t.str = t.str.min(i);
    } else if t.str > 18 {
        let x = t.str_add + (t.str - 18) * 10;
        t.str_add = x.min(100);
        t.str = 18;
    }
}

pub fn affect_to_char(w: &mut World, ch: CharId, af: &Affect) {
    w[ch].affected.insert(0, *af);
    affect_modify(w, ch, af.location, af.modifier, af.bitvector, true);
    affect_total(w, ch);
}

/// Remove the affect at position `idx` of `ch->affected`.
pub fn affect_remove(w: &mut World, ch: CharId, idx: usize) {
    if idx >= w[ch].affected.len() {
        return;
    }
    let af = w[ch].affected[idx];
    affect_modify(w, ch, af.location, af.modifier, af.bitvector, false);
    w[ch].affected.remove(idx);
    affect_total(w, ch);
}

pub fn affect_from_char(w: &mut World, ch: CharId, skill: i32) {
    let mut i = 0;
    while i < w[ch].affected.len() {
        if w[ch].affected[i].type_ == skill {
            affect_remove(w, ch, i);
        } else {
            i += 1;
        }
    }
}

pub fn affected_by_spell(w: &World, ch: CharId, skill: i32) -> bool {
    w[ch].affected.iter().any(|a| a.type_ == skill)
}

pub fn affect_join(w: &mut World, ch: CharId, af: &mut Affect, avg_dur: bool, avg_mod: bool) {
    if let Some(i) = w[ch].affected.iter().position(|h| h.type_ == af.type_) {
        let h = w[ch].affected[i];
        af.duration += h.duration;
        if avg_dur {
            af.duration /= 2;
        }
        af.modifier += h.modifier;
        if avg_mod {
            af.modifier /= 2;
        }
        affect_remove(w, ch, i);
        affect_to_char(w, ch, af);
    } else {
        affect_to_char(w, ch, af);
    }
}

fn light_on(w: &World, ch: CharId) -> bool {
    if let Some(o) = w[ch].equipment[WEAR_LIGHT] {
        w[o].obj_flags.type_flag == ITEM_LIGHT && w[o].obj_flags.value[2] != 0
    } else {
        false
    }
}

pub fn char_from_room(w: &mut World, ch: CharId) {
    let r = w[ch].in_room;
    if r == NOWHERE {
        log("NOWHERE extracting char from room (handler.c, char_from_room)");
        return;
    }
    if light_on(w, ch) {
        w.room_mut(r).light -= 1;
    }
    w.room_mut(r).people.retain(|&p| p != ch);
    w[ch].in_room = NOWHERE;
}

pub fn char_to_room(w: &mut World, ch: CharId, room: i32) {
    if room < 0 || room as usize >= w.world.len() {
        log(&format!("char_to_room: illegal room {}", room));
        return;
    }
    w.room_mut(room).people.insert(0, ch);
    w[ch].in_room = room;
    if light_on(w, ch) {
        w.room_mut(room).light += 1;
    }
}

pub fn obj_to_char(w: &mut World, obj: ObjId, ch: CharId) {
    w[ch].carrying.insert(0, obj);
    let wt = w[obj].obj_flags.weight;
    w[obj].carried_by = Some(ch);
    w[obj].in_room = NOWHERE;
    w[ch].specials.carry_weight += wt;
    w[ch].specials.carry_items += 1;
}

pub fn obj_from_char(w: &mut World, obj: ObjId) {
    let Some(ch) = w[obj].carried_by else { return };
    w[ch].carrying.retain(|&o| o != obj);
    let wt = w[obj].obj_flags.weight;
    w[ch].specials.carry_weight -= wt;
    w[ch].specials.carry_items -= 1;
    w[obj].carried_by = None;
}

pub fn apply_ac(w: &World, ch: CharId, eq_pos: usize) -> i32 {
    let Some(o) = w[ch].equipment[eq_pos] else { return 0 };
    if w[o].obj_flags.type_flag != ITEM_ARMOR {
        return 0;
    }
    let v = w[o].obj_flags.value[0];
    match eq_pos {
        WEAR_BODY => 3 * v,
        WEAR_HEAD | WEAR_LEGS => 2 * v,
        WEAR_FEET | WEAR_HANDS | WEAR_ARMS | WEAR_SHIELD => v,
        _ => 0,
    }
}

pub fn equip_char(w: &mut World, ch: CharId, obj: ObjId, pos: usize) {
    if pos >= MAX_WEAR || w[ch].equipment[pos].is_some() {
        log("EQUIP: position illegal or already occupied.");
        return;
    }
    if w[obj].carried_by.is_some() {
        log("EQUIP: Obj is carried_by when equip.");
        return;
    }
    if w[obj].in_room != NOWHERE {
        log("EQUIP: Obj is in_room when equip.");
        return;
    }
    if (w.is_obj_stat(obj, ITEM_ANTI_EVIL) && w.is_evil(ch))
        || (w.is_obj_stat(obj, ITEM_ANTI_GOOD) && w.is_good(ch))
        || (w.is_obj_stat(obj, ITEM_ANTI_NEUTRAL) && w.is_neutral(ch))
    {
        if w[ch].in_room != NOWHERE {
            act(w, "You are zapped by $p and instantly drop it.", false, ch, Some(obj), Vict::None, TO_CHAR);
            act(w, "$n is zapped by $p and instantly drop it.", false, ch, Some(obj), Vict::None, TO_ROOM);
            let r = w[ch].in_room;
            obj_to_room(w, obj, r);
            return;
        } else {
            log("ch->in_room = NOWHERE when equipping char.");
        }
    }
    w[ch].equipment[pos] = Some(obj);
    if w[obj].obj_flags.type_flag == ITEM_ARMOR {
        let a = apply_ac(w, ch, pos);
        w[ch].points.armor -= a;
    }
    let affs = w[obj].affected;
    let bv = w[obj].obj_flags.bitvector;
    for a in affs.iter() {
        affect_modify(w, ch, a.location, a.modifier, bv, true);
    }
    affect_total(w, ch);
}

pub fn unequip_char(w: &mut World, ch: CharId, pos: usize) -> Option<ObjId> {
    let obj = w[ch].equipment[pos]?;
    if w[obj].obj_flags.type_flag == ITEM_ARMOR {
        let a = apply_ac(w, ch, pos);
        w[ch].points.armor += a;
    }
    w[ch].equipment[pos] = None;
    let affs = w[obj].affected;
    let bv = w[obj].obj_flags.bitvector;
    for a in affs.iter() {
        affect_modify(w, ch, a.location, a.modifier, bv, false);
    }
    affect_total(w, ch);
    Some(obj)
}

/// Split "3.sword" into (3, "sword").  Returns number 0 for a bad prefix.
pub fn get_number(name: &str) -> (i32, String) {
    if let Some(p) = name.find('.') {
        let (num, rest) = (&name[..p], &name[p + 1..]);
        if !num.chars().all(|c| c.is_ascii_digit()) {
            return (0, rest.to_string());
        }
        (atoi(num), rest.to_string())
    } else {
        (1, name.to_string())
    }
}

pub fn get_obj_in_list(w: &World, name: &str, list: &[ObjId]) -> Option<ObjId> {
    let (number, tmp) = get_number(name);
    if number == 0 {
        return None;
    }
    let mut j = 1;
    for &i in list {
        if isname(&tmp, &w[i].name) {
            if j == number {
                return Some(i);
            }
            j += 1;
        }
    }
    None
}

pub fn get_obj_in_list_num(w: &World, num: i32, list: &[ObjId]) -> Option<ObjId> {
    list.iter().copied().find(|&i| w[i].item_number == num)
}

pub fn get_obj(w: &World, name: &str) -> Option<ObjId> {
    let (number, tmp) = get_number(name);
    if number == 0 {
        return None;
    }
    let mut j = 1;
    for &i in &w.object_list {
        if isname(&tmp, &w[i].name) {
            if j == number {
                return Some(i);
            }
            j += 1;
        }
    }
    None
}

pub fn get_obj_num(w: &World, nr: i32) -> Option<ObjId> {
    w.object_list.iter().copied().find(|&i| w[i].item_number == nr)
}

pub fn get_char_room(w: &World, name: &str, room: i32) -> Option<CharId> {
    let (number, tmp) = get_number(name);
    if number == 0 {
        return None;
    }
    let mut j = 1;
    for &i in &w.room(room).people {
        if isname(&tmp, &w[i].player.name) {
            if j == number {
                return Some(i);
            }
            j += 1;
        }
    }
    None
}

pub fn get_char(w: &World, name: &str) -> Option<CharId> {
    let (number, tmp) = get_number(name);
    if number == 0 {
        return None;
    }
    let mut j = 1;
    for &i in &w.character_list {
        if isname(&tmp, &w[i].player.name) {
            if j == number {
                return Some(i);
            }
            j += 1;
        }
    }
    None
}

pub fn get_char_num(w: &World, nr: i32) -> Option<CharId> {
    w.character_list.iter().copied().find(|&i| w[i].nr == nr)
}

pub fn obj_to_room(w: &mut World, obj: ObjId, room: i32) {
    w.room_mut(room).contents.insert(0, obj);
    w[obj].in_room = room;
    w[obj].carried_by = None;
}

pub fn obj_from_room(w: &mut World, obj: ObjId) {
    let r = w[obj].in_room;
    if r >= 0 {
        w.room_mut(r).contents.retain(|&o| o != obj);
    }
    w[obj].in_room = NOWHERE;
}

pub fn obj_to_obj(w: &mut World, obj: ObjId, obj_to: ObjId) {
    w[obj_to].contains.insert(0, obj);
    w[obj].in_obj = Some(obj_to);
    let wt = w[obj].obj_flags.weight;
    let mut t = Some(obj_to);
    while let Some(o) = t {
        w[o].obj_flags.weight += wt;
        t = w[o].in_obj;
    }
}

pub fn obj_from_obj(w: &mut World, obj: ObjId) {
    let Some(from) = w[obj].in_obj else {
        log("Trying to object from object when in no object.");
        return;
    };
    w[from].contains.retain(|&o| o != obj);
    let wt = w[obj].obj_flags.weight;
    let mut tmp = from;
    loop {
        w[tmp].obj_flags.weight -= wt;
        match w[tmp].in_obj {
            Some(n) => tmp = n,
            None => break,
        }
    }
    if let Some(c) = w[tmp].carried_by {
        w[c].specials.carry_weight -= wt;
    }
    w[obj].in_obj = None;
}

/// Set carried_by of a list of objects (and everything inside them).
pub fn object_list_new_owner(w: &mut World, list: &[ObjId], ch: Option<CharId>) {
    for &o in list {
        let inner = w[o].contains.clone();
        object_list_new_owner(w, &inner, ch);
        w[o].carried_by = ch;
    }
}

pub fn extract_obj(w: &mut World, obj: ObjId) {
    if w[obj].in_room != NOWHERE {
        obj_from_room(w, obj);
    } else if w[obj].carried_by.is_some() {
        obj_from_char(w, obj);
    } else if let Some(t) = w[obj].in_obj {
        if w.obj_exists(t) {
            w[t].contains.retain(|&o| o != obj);
        }
    }
    while let Some(&c) = w[obj].contains.first() {
        extract_obj(w, c);
        if w.obj_exists(obj) {
            w[obj].contains.retain(|&o| o != c);
        }
    }
    w.object_list.retain(|&o| o != obj);
    let nr = w[obj].item_number;
    if nr >= 0 {
        w.obj_index[nr as usize].number -= 1;
    }
    // Make sure no character still has it equipped.
    w.objs.remove(obj);
}

pub fn update_object(w: &mut World, list: &[ObjId], use_: i32) {
    for &o in list {
        if w[o].obj_flags.timer > 0 {
            w[o].obj_flags.timer -= use_;
        }
        let inner = w[o].contains.clone();
        update_object(w, &inner, use_);
    }
}

pub fn update_char_objects(w: &mut World, ch: CharId) {
    if let Some(o) = w[ch].equipment[WEAR_LIGHT] {
        if w[o].obj_flags.type_flag == ITEM_LIGHT && w[o].obj_flags.value[2] > 0 {
            w[o].obj_flags.value[2] -= 1;
        }
    }
    for i in 0..MAX_WEAR {
        if let Some(o) = w[ch].equipment[i] {
            update_object(w, &[o], 2);
        }
    }
    let c = w[ch].carrying.clone();
    update_object(w, &c, 1);
}

/// Extract a character completely from the world, leaving the stuff behind.
pub fn extract_char(w: &mut World, ch: CharId) {
    if !w.is_npc(ch) && w[ch].desc.is_none() {
        let ds: Vec<(DescId, Option<CharId>, Option<CharId>)> =
            w.descriptors.iter().map(|d| (d.id, d.original, d.character)).collect();
        for (_, orig, cur) in ds {
            if orig == Some(ch) {
                if let Some(cur) = cur {
                    crate::act_wizard::do_return(w, cur, "", 0);
                }
            }
        }
    }

    if w[ch].in_room == NOWHERE {
        log("NOWHERE, extracting char.");
        return;
    }

    if !w[ch].followers.is_empty() || w[ch].master.is_some() {
        crate::spell_parser::die_follower(w, ch);
    }

    if let Some(did) = w[ch].desc {
        let (snooping, snoop_by) = match w.desc(did) {
            Some(d) => (d.snoop.snooping, d.snoop.snoop_by),
            None => (None, None),
        };
        if let Some(s) = snooping {
            if let Some(sd) = w[s].desc {
                if let Some(sd) = w.desc_mut(sd) {
                    sd.snoop.snoop_by = None;
                }
            }
        }
        if let Some(s) = snoop_by {
            send_to_char(w, "Your victim is no longer among us.\n\r", s);
            if let Some(sd) = w[s].desc {
                if let Some(sd) = w.desc_mut(sd) {
                    sd.snoop.snooping = None;
                }
            }
        }
        if let Some(d) = w.desc_mut(did) {
            d.snoop.snooping = None;
            d.snoop.snoop_by = None;
        }
    }

    let room = w[ch].in_room;
    let carrying = std::mem::take(&mut w[ch].carrying);
    for &o in &carrying {
        w[o].carried_by = None;
        w[o].in_room = room;
    }
    w.room_mut(room).contents.extend(carrying);
    w[ch].specials.carry_weight = 0;
    w[ch].specials.carry_items = 0;

    if w[ch].specials.fighting.is_some() {
        crate::fight::stop_fighting(w, ch);
    }
    let combat: Vec<CharId> = w.combat_list.clone();
    for k in combat {
        if w.ch_exists(k) && w[k].specials.fighting == Some(ch) {
            crate::fight::stop_fighting(w, k);
        }
    }

    char_from_room(w, ch);

    for l in 0..MAX_WEAR {
        if w[ch].equipment[l].is_some() {
            if let Some(o) = unequip_char(w, ch, l) {
                obj_to_room(w, o, room);
            }
        }
    }

    w.character_list.retain(|&c| c != ch);
    w[ch].points.armor = 100;

    if let Some(did) = w[ch].desc {
        let orig = w.desc(did).and_then(|d| d.original);
        if orig.is_some() {
            crate::act_wizard::do_return(w, ch, "", 0);
        }
        crate::save::save_char(w, ch, NOWHERE);
    }

    if w.is_npc(ch) {
        let nr = w[ch].nr;
        if nr > -1 {
            w.mob_index[nr as usize].number -= 1;
        }
        w.chars.remove(ch);
        return;
    }

    match w[ch].desc {
        Some(did) => {
            if let Some(d) = w.desc_mut(did) {
                d.connected = CON_SLCT;
                d.output.push(crate::db::MENU.to_string());
            }
        }
        None => {
            // Link-dead player: nobody owns the struct any more.
            w.chars.remove(ch);
        }
    }
}

/* ---------- high-level versions that respect visibility ---------- */

pub fn get_char_room_vis(w: &World, ch: CharId, name: &str) -> Option<CharId> {
    let (number, tmp) = get_number(name);
    if number == 0 {
        return None;
    }
    let mut j = 1;
    for &i in &w.room(w[ch].in_room).people {
        if isname(&tmp, &w[i].player.name) && w.can_see(ch, i) {
            if j == number {
                return Some(i);
            }
            j += 1;
        }
    }
    None
}

pub fn get_char_vis(w: &World, ch: CharId, name: &str) -> Option<CharId> {
    if let Some(i) = get_char_room_vis(w, ch, name) {
        return Some(i);
    }
    let (number, tmp) = get_number(name);
    if number == 0 {
        return None;
    }
    let mut j = 1;
    for &i in &w.character_list {
        if isname(&tmp, &w[i].player.name) && w.can_see(ch, i) {
            if j == number {
                return Some(i);
            }
            j += 1;
        }
    }
    None
}

pub fn get_obj_in_list_vis(w: &World, ch: CharId, name: &str, list: &[ObjId]) -> Option<ObjId> {
    let (number, tmp) = get_number(name);
    if number == 0 {
        return None;
    }
    let mut j = 1;
    for &i in list {
        if isname(&tmp, &w[i].name) && w.can_see_obj(ch, i) {
            if j == number {
                return Some(i);
            }
            j += 1;
        }
    }
    None
}

pub fn get_obj_vis(w: &World, ch: CharId, name: &str) -> Option<ObjId> {
    if let Some(i) = get_obj_in_list_vis(w, ch, name, &w[ch].carrying) {
        return Some(i);
    }
    if let Some(i) = get_obj_in_list_vis(w, ch, name, &w.room(w[ch].in_room).contents) {
        return Some(i);
    }
    let (number, tmp) = get_number(name);
    if number == 0 {
        return None;
    }
    let mut j = 1;
    for &i in &w.object_list {
        if isname(&tmp, &w[i].name) && w.can_see_obj(ch, i) {
            if j == number {
                return Some(i);
            }
            j += 1;
        }
    }
    None
}

pub fn create_money(w: &mut World, amount: i32) -> ObjId {
    let amount = amount.max(1);
    let mut obj = Obj { item_number: -1, in_room: NOWHERE, ..Default::default() };
    let mut ed = ExtraDescr::default();
    if amount == 1 {
        obj.name = "coin gold".into();
        obj.short_description = "a gold coin".into();
        obj.description = "One miserable gold coin.".into();
        ed.keyword = "coin gold".into();
        ed.description = "One miserable gold coin.".into();
    } else {
        obj.name = "coins gold".into();
        obj.short_description = "gold coins".into();
        obj.description = "A pile of gold coins.".into();
        ed.keyword = "coins gold".into();
        ed.description = if amount < 10 {
            format!("There is {} coins.", amount)
        } else if amount < 100 {
            format!("There is about {} coins", 10 * (amount / 10))
        } else if amount < 1000 {
            format!("It looks like something round {} coins", 100 * (amount / 100))
        } else if amount < 100000 {
            format!("You guess there is {} coins", 1000 * ((amount / 1000) + number(0, amount / 1000)))
        } else {
            "There is A LOT of coins".into()
        };
    }
    obj.ex_description.push(ed);
    obj.obj_flags.type_flag = ITEM_MONEY;
    obj.obj_flags.wear_flags = ITEM_TAKE;
    obj.obj_flags.value[0] = amount;
    obj.obj_flags.cost = amount;
    let id = w.objs.insert(obj);
    w.object_list.insert(0, id);
    id
}

/// Generic find.  Returns (bit found, character, object).
pub fn generic_find(w: &World, arg: &str, bitvector: i32, ch: CharId) -> (i32, Option<CharId>, Option<ObjId>) {
    const IGNORE: [&str; 4] = ["the", "in", "on", "at"];
    let mut name = String::new();
    let mut rest = arg;
    loop {
        rest = rest.trim_start_matches(' ');
        if rest.is_empty() {
            break;
        }
        let end = rest.find(' ').unwrap_or(rest.len());
        name = rest[..end].to_string();
        rest = &rest[end..];
        if IGNORE.contains(&name.to_ascii_lowercase().as_str()) {
            // C quirk: an ignore word stops the scan (`found`), leaving it as the name
            name = name.to_ascii_lowercase();
            break;
        }
    }
    if name.is_empty() {
        return (0, None, None);
    }
    if bitvector & FIND_CHAR_ROOM != 0 {
        if let Some(c) = get_char_room_vis(w, ch, &name) {
            return (FIND_CHAR_ROOM, Some(c), None);
        }
    }
    if bitvector & FIND_CHAR_WORLD != 0 {
        if let Some(c) = get_char_vis(w, ch, &name) {
            return (FIND_CHAR_WORLD, Some(c), None);
        }
    }
    if bitvector & FIND_OBJ_EQUIP != 0 {
        for i in 0..MAX_WEAR {
            if let Some(o) = w[ch].equipment[i] {
                if str_cmp(&name, &w[o].name) == 0 {
                    return (FIND_OBJ_EQUIP, None, Some(o));
                }
            }
        }
    }
    if bitvector & FIND_OBJ_INV != 0 {
        if let Some(o) = get_obj_in_list_vis(w, ch, &name, &w[ch].carrying) {
            return (FIND_OBJ_INV, None, Some(o));
        }
    }
    if bitvector & FIND_OBJ_ROOM != 0 {
        if let Some(o) = get_obj_in_list_vis(w, ch, &name, &w.room(w[ch].in_room).contents) {
            return (FIND_OBJ_ROOM, None, Some(o));
        }
    }
    if bitvector & FIND_OBJ_WORLD != 0 {
        if let Some(o) = get_obj_vis(w, ch, &name) {
            return (FIND_OBJ_WORLD, None, Some(o));
        }
    }
    (0, None, None)
}
