//! Limits, gains and periodic point updates (limits.c).

use crate::comm::*;
use crate::constants::*;
use crate::fight::{damage, stop_fighting, update_pos};
use crate::game::World;
use crate::handler::*;
use crate::spells::*;
use crate::structs::*;
use crate::utils::*;

pub fn graf(age: i32, p0: i32, p1: i32, p2: i32, p3: i32, p4: i32, p5: i32, p6: i32) -> i32 {
    if age < 15 {
        p0
    } else if age <= 29 {
        p1 + ((age - 15) * (p2 - p1)) / 15
    } else if age <= 44 {
        p2 + ((age - 30) * (p3 - p2)) / 15
    } else if age <= 59 {
        p3 + ((age - 45) * (p4 - p3)) / 15
    } else if age <= 79 {
        p4 + ((age - 60) * (p5 - p4)) / 20
    } else {
        p6
    }
}

fn age_year(w: &World, ch: CharId) -> i32 {
    age(&w[ch]).year
}

pub fn mana_limit(_w: &World, _ch: CharId) -> i32 {
    100
}

pub fn hit_limit(w: &World, ch: CharId) -> i32 {
    if !w.is_npc(ch) {
        w[ch].points.max_hit + graf(age_year(w, ch), 2, 4, 17, 14, 8, 4, 3)
    } else {
        w[ch].points.max_hit
    }
}

pub fn move_limit(w: &World, ch: CharId) -> i32 {
    if !w.is_npc(ch) {
        graf(age_year(w, ch), 50, 70, 160, 120, 100, 40, 20)
    } else {
        w[ch].points.max_move
    }
}

fn starving(w: &World, ch: CharId) -> bool {
    w[ch].specials.conditions[FULL] == 0 || w[ch].specials.conditions[THIRST] == 0
}

pub fn mana_gain(w: &World, ch: CharId) -> i32 {
    let mut gain;
    if w.is_npc(ch) {
        gain = w.level(ch);
    } else {
        gain = graf(age_year(w, ch), 2, 4, 6, 8, 6, 5, 8);
        match w.get_pos(ch) {
            POSITION_SLEEPING => gain += gain,
            POSITION_RESTING => gain += gain >> 1,
            POSITION_SITTING => gain += gain >> 2,
            _ => {}
        }
        let c = w[ch].player.class;
        if c == CLASS_MAGIC_USER || c == CLASS_CLERIC {
            gain += gain;
        }
    }
    if w.is_affected(ch, AFF_POISON) {
        gain >>= 2;
    }
    if starving(w, ch) {
        gain >>= 2;
    }
    gain
}

/// hit_gain() without the poison damage side effect (used by stat).
pub fn hit_gain_calc(w: &World, ch: CharId) -> i32 {
    let mut gain;
    if w.is_npc(ch) {
        gain = w.level(ch);
    } else {
        gain = graf(age_year(w, ch), 2, 5, 10, 18, 6, 4, 2);
        match w.get_pos(ch) {
            POSITION_SLEEPING => gain += gain >> 1,
            POSITION_RESTING => gain += gain >> 2,
            POSITION_SITTING => gain += gain >> 3,
            _ => {}
        }
        let c = w[ch].player.class;
        if c == CLASS_MAGIC_USER || c == CLASS_CLERIC {
            gain >>= 1;
        }
    }
    if w.is_affected(ch, AFF_POISON) {
        gain >>= 2;
    }
    if starving(w, ch) {
        gain >>= 2;
    }
    gain
}

pub fn hit_gain(w: &mut World, ch: CharId) -> i32 {
    let gain = hit_gain_calc(w, ch);
    if w.is_affected(ch, AFF_POISON) {
        damage(w, ch, ch, 2, SPELL_POISON);
        if !w.ch_exists(ch) {
            return 0;
        }
    }
    gain
}

pub fn move_gain(w: &World, ch: CharId) -> i32 {
    if w.is_npc(ch) {
        return w.level(ch);
    }
    let mut gain = graf(age_year(w, ch), 12, 18, 22, 21, 14, 10, 6);
    match w.get_pos(ch) {
        POSITION_SLEEPING => gain += gain >> 1,
        POSITION_RESTING => gain += gain >> 2,
        POSITION_SITTING => gain += gain >> 3,
        _ => {}
    }
    if w.is_affected(ch, AFF_POISON) {
        gain >>= 2;
    }
    if starving(w, ch) {
        gain >>= 2;
    }
    gain
}

