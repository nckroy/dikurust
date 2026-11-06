//! Miscellaneous player commands (act.other.c).

use std::io::Write as _;

use crate::act_comm::do_shout;
use crate::comm::*;
use crate::constants::DEX_APP_SKILL;
use crate::db::{BUG_FILE, IDEA_FILE, TYPO_FILE};
use crate::fight::{die, hit};
use crate::game::World;
use crate::handler::*;
use crate::interpreter::one_argument;
use crate::save::save_char;
use crate::spell_parser::spell_info;
use crate::spells::*;
use crate::structs::*;
use crate::utils::*;


pub fn do_qui(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    send_to_char(w, "You have to write quit - no less, to quit!\n\r", ch);
}

pub fn do_quit(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    if w.is_npc(ch) || w[ch].desc.is_none() {
        return;
    }
    if w.get_pos(ch) == POSITION_FIGHTING {
        send_to_char(w, "No way! You are fighting.\n\r", ch);
        return;
    }
    if w.get_pos(ch) < POSITION_STUNNED {
        send_to_char(w, "You die before your time!\n\r", ch);
        die(w, ch);
        return;
    }
    act(w, "Goodbye, friend.. Come back soon!", false, ch, None, Vict::None, TO_CHAR);
    act(w, "$n has left the game.", true, ch, None, Vict::None, TO_ROOM);
    extract_char(w, ch);
}

pub fn do_save(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    if w.is_npc(ch) || w[ch].desc.is_none() {
        return;
    }
    let buf = format!("Saving {}.\n\r", w[ch].player.name);
    send_to_char(w, &buf, ch);
    save_char(w, ch, NOWHERE);
}

pub fn do_not_here(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    send_to_char(w, "Sorry, but you cannot do that here!\n\r", ch);
}

fn dex(w: &World, ch: CharId) -> usize {
    w[ch].tmpabilities.dex.clamp(0, 25) as usize
}

pub fn do_sneak(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    send_to_char(w, "Ok, you'll try to move silently for a while.\n\r", ch);
    if w.is_affected(ch, AFF_SNEAK) {
        affect_from_char(w, ch, SKILL_SNEAK);
    }
    let percent = number(1, 101);
    if percent > w[ch].skills[SKILL_SNEAK as usize].learned + DEX_APP_SKILL[dex(w, ch)].sneak {
        return;
    }
    let af = Affect { type_: SKILL_SNEAK, duration: w.level(ch), modifier: 0, location: APPLY_NONE, bitvector: AFF_SNEAK };
    affect_to_char(w, ch, &af);
}

pub fn do_hide(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    send_to_char(w, "You attempt to hide yourself.\n\r", ch);
    w[ch].specials.affected_by &= !AFF_HIDE;
    let percent = number(1, 101);
    if percent > w[ch].skills[SKILL_HIDE as usize].learned + DEX_APP_SKILL[dex(w, ch)].hide {
        return;
    }
    w[ch].specials.affected_by |= AFF_HIDE;
}

