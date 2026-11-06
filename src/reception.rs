//! Inns: renting, storing and restoring player equipment (reception.c).

use crate::act_social::do_action;
use crate::comm::*;
use crate::db::*;
use crate::game::World;
use crate::handler::*;
use crate::save::*;
use crate::structs::*;
use crate::utils::*;

struct ObjCost {
    total_cost: i32,
    no_carried: usize,
    ok: bool,
}

fn add_obj_cost(w: &mut World, ch: CharId, re: CharId, list: &[ObjId], cost: &mut ObjCost) {
    for &obj in list {
        if w[obj].item_number > -1 && cost.ok {
            cost.total_cost += w[obj].obj_flags.cost_per_day.max(0);
            cost.no_carried += 1;
            let inner = w[obj].contains.clone();
            add_obj_cost(w, ch, re, &inner, cost);
        } else if cost.ok {
            act(w, "$n tells you 'I refuse storing $p'", false, re, Some(obj), Vict::Ch(ch), TO_VICT);
            cost.ok = false;
        }
    }
}

fn recep_offer(w: &mut World, ch: CharId, recep: CharId, cost: &mut ObjCost) -> bool {
    cost.total_cost = 100;
    cost.no_carried = 0;
    cost.ok = true;
    let carrying = w[ch].carrying.clone();
    add_obj_cost(w, ch, recep, &carrying, cost);
    for i in 0..MAX_WEAR {
        if let Some(o) = w[ch].equipment[i] {
            add_obj_cost(w, ch, recep, &[o], cost);
        }
    }
    if !cost.ok {
        return false;
    }
    if cost.no_carried == 0 {
        act(w, "$n tells you 'But you are not carrying anything?'", false, recep, None, Vict::Ch(ch), TO_VICT);
        return false;
    }
    if cost.no_carried > MAX_OBJ_SAVE {
        let buf = format!("$n tells you 'Sorry, but I can't store any more than {} items.", MAX_OBJ_SAVE);
        act(w, &buf, false, recep, None, Vict::Ch(ch), TO_VICT);
        return false;
    }
    let buf = format!("$n tells you 'It will cost you {} coins per day'", cost.total_cost);
    act(w, &buf, false, recep, None, Vict::Ch(ch), TO_VICT);
    if cost.total_cost > w[ch].points.gold {
        if w.level(ch) < 21 {
            act(w, "$n tells you 'Which I can see you can't afford'", false, recep, None, Vict::Ch(ch), TO_VICT);
        } else {
            act(w, "$n tells you 'Well, since you're a God, I guess it's okay'", false, recep, None, Vict::Ch(ch), TO_VICT);
            cost.total_cost = 0;
        }
    }
    cost.total_cost <= w[ch].points.gold
}

/// Load a character's rented equipment.
pub fn load_char_objs(w: &mut World, ch: CharId) {
    let name = w[ch].player.name.clone();
    match read_obj_file(&name) {
        Some(st) => {
            for e in &st.objects {
                if e.item_number > -1 && real_object(w, e.item_number) > -1 {
                    if let Some(obj) = read_object(w, e.item_number, VIRTUAL) {
                        let o = &mut w[obj];
                        o.obj_flags.value = e.value;
                        o.obj_flags.extra_flags = e.extra_flags;
                        o.obj_flags.weight = e.weight;
                        o.obj_flags.timer = e.timer;
                        o.obj_flags.bitvector = e.bitvector;
                        o.affected = e.affected;
                        obj_to_char(w, obj, ch);
                    }
                }
            }
            let timegold = (st.total_cost as f64 * (now() - st.last_update) as f64 / SECS_PER_REAL_DAY as f64) as i32;
            w[ch].points.gold -= timegold;
            if w[ch].points.gold < 0 {
                w[ch].points.gold = 0;
            }
            remove_obj_file(&name);
        }
        None => log("Char has no data in file!"),
    }
    save_char(w, ch, NOWHERE);
}

fn put_obj_in_store(w: &World, obj: ObjId, st: &mut ObjFile) {
    let o = &w[obj];
    st.objects.push(crate::save::ObjFileElem {
        item_number: w.obj_index[o.item_number as usize].virtual_nr,
        value: o.obj_flags.value,
        extra_flags: o.obj_flags.extra_flags,
        weight: o.obj_flags.weight,
        timer: o.obj_flags.timer,
        bitvector: o.obj_flags.bitvector,
        affected: o.affected,
    });
}