pub fn advance_level(w: &mut World, ch: CharId) {
    let mut add_hp = CON_APP[w[ch].tmpabilities.con.clamp(0, 25) as usize].hitp;
    add_hp += match w[ch].player.class {
        CLASS_MAGIC_USER => number(3, 8),
        CLASS_CLERIC => number(5, 10),
        CLASS_THIEF => number(7, 13),
        CLASS_WARRIOR => number(10, 15),
        _ => 0,
    };
    w[ch].points.max_hit += add_hp.max(1);
    let bonus = WIS_APP[w[ch].tmpabilities.wis.clamp(0, 25) as usize];
    let c = w[ch].player.class;
    if c == CLASS_MAGIC_USER || c == CLASS_CLERIC {
        w[ch].specials.spells_to_learn += bonus.max(2);
    } else {
        w[ch].specials.spells_to_learn += bonus.max(1).min(2);
    }
    if w.level(ch) > 20 {
        w[ch].specials.conditions = [-1; 3];
    }
}

pub fn read_title(w: &World, ch: CharId) -> &'static str {
    let c = &w[ch];
    let cls = (c.player.class - 1).clamp(0, 3) as usize;
    let lvl = c.player.level.clamp(0, 24) as usize;
    if c.player.sex == SEX_MALE {
        TITLES[cls][lvl].title_m
    } else {
        TITLES[cls][lvl].title_f
    }
}

pub fn set_title(w: &mut World, ch: CharId) {
    let t = read_title(w, ch).to_string();
    w[ch].player.title = t;
}

pub fn gain_exp(w: &mut World, ch: CharId, gain: i32) {
    if !w.ch_exists(ch) {
        return;
    }
    let lvl = w.level(ch);
    let mut is_altered = false;
    if w.is_npc(ch) || (lvl < 21 && lvl > 0) {
        if gain > 0 {
            let gain = gain.min(100000);
            w[ch].points.exp += gain;
            if !w.is_npc(ch) {
                let cls = (w[ch].player.class - 1).clamp(0, 3) as usize;
                let mut i = 0;
                while i < 25 && TITLES[cls][i].exp <= w[ch].points.exp {
                    if i as i32 > w.level(ch) {
                        send_to_char(w, "You raise a level\n\r", ch);
                        w[ch].player.level = i as i32;
                        advance_level(w, ch);
                        is_altered = true;
                    }
                    i += 1;
                }
            }
        }
        if gain < 0 {
            let gain = gain.max(-500000);
            w[ch].points.exp += gain;
            if w[ch].points.exp < 0 {
                w[ch].points.exp = 0;
            }
        }
        if is_altered {
            set_title(w, ch);
        }
    }
}

pub fn gain_exp_regardless(w: &mut World, ch: CharId, gain: i32) {
    let mut is_altered = false;
    if !w.is_npc(ch) {
        if gain > 0 {
            w[ch].points.exp = w[ch].points.exp.saturating_add(gain);
            let cls = (w[ch].player.class - 1).clamp(0, 3) as usize;
            let mut i = 0;
            while i < 25 && TITLES[cls][i].exp <= w[ch].points.exp {
                if i as i32 > w.level(ch) {
                    send_to_char(w, "You raise a level\n\r", ch);
                    w[ch].player.level = i as i32;
                    advance_level(w, ch);
                    is_altered = true;
                }
                i += 1;
            }
        }
        if gain < 0 {
            w[ch].points.exp += gain;
        }
        if w[ch].points.exp < 0 {
            w[ch].points.exp = 0;
        }
    }
    if is_altered {
        set_title(w, ch);
    }
}

pub fn gain_condition(w: &mut World, ch: CharId, condition: usize, value: i32) {
    if w[ch].specials.conditions[condition] == -1 {
        return;
    }
    let intoxicated = w[ch].specials.conditions[DRUNK] > 0;
    let c = &mut w[ch].specials.conditions[condition];
    *c = (*c + value).clamp(0, 24);
    if *c != 0 {
        return;
    }
    match condition {
        FULL => send_to_char(w, "You are hungry.\n\r", ch),
        THIRST => send_to_char(w, "You are thirsty.\n\r", ch),
        DRUNK => {
            if intoxicated {
                send_to_char(w, "You are now sober.\n\r", ch);
            }
        }
        _ => {}
    }
}