pub fn do_steal(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (obj_name, rest) = one_argument(argument);
    let (victim_name, _) = one_argument(rest);
    let Some(victim) = get_char_room_vis(w, ch, &victim_name) else {
        send_to_char(w, "Steal what from who?\n\r", ch);
        return;
    };
    if victim == ch {
        send_to_char(w, "Come on now, that's rather stupid!\n\r", ch);
        return;
    }
    if w[ch].points.exp < 1250 && !w.is_npc(victim) {
        send_to_char(w, "Due to misuse of steal, you can't steal from other players\n\r", ch);
        send_to_char(w, "unless you got at least 1,250 experience points.\n\r", ch);
        return;
    }
    w.wait_state(ch, 10);
    let mut percent = number(1, 101) - DEX_APP_SKILL[dex(w, ch)].p_pocket;
    percent += if w.awake(victim) { 10 } else { -50 };
    if w.get_pos(victim) < POSITION_SLEEPING {
        percent = -1;
    }
    if w.level(victim) > 20 {
        percent = 101;
    }
    let learned = w[ch].skills[SKILL_STEAL as usize].learned;
    let mut ohoh = false;

    if str_cmp(&obj_name, "coins") != 0 && str_cmp(&obj_name, "gold") != 0 {
        match get_obj_in_list_vis(w, ch, &obj_name, &w[victim].carrying) {
            None => {
                let mut found = None;
                for eq_pos in 0..MAX_WEAR {
                    if let Some(o) = w[victim].equipment[eq_pos] {
                        if isname(&obj_name, &w[o].name) && w.can_see_obj(ch, o) {
                            found = Some((o, eq_pos));
                            break;
                        }
                    }
                }
                let Some((obj, eq_pos)) = found else {
                    act(w, "$E has not got that item.", false, ch, None, Vict::Ch(victim), TO_CHAR);
                    return;
                };
                if w.get_pos(victim) > POSITION_STUNNED {
                    send_to_char(w, "Steal the equipment now? Impossible!\n\r", ch);
                    return;
                }
                act(w, "You unequip $p and steal it.", false, ch, Some(obj), Vict::None, TO_CHAR);
                act(w, "$n steals $p from $N.", false, ch, Some(obj), Vict::Ch(victim), TO_NOTVICT);
                if let Some(o) = unequip_char(w, victim, eq_pos) {
                    obj_to_char(w, o, ch);
                }
            }
            Some(obj) => {
                percent += w[obj].obj_flags.weight;
                if percent > learned {
                    ohoh = true;
                    act(w, "Oops..", false, ch, None, Vict::None, TO_CHAR);
                    act(w, "$n tried to steal something from you!", false, ch, None, Vict::Ch(victim), TO_VICT);
                    act(w, "$n tries to steal something from $N.", true, ch, None, Vict::Ch(victim), TO_NOTVICT);
                } else if w[ch].specials.carry_items + 1 < w.can_carry_n(ch) {
                    if w[ch].specials.carry_weight + w[obj].obj_flags.weight < w.can_carry_w(ch) {
                        obj_from_char(w, obj);
                        obj_to_char(w, obj, ch);
                        send_to_char(w, "Got it!\n\r", ch);
                    }
                } else {
                    send_to_char(w, "You cannot carry that much.\n\r", ch);
                }
            }
        }
    } else if percent > learned {
        ohoh = true;
        act(w, "Oops..", false, ch, None, Vict::None, TO_CHAR);
        act(w, "You discover that $n has $s hands in your wallet.", false, ch, None, Vict::Ch(victim), TO_VICT);
        act(w, "$n tries to steal gold from $N.", true, ch, None, Vict::Ch(victim), TO_NOTVICT);
    } else {
        let gold = ((w[victim].points.gold as i64 * number(1, 10) as i64) / 100).min(1782) as i32;
        if gold > 0 {
            w[ch].points.gold += gold;
            w[victim].points.gold -= gold;
            send_to_char(w, &format!("Bingo! You got {} gold coins.\n\r", gold), ch);
        } else {
            send_to_char(w, "You couldn't get any gold...\n\r", ch);
        }
    }

    if ohoh && w.is_npc(victim) && w.awake(victim) {
        if w[victim].specials.act & ACT_NICE_THIEF != 0 {
            let buf = format!("{} is a bloody thief.", w[ch].player.name);
            do_shout(w, victim, &buf, 0);
            log(&buf);
            send_to_char(w, "Don't you ever do that again!\n\r", ch);
        } else {
            hit(w, victim, ch, TYPE_UNDEFINED);
        }
    }
}

pub fn do_practice(w: &mut World, ch: CharId, _arg: &str, _cmd: i32) {
    send_to_char(w, "You can only practise in a guild.\n\r", ch);
}

fn append_file(name: &str, text: &str) -> bool {
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(name)
        .and_then(|mut f| f.write_all(text.as_bytes()))
        .is_ok()
}

pub fn do_idea(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        send_to_char(w, "Monsters can't have ideas - Go away.\n\r", ch);
        return;
    }
    let a = argument.trim_start();
    if a.is_empty() {
        send_to_char(w, "That doesn't sound like a good idea to me.. Sorry.\n\r", ch);
        return;
    }
    let s = format!("**{}: {}\n", w[ch].player.name, a);
    if append_file(IDEA_FILE, &s) {
        send_to_char(w, "Ok. Thanks.\n\r", ch);
    } else {
        send_to_char(w, "Could not open the idea-file.\n\r", ch);
    }
}

pub fn do_typo(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        send_to_char(w, "Monsters can't spell - leave me alone.\n\r", ch);
        return;
    }
    let a = argument.trim_start();
    if a.is_empty() {
        send_to_char(w, "I beg your pardon?\n\r", ch);
        return;
    }
    let s = format!("**{}[{}]: {}\n", w[ch].player.name, w.room(w[ch].in_room).number, a);
    if append_file(TYPO_FILE, &s) {
        send_to_char(w, "Ok. thanks.\n\r", ch);
    } else {
        send_to_char(w, "Could not open the typo-file.\n\r", ch);
    }
}