fn obj_to_store(w: &mut World, list: &[ObjId], st: &mut ObjFile, ch: CharId) {
    for &obj in list {
        if !w.obj_exists(obj) {
            continue;
        }
        let inner = w[obj].contains.clone();
        obj_to_store(w, &inner, st, ch);
        let timer = w[obj].obj_flags.timer;
        if timer < 0 && timer != OBJ_NOTIMER {
            let buf = format!(
                "You're told: 'The {} is just old junk, I'll throw it away for you.'\n\r",
                fname(&w[obj].name)
            );
            send_to_char(w, &buf, ch);
        } else {
            put_obj_in_store(w, obj, st);
            if w[obj].in_obj.is_some() {
                obj_from_obj(w, obj);
            }
            extract_obj(w, obj);
        }
    }
}

fn save_obj(w: &mut World, ch: CharId, cost: &ObjCost) {
    let mut st = ObjFile {
        owner: w[ch].player.name.clone(),
        gold_left: w[ch].points.gold,
        total_cost: cost.total_cost,
        last_update: now(),
        objects: Vec::new(),
    };
    let carrying = w[ch].carrying.clone();
    obj_to_store(w, &carrying, &mut st, ch);
    // anything left (junk that was not stored) goes away with the char
    w[ch].carrying.clear();
    w[ch].specials.carry_weight = 0;
    w[ch].specials.carry_items = 0;
    for i in 0..MAX_WEAR {
        if let Some(o) = w[ch].equipment[i] {
            obj_to_store(w, &[o], &mut st, ch);
            // the object is gone already; clear the slot and recompute affects
            w[ch].equipment[i] = None;
        }
    }
    affect_total(w, ch);
    write_obj_file(&st);
}

/// At boot: charge rent for the days the game was down.
pub fn update_obj_file(w: &mut World) {
    for mut st in all_obj_files() {
        if st.owner.is_empty() {
            continue;
        }
        log(&format!("   Processing {}.", st.owner));
        let t = now();
        let days_passed = (t - st.last_update) / SECS_PER_REAL_DAY;
        let secs_lost = (t - st.last_update) % SECS_PER_REAL_DAY;
        if days_passed > 0 {
            if st.total_cost as i64 * days_passed > st.gold_left as i64 {
                if find_name(w, &st.owner).is_none() {
                    log("   Character not in list. (update_obj_file)");
                    continue;
                }
                log(&format!("   Dumping {} from object file.", st.owner));
                modify_stored_char(&st.owner, |c| {
                    c.points.gold = 0;
                    c.load_room = NOWHERE;
                });
                remove_obj_file(&st.owner);
            } else {
                log(&format!("   Updating {}", st.owner));
                st.gold_left -= (st.total_cost as i64 * days_passed) as i32;
                st.last_update = t - secs_lost;
                write_obj_file(&st);
            }
        }
    }
}

pub fn receptionist(w: &mut World, ch: CharId, cmd: i32, _arg: &str) -> bool {
    const ACTION_TABEL: [i32; 9] = [23, 24, 36, 105, 106, 109, 111, 142, 147];
    if w[ch].desc.is_none() {
        return false;
    }
    let recep = w
        .room(w[ch].in_room)
        .people
        .iter()
        .copied()
        .find(|&t| w.is_mob(t) && w.mob_index[w[t].nr as usize].spec == "receptionist");
    let Some(recep) = recep else {
        log("Ingen receptionist.");
        return false;
    };
    if w.is_npc(ch) {
        return false;
    }
    if cmd != 92 && cmd != 93 {
        if number(0, 30) == 0 {
            do_action(w, recep, "", ACTION_TABEL[number(0, 8) as usize]);
        }
        return false;
    }
    if !w.awake(recep) {
        act(w, "$e isn't able to talk to you...", false, recep, None, Vict::Ch(ch), TO_VICT);
        return true;
    }
    if !w.can_see(recep, ch) {
        act(w, "$n says, 'I don't deal with people I can't see!'", false, recep, None, Vict::None, TO_ROOM);
        return true;
    }
    let mut cost = ObjCost { total_cost: 0, no_carried: 0, ok: true };
    if cmd == 92 {
        if recep_offer(w, ch, recep, &mut cost) {
            act(w, "$n stores your stuff in the safe, and helps you into your chamber.", false, recep, None, Vict::Ch(ch), TO_VICT);
            act(w, "$n helps $N into $S private chamber.", false, recep, None, Vict::Ch(ch), TO_NOTVICT);
            save_obj(w, ch, &cost);
            let save_room = w[ch].in_room;
            extract_char(w, ch);
            if w.ch_exists(ch) {
                let vnum = w.room(save_room).number;
                w[ch].in_room = vnum;
                save_char(w, ch, vnum);
            }
        }
    } else {
        recep_offer(w, ch, recep, &mut cost);
        act(w, "$N gives $n an offer.", false, ch, None, Vict::Ch(recep), TO_ROOM);
    }
    true
}