pub fn check_idling(w: &mut World, ch: CharId) {
    w[ch].specials.timer += 1;
    if w[ch].specials.timer <= 8 {
        return;
    }
    if w[ch].specials.was_in_room == NOWHERE && w[ch].in_room != NOWHERE {
        w[ch].specials.was_in_room = w[ch].in_room;
        if let Some(f) = w[ch].specials.fighting {
            if w.ch_exists(f) {
                stop_fighting(w, f);
            }
            stop_fighting(w, ch);
        }
        act(w, "$n disappears into the void.", true, ch, None, Vict::None, TO_ROOM);
        send_to_char(w, "You have been idle, and are pulled into a void.\n\r", ch);
        char_from_room(w, ch);
        char_to_room(w, ch, 1);
    } else if w[ch].specials.timer > 48 {
        if w[ch].in_room != NOWHERE {
            char_from_room(w, ch);
        }
        let was = w[ch].specials.was_in_room;
        if was != NOWHERE {
            char_to_room(w, ch, was);
        } else {
            char_to_room(w, ch, 1);
        }
        if let Some(d) = w[ch].desc {
            crate::net::close_socket(w, d);
        }
        if w.ch_exists(ch) {
            w[ch].desc = None;
            extract_char(w, ch);
        }
    }
}

/// Update both PCs & NPCs and objects, once per mud hour.
pub fn point_update(w: &mut World) {
    for i in w.character_list.clone() {
        if !w.ch_exists(i) {
            continue;
        }
        let pos = w.get_pos(i);
        if pos >= POSITION_STUNNED {
            let hg = hit_gain(w, i);
            if !w.ch_exists(i) {
                continue;
            }
            let hl = hit_limit(w, i);
            let mg = mana_gain(w, i);
            let ml = mana_limit(w, i);
            let vg = move_gain(w, i);
            let vl = move_limit(w, i);
            let c = &mut w[i];
            c.points.hit = (c.points.hit + hg).min(hl);
            c.points.mana = (c.points.mana + mg).min(ml);
            c.points.mov = (c.points.mov + vg).min(vl);
            if pos == POSITION_STUNNED {
                update_pos(w, i);
            }
        } else if pos == POSITION_INCAP {
            damage(w, i, i, 1, TYPE_SUFFERING);
        } else if !w.is_npc(i) && pos == POSITION_MORTALLYW {
            damage(w, i, i, 2, TYPE_SUFFERING);
        }
        if !w.ch_exists(i) || !w.character_list.contains(&i) {
            continue;
        }
        if !w.is_npc(i) {
            update_char_objects(w, i);
            if w.level(i) < 22 {
                check_idling(w, i);
            }
        }
        if !w.ch_exists(i) || !w.character_list.contains(&i) {
            continue;
        }
        gain_condition(w, i, FULL, -1);
        gain_condition(w, i, DRUNK, -1);
        gain_condition(w, i, THIRST, -1);
    }

    for j in w.object_list.clone() {
        if !w.obj_exists(j) {
            continue;
        }
        if w[j].obj_flags.type_flag == ITEM_CONTAINER && w[j].obj_flags.value[3] != 0 {
            if w[j].obj_flags.timer > 0 {
                w[j].obj_flags.timer -= 1;
            }
            if w[j].obj_flags.timer == 0 {
                if let Some(c) = w[j].carried_by {
                    act(w, "$p decay in your hands.", false, c, Some(j), Vict::None, TO_CHAR);
                } else if w[j].in_room != NOWHERE {
                    if let Some(&p) = w.room(w[j].in_room).people.first() {
                        act(w, "A quivering hoard of maggots consume $p.", true, p, Some(j), Vict::None, TO_ROOM);
                        act(w, "A quivering hoard of maggots consume $p.", true, p, Some(j), Vict::None, TO_CHAR);
                    }
                }
                for jj in w[j].contains.clone() {
                    obj_from_obj(w, jj);
                    if let Some(io) = w[j].in_obj {
                        obj_to_obj(w, jj, io);
                    } else if let Some(c) = w[j].carried_by {
                        let r = w[c].in_room;
                        obj_to_room(w, jj, r);
                    } else if w[j].in_room != NOWHERE {
                        let r = w[j].in_room;
                        obj_to_room(w, jj, r);
                    } else {
                        extract_obj(w, jj);
                    }
                }
                extract_obj(w, j);
            }
        }
    }
}