pub fn do_bug(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        send_to_char(w, "You are a monster! Bug off!\n\r", ch);
        return;
    }
    let a = argument.trim_start();
    if a.is_empty() {
        send_to_char(w, "Pardon?\n\r", ch);
        return;
    }
    let s = format!("**{}[{}]: {}\n", w[ch].player.name, w.room(w[ch].in_room).number, a);
    if append_file(BUG_FILE, &s) {
        send_to_char(w, "Ok.\n\r", ch);
    } else {
        send_to_char(w, "Could not open the bug-file.\n\r", ch);
    }
}

pub fn do_brief(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        return;
    }
    if w[ch].specials.act & PLR_BRIEF != 0 {
        send_to_char(w, "Brief mode off.\n\r", ch);
        w[ch].specials.act &= !PLR_BRIEF;
    } else {
        send_to_char(w, "Brief mode on.\n\r", ch);
        w[ch].specials.act |= PLR_BRIEF;
    }
}

pub fn do_compact(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        return;
    }
    if w[ch].specials.act & PLR_COMPACT != 0 {
        send_to_char(w, "You are now in the uncompacted mode.\n\r", ch);
        w[ch].specials.act &= !PLR_COMPACT;
    } else {
        send_to_char(w, "You are now in compact mode.\n\r", ch);
        w[ch].specials.act |= PLR_COMPACT;
    }
}

pub fn do_group(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (name, _) = one_argument(argument);
    if name.is_empty() {
        if !w.is_affected(ch, AFF_GROUP) {
            send_to_char(w, "But you are a member of no group?!\n\r", ch);
        } else {
            send_to_char(w, "Your group consists of:\n\r", ch);
            let k = w[ch].master.unwrap_or(ch);
            if w.is_affected(k, AFF_GROUP) {
                act(w, "     $N (Head of group)", false, ch, None, Vict::Ch(k), TO_CHAR);
            }
            for f in w[k].followers.clone() {
                if w.is_affected(f, AFF_GROUP) {
                    act(w, "     $N", false, ch, None, Vict::Ch(f), TO_CHAR);
                }
            }
        }
        return;
    }
    let Some(victim) = get_char_room_vis(w, ch, &name) else {
        send_to_char(w, "No one here by that name.\n\r", ch);
        return;
    };
    if w[ch].master.is_some() {
        act(w, "You can not enroll group members without being head of a group.", false, ch, None, Vict::None, TO_CHAR);
        return;
    }
    let found = victim == ch || w[ch].followers.contains(&victim);
    if found {
        if w.is_affected(victim, AFF_GROUP) {
            act(w, "$n has been kicked out of the group!", false, victim, None, Vict::Ch(ch), TO_ROOM);
            act(w, "You are no longer a member of the group!", false, victim, None, Vict::None, TO_CHAR);
            w[victim].specials.affected_by &= !AFF_GROUP;
        } else {
            act(w, "$n is now a group member.", false, victim, None, Vict::None, TO_ROOM);
            act(w, "You are now a group member.", false, victim, None, Vict::None, TO_CHAR);
            w[victim].specials.affected_by |= AFF_GROUP;
        }
    } else {
        act(w, "$N must follow you, to enter the group", false, ch, None, Vict::Ch(victim), TO_CHAR);
    }
}

/// Find an item in the inventory, or held in the hand.  (obj, equipped?)
fn find_carried_or_held(w: &World, ch: CharId, name: &str) -> Option<(ObjId, bool)> {
    if let Some(o) = get_obj_in_list_vis(w, ch, name, &w[ch].carrying) {
        return Some((o, false));
    }
    match w[ch].equipment[HOLD] {
        Some(o) if isname(name, &w[o].name) => Some((o, true)),
        _ => None,
    }
}

fn cast_from_item(w: &mut World, spell: i32, level: i32, ch: CharId, typ: i32, v: Option<CharId>, o: Option<ObjId>) {
    if spell < 1 {
        return;
    }
    match spell_info(spell).spell_pointer {
        Some(f) if w.ch_exists(ch) => f(w, level, ch, "", typ, v, o),
        _ => {}
    }
}

pub fn do_quaff(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (buf, _) = one_argument(argument);
    let Some((temp, equipped)) = find_carried_or_held(w, ch, &buf) else {
        act(w, "You do not have that item.", false, ch, None, Vict::None, TO_CHAR);
        return;
    };
    if w[temp].obj_flags.type_flag != ITEM_POTION {
        act(w, "You can only quaff potions.", false, ch, None, Vict::None, TO_CHAR);
        return;
    }
    act(w, "$n quaffs $p.", true, ch, Some(temp), Vict::None, TO_ROOM);
    act(w, "You quaff $p which dissolves.", false, ch, Some(temp), Vict::None, TO_CHAR);
    let v = w[temp].obj_flags.value;
    for i in 1..4 {
        if v[i] >= 1 {
            cast_from_item(w, v[i], v[0], ch, SPELL_TYPE_POTION, Some(ch), None);
        }
    }
    if !w.obj_exists(temp) {
        return;
    }
    if equipped && w.ch_exists(ch) && w[ch].equipment[HOLD] == Some(temp) {
        unequip_char(w, ch, HOLD);
    }
    extract_obj(w, temp);
}

pub fn do_recite(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (buf, rest) = one_argument(argument);
    let Some((scroll, equipped)) = find_carried_or_held(w, ch, &buf) else {
        act(w, "You do not have that item.", false, ch, None, Vict::None, TO_CHAR);
        return;
    };
    if w[scroll].obj_flags.type_flag != ITEM_SCROLL {
        act(w, "Recite is normally used for scroll's.", false, ch, None, Vict::None, TO_CHAR);
        return;
    }
    let (victim, obj) = if !rest.trim().is_empty() {
        let (bits, v, o) = generic_find(w, rest, FIND_OBJ_INV | FIND_OBJ_ROOM | FIND_OBJ_EQUIP | FIND_CHAR_ROOM, ch);
        if bits == 0 {
            send_to_char(w, "No such thing around to recite the scroll on.\n\r", ch);
            return;
        }
        (v, o)
    } else {
        (Some(ch), None)
    };
    act(w, "$n recites $p.", true, ch, Some(scroll), Vict::None, TO_ROOM);
    act(w, "You recite $p which dissolves.", false, ch, Some(scroll), Vict::None, TO_CHAR);
    let v = w[scroll].obj_flags.value;
    for i in 1..4 {
        if v[i] >= 1 {
            let victim = victim.filter(|c| w.ch_exists(*c));
            let obj = obj.filter(|o| w.obj_exists(*o));
            cast_from_item(w, v[i], v[0], ch, SPELL_TYPE_SCROLL, victim, obj);
        }
    }
    if !w.obj_exists(scroll) {
        return;
    }
    if equipped && w.ch_exists(ch) && w[ch].equipment[HOLD] == Some(scroll) {
        unequip_char(w, ch, HOLD);
    }
    extract_obj(w, scroll);
}

pub fn do_use(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (buf, rest) = one_argument(argument);
    let stick = match w[ch].equipment[HOLD] {
        Some(o) if isname(&buf, &w[o].name) => o,
        _ => {
            act(w, "You do not hold that item in your hand.", false, ch, None, Vict::None, TO_CHAR);
            return;
        }
    };
    match w[stick].obj_flags.type_flag {
        ITEM_STAFF => {
            act(w, "$n taps $p three times on the ground.", true, ch, Some(stick), Vict::None, TO_ROOM);
            act(w, "You tap $p three times on the ground.", false, ch, Some(stick), Vict::None, TO_CHAR);
            if w[stick].obj_flags.value[2] > 0 {
                w[stick].obj_flags.value[2] -= 1;
                let v = w[stick].obj_flags.value;
                cast_from_item(w, v[3], v[0], ch, SPELL_TYPE_STAFF, None, None);
            } else {
                send_to_char(w, "The staff seems powerless.\n\r", ch);
            }
        }
        ITEM_WAND => {
            let (bits, tc, to) = generic_find(w, rest, FIND_CHAR_ROOM | FIND_OBJ_INV | FIND_OBJ_ROOM | FIND_OBJ_EQUIP, ch);
            if bits == 0 {
                send_to_char(w, "What should the wand be pointed at?\n\r", ch);
                return;
            }
            if bits == FIND_CHAR_ROOM {
                let t = tc.unwrap();
                act(w, "$n point $p at $N.", true, ch, Some(stick), Vict::Ch(t), TO_ROOM);
                act(w, "You point $p at $N.", false, ch, Some(stick), Vict::Ch(t), TO_CHAR);
            } else {
                let o = to.unwrap();
                act(w, "$n point $p at $P.", true, ch, Some(stick), Vict::Obj(o), TO_ROOM);
                act(w, "You point $p at $P.", false, ch, Some(stick), Vict::Obj(o), TO_CHAR);
            }
            if w[stick].obj_flags.value[2] > 0 {
                w[stick].obj_flags.value[2] -= 1;
                let v = w[stick].obj_flags.value;
                cast_from_item(w, v[3], v[0], ch, SPELL_TYPE_WAND, tc, to);
            } else {
                send_to_char(w, "The wand seems powerless.\n\r", ch);
            }
        }
        _ => send_to_char(w, "Use is normally only for wand's and staff's.\n\r", ch),
    }
}
