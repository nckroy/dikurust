//! Special procedures for rooms and mobiles (spec_procs.c).
//!
//! A special procedure is called with the *acting* character: for mobile
//! specials that is the mobile itself on its heartbeat (cmd == 0), or the
//! player who typed a command in the mobile's room (cmd > 0).

use std::cell::Cell;

use crate::act_comm::do_say;
use crate::act_info::do_look;
use crate::act_move::{do_close, do_lock, do_move, do_open, do_unlock};
use crate::act_obj1::do_drop;
use crate::comm::*;
use crate::db::{read_mobile, read_object, real_object, real_room, REAL};
use crate::fight::hit;
use crate::game::World;
use crate::handler::*;
use crate::interpreter::{one_argument, search_block};
use crate::limits::gain_exp;
use crate::spell_parser::{add_follower, spell_info, SPELLS};
use crate::spells::*;
use crate::structs::*;
use crate::utils::*;

fn how_good(percent: i32) -> &'static str {
    if percent == 0 {
        " (not learned)"
    } else if percent <= 10 {
        " (awful)"
    } else if percent <= 20 {
        " (bad)"
    } else if percent <= 40 {
        " (poor)"
    } else if percent <= 55 {
        " (average)"
    } else if percent <= 70 {
        " (fair)"
    } else if percent <= 80 {
        " (good)"
    } else if percent <= 85 {
        " (very good)"
    } else {
        " (Superb)"
    }
}

fn int_learn(w: &World, ch: CharId) -> i32 {
    crate::constants::INT_APP[w[ch].tmpabilities.intel.clamp(0, 25) as usize]
}

/// Practising spells or skills in the guilds.
pub fn guild(w: &mut World, ch: CharId, cmd: i32, arg: &str) -> bool {
    const W_SKILLS: [&str; 3] = ["kick", "bash", "rescue"];
    const T_SKILLS: [&str; 5] = ["sneak", "hide", "steal", "backstab", "pick"];
    if cmd != 164 && cmd != 170 {
        return false;
    }
    let arg = arg.trim_start_matches(' ');
    let class = w[ch].player.class;
    let lvl = w.level(ch);
    let sessions = w[ch].specials.spells_to_learn;

    let list_header = |w: &mut World, what: &str| {
        send_to_char(w, &format!("You have got {} practice sessions left.\n\r", sessions), ch);
        send_to_char(w, &format!("You can practise any of these {}:\n\r", what), ch);
    };

    match class {
        CLASS_MAGIC_USER | CLASS_CLERIC => {
            let min_level = |s: i32| {
                let si = spell_info(s);
                if class == CLASS_MAGIC_USER {
                    si.min_level_magic
                } else {
                    si.min_level_cleric
                }
            };
            if arg.is_empty() {
                list_header(w, "spells");
                for (i, name) in SPELLS.iter().enumerate() {
                    let s = i as i32 + 1;
                    if spell_info(s).spell_pointer.is_some() && min_level(s) <= lvl {
                        let b = format!("{}{}\n\r", name, how_good(w[ch].skills[s as usize].learned));
                        send_to_char(w, &b, ch);
                    }
                }
                return true;
            }
            // old_search_block(): prefix match, 1-based
            let a = arg.to_ascii_lowercase();
            let Some(pos) = SPELLS.iter().position(|s| s.starts_with(&a)) else {
                send_to_char(w, "You do not know of this spell...\n\r", ch);
                return true;
            };
            let number = pos + 1;
            if lvl < min_level(number as i32) {
                send_to_char(w, "You do not know of this spell...\n\r", ch);
                return true;
            }
            if sessions <= 0 {
                send_to_char(w, "You do not seem to be able to practice now.\n\r", ch);
                return true;
            }
            if w[ch].skills[number].learned >= 95 {
                send_to_char(w, "You are already learned in this area.\n\r", ch);
                return true;
            }
            send_to_char(w, "You Practice for a while...\n\r", ch);
            w[ch].specials.spells_to_learn -= 1;
            let percent = w[ch].skills[number].learned + int_learn(w, ch).max(25);
            w[ch].skills[number].learned = percent.min(95);
            if w[ch].skills[number].learned >= 95 {
                send_to_char(w, "You are now learned in this area.\n\r", ch);
            }
        }
        CLASS_THIEF | CLASS_WARRIOR => {
            let (list, base, cap, nomsg): (&[&str], usize, i32, &str) = if class == CLASS_THIEF {
                (&T_SKILLS, SKILL_SNEAK as usize, 85, "You do not know of this spell...\n\r")
            } else {
                (&W_SKILLS, SKILL_KICK as usize, 80, "You do not have ability to practise this skill!\n\r")
            };
            if arg.is_empty() {
                list_header(w, "skills");
                for (i, name) in list.iter().enumerate() {
                    let b = format!("{}{}\n\r", name, how_good(w[ch].skills[i + base].learned));
                    send_to_char(w, &b, ch);
                }
                return true;
            }
            let number = search_block(arg, list, false);
            if number == -1 {
                send_to_char(w, nomsg, ch);
                return true;
            }
            if sessions <= 0 {
                send_to_char(w, "You do not seem to be able to practice now.\n\r", ch);
                return true;
            }
            let idx = number as usize + base;
            if w[ch].skills[idx].learned >= cap {
                send_to_char(w, "You are already learned in this area.\n\r", ch);
                return true;
            }
            send_to_char(w, "You Practice for a while...\n\r", ch);
            w[ch].specials.spells_to_learn -= 1;
            let percent = w[ch].skills[idx].learned + int_learn(w, ch).min(12);
            w[ch].skills[idx].learned = percent.min(cap);
            if w[ch].skills[idx].learned >= cap {
                send_to_char(w, "You are now learned in this area.\n\r", ch);
            }
        }
        _ => {}
    }
    true
}

fn vanish_room_contents(w: &mut World, ch: CharId, award: bool) -> i32 {
    let room = w[ch].in_room;
    let mut value = 0;
    while let Some(&k) = w.room(room).contents.first() {
        let buf = format!("The {} vanish in a puff of smoke.\n\r", fname(&w[k].name));
        for t in w.room(room).people.clone() {
            if w.can_see_obj(t, k) {
                send_to_char(w, &buf, t);
            }
        }
        if award {
            value += (w[k].obj_flags.cost / 10).min(50).max(1);
        }
        extract_obj(w, k);
    }
    value
}

/// The city dump: everything dropped here vanishes, for a small reward.
pub fn dump(w: &mut World, ch: CharId, cmd: i32, arg: &str) -> bool {
    vanish_room_contents(w, ch, false);
    if cmd != 60 {
        return false;
    }
    do_drop(w, ch, arg, cmd);
    let value = vanish_room_contents(w, ch, true);
    if value != 0 {
        act(w, "You are awarded for outstanding performance.", false, ch, None, Vict::None, TO_CHAR);
        act(w, "$n has been awarded for being a good citizen.", true, ch, None, Vict::None, TO_ROOM);
        if w.level(ch) < 3 {
            gain_exp(w, ch, value);
        } else {
            w[ch].points.gold += value;
        }
    }
    true
}

thread_local! {
    /// (moving?, which path: 0 = open / 1 = close, index into the path)
    static MAYOR: Cell<(bool, u8, usize)> = const { Cell::new((false, 0, 0)) };
}

pub fn mayor(w: &mut World, ch: CharId, cmd: i32, _arg: &str) -> bool {
    const OPEN_PATH: &[u8] = b"W3a3003b33000c111d0d111Oe333333Oe22c222112212111a1S.";
    const CLOSE_PATH: &[u8] = b"W3a3003b33000c111d0d111CE333333CE22c222112212111a1S.";
    let (mut moving, mut which, mut index) = MAYOR.with(|m| m.get());
    if !moving {
        if w.time_info.hours == 6 {
            moving = true;
            which = 0;
            index = 0;
        } else if w.time_info.hours == 20 {
            moving = true;
            which = 1;
            index = 0;
        }
    }
    MAYOR.with(|m| m.set((moving, which, index)));
    if cmd != 0 || !moving || w.get_pos(ch) < POSITION_SLEEPING || w.get_pos(ch) == POSITION_FIGHTING {
        return false;
    }
    let path = if which == 0 { OPEN_PATH } else { CLOSE_PATH };
    let step = path.get(index).copied().unwrap_or(b'.');
    // advance first: the actions below may run other code that calls us
    MAYOR.with(|m| m.set((step != b'.', which, index + 1)));
    let say = |w: &mut World, s: &str| act(w, s, false, ch, None, Vict::None, TO_ROOM);
    match step {
        b'0'..=b'3' => do_move(w, ch, "", (step - b'0') as i32 + 1),
        b'W' => {
            w[ch].specials.position = POSITION_STANDING;
            say(w, "$n awakens and groans loudly.");
        }
        b'S' => {
            w[ch].specials.position = POSITION_SLEEPING;
            say(w, "$n lies down and instantly falls asleep.");
        }
        b'a' => {
            say(w, "$n says 'Hello Honey!'");
            say(w, "$n smirks.");
        }
        b'b' => say(w, "$n says 'What a view! I must get something done about that dump!'"),
        b'c' => say(w, "$n says 'Vandals! Youngsters nowadays have no respect for anything!'"),
        b'd' => say(w, "$n says 'Good day, citizens!'"),
        b'e' => say(w, "$n says 'I hereby declare the bazaar open!'"),
        b'E' => say(w, "$n says 'I hereby declare Midgaard closed!'"),
        b'O' => {
            do_unlock(w, ch, "gate", 0);
            do_open(w, ch, "gate", 0);
        }
        b'C' => {
            do_close(w, ch, "gate", 0);
            do_lock(w, ch, "gate", 0);
        }
        _ => {}
    }
    false
}

fn npc_steal(w: &mut World, ch: CharId, victim: CharId) {
    if w.is_npc(victim) || w.level(victim) > 20 {
        return;
    }
    if w.awake(victim) && number(0, w.level(ch)) == 0 {
        act(w, "You discover that $n has $s hands in your wallet.", false, ch, None, Vict::Ch(victim), TO_VICT);
        act(w, "$n tries to steal gold from $N.", true, ch, None, Vict::Ch(victim), TO_NOTVICT);
    } else {
        let gold = (w[victim].points.gold as i64 * number(1, 10) as i64 / 100) as i32;
        if gold > 0 {
            w[ch].points.gold += gold;
            w[victim].points.gold -= gold;
        }
    }
}

pub fn snake(w: &mut World, ch: CharId, cmd: i32, _arg: &str) -> bool {
    if cmd != 0 || w.get_pos(ch) != POSITION_FIGHTING {
        return false;
    }
    if let Some(f) = w[ch].specials.fighting {
        if w.ch_exists(f) && w[f].in_room == w[ch].in_room && number(0, 32 - w.level(ch)) == 0 {
            act(w, "$n bites $N!", true, ch, None, Vict::Ch(f), TO_NOTVICT);
            act(w, "$n bites you!", true, ch, None, Vict::Ch(f), TO_VICT);
            let l = w.level(ch);
            cast_poison(w, l, ch, "", SPELL_TYPE_SPELL, Some(f), None);
            return true;
        }
    }
    false
}

pub fn thief(w: &mut World, ch: CharId, cmd: i32, _arg: &str) -> bool {
    if cmd != 0 || w.get_pos(ch) != POSITION_STANDING {
        return false;
    }
    for cons in w.room(w[ch].in_room).people.clone() {
        if !w.is_npc(cons) && w.level(cons) < 21 && number(1, 5) == 1 {
            npc_steal(w, ch, cons);
        }
    }
    true
}

/// Pick one of the characters fighting `ch` (each with a 1/3 chance).
fn pick_attacker(w: &World, ch: CharId) -> Option<CharId> {
    w.room(w[ch].in_room)
        .people
        .iter()
        .copied()
        .find(|&v| w[v].specials.fighting == Some(ch) && number(0, 2) == 0)
}

fn utter(w: &mut World, ch: CharId, words: &str) {
    let b = format!("$n utters the words '{}'.", words);
    act(w, &b, true, ch, None, Vict::None, TO_ROOM);
}

pub fn magic_user(w: &mut World, ch: CharId, cmd: i32, _arg: &str) -> bool {
    if cmd != 0 || w.get_pos(ch) != POSITION_FIGHTING || w[ch].specials.fighting.is_none() {
        return false;
    }
    let Some(vict) = pick_attacker(w, ch) else { return false };
    let l = w.level(ch);
    let v = Some(vict);
    if Some(vict) != w[ch].specials.fighting && l > 13 && number(0, 7) == 0 {
        utter(w, ch, "dilan oso");
        cast_sleep(w, l, ch, "", SPELL_TYPE_SPELL, v, None);
        return true;
    }
    if l > 12 && number(0, 6) == 0 {
        utter(w, ch, "gharia miwi");
        cast_curse(w, l, ch, "", SPELL_TYPE_SPELL, v, None);
        return true;
    }
    if l > 7 && number(0, 5) == 0 {
        utter(w, ch, "koholian dia");
        cast_blindness(w, l, ch, "", SPELL_TYPE_SPELL, v, None);
        return true;
    }
    if l > 12 && number(0, 8) == 0 && w.is_evil(ch) {
        utter(w, ch, "ib er dranker");
        cast_energy_drain(w, l, ch, "", SPELL_TYPE_SPELL, v, None);
        return true;
    }
    match l {
        1..=4 => {
            utter(w, ch, "hahili duvini");
            cast_magic_missile(w, l, ch, "", SPELL_TYPE_SPELL, v, None);
        }
        5..=8 => {
            utter(w, ch, "grynt oef");
            cast_burning_hands(w, l, ch, "", SPELL_TYPE_SPELL, v, None);
        }
        9 | 10 => {
            utter(w, ch, "sjulk divi");
            cast_lightning_bolt(w, l, ch, "", SPELL_TYPE_SPELL, v, None);
        }
        11..=14 => {
            utter(w, ch, "nasson hof");
            cast_colour_spray(w, l, ch, "", SPELL_TYPE_SPELL, v, None);
        }
        _ => {
            utter(w, ch, "tuborg");
            cast_fireball(w, l, ch, "", SPELL_TYPE_SPELL, v, None);
        }
    }
    true
}

fn breather(w: &mut World, ch: CharId, cmd: i32, msg: &str, f: CastFn) -> bool {
    if cmd != 0 || w.get_pos(ch) != POSITION_FIGHTING || w[ch].specials.fighting.is_none() {
        return false;
    }
    let Some(vict) = pick_attacker(w, ch) else { return false };
    act(w, msg, true, ch, None, Vict::None, TO_ROOM);
    let l = w.level(ch);
    f(w, l, ch, "", SPELL_TYPE_SPELL, Some(vict), None);
    true
}

pub fn bat_red(w: &mut World, ch: CharId, cmd: i32, _arg: &str) -> bool {
    breather(w, ch, cmd, "$n breathes fire.", cast_fire_breath)
}

pub fn bat_white(w: &mut World, ch: CharId, cmd: i32, _arg: &str) -> bool {
    breather(w, ch, cmd, "$n breathes frost.", cast_frost_breath)
}

pub fn bat_black(w: &mut World, ch: CharId, cmd: i32, _arg: &str) -> bool {
    breather(w, ch, cmd, "$n breathes acid.", cast_acid_breath)
}

pub fn bat_blue(w: &mut World, ch: CharId, cmd: i32, _arg: &str) -> bool {
    breather(w, ch, cmd, "$n breathes lightning.", cast_lightning_breath)
}

pub fn bat_green(w: &mut World, ch: CharId, cmd: i32, _arg: &str) -> bool {
    if cmd != 0 || w.get_pos(ch) != POSITION_FIGHTING || w[ch].specials.fighting.is_none() {
        return false;
    }
    if number(0, 3) != 0 {
        return false;
    }
    act(w, "$n breathes gas.", true, ch, None, Vict::None, TO_ROOM);
    let l = w.level(ch);
    cast_gas_breath(w, l, ch, "", SPELL_TYPE_SPELL, None, None);
    true
}

/// The guild guards only let members of their own class pass.
pub fn guild_guard(w: &mut World, ch: CharId, cmd: i32, _arg: &str) -> bool {
    if !(1..=6).contains(&cmd) {
        return false;
    }
    let room = w[ch].in_room;
    let class = w[ch].player.class;
    let blocked = (room == real_room(w, 3017) && cmd == 3 && class != CLASS_MAGIC_USER)
        || (room == real_room(w, 3004) && cmd == 1 && class != CLASS_CLERIC)
        || (room == real_room(w, 3027) && cmd == 2 && class != CLASS_THIEF)
        || (room == real_room(w, 3021) && cmd == 2 && class != CLASS_WARRIOR);
    if blocked {
        act(w, "The guard humiliates $n, and blocks $s way.", false, ch, None, Vict::None, TO_ROOM);
        send_to_char(w, "The guard humiliates you, and block your way.\n\r", ch);
        return true;
    }
    false
}

pub fn puff(w: &mut World, ch: CharId, cmd: i32, _arg: &str) -> bool {
    if cmd != 0 {
        return false;
    }
    let msg = match number(0, 60) {
        0 => "My god! It's full of stars!",
        1 => "How'd all those fish get up here?",
        2 => "I'm a very female dragon.",
        3 => "I've got a peaceful, easy feeling.",
        _ => return false,
    };
    do_say(w, ch, msg, 0);
    true
}

pub fn fido(w: &mut World, ch: CharId, cmd: i32, _arg: &str) -> bool {
    if cmd != 0 || !w.awake(ch) {
        return false;
    }
    let room = w[ch].in_room;
    for i in w.room(room).contents.clone() {
        if w[i].obj_flags.type_flag == ITEM_CONTAINER && w[i].obj_flags.value[3] != 0 {
            act(w, "$n savagely devour a corpse.", false, ch, None, Vict::None, TO_ROOM);
            for t in w[i].contains.clone() {
                obj_from_obj(w, t);
                obj_to_room(w, t, room);
            }
            extract_obj(w, i);
            return true;
        }
    }
    false
}

pub fn janitor(w: &mut World, ch: CharId, cmd: i32, _arg: &str) -> bool {
    if cmd != 0 || !w.awake(ch) {
        return false;
    }
    for i in w.room(w[ch].in_room).contents.clone() {
        let f = &w[i].obj_flags;
        if f.wear_flags & ITEM_TAKE != 0 && (f.type_flag == ITEM_DRINKCON || f.cost <= 10) {
            act(w, "$n picks up some trash.", false, ch, None, Vict::None, TO_ROOM);
            obj_from_room(w, i);
            obj_to_char(w, i, ch);
            return true;
        }
    }
    false
}

pub fn cityguard(w: &mut World, ch: CharId, cmd: i32, _arg: &str) -> bool {
    if cmd != 0 || !w.awake(ch) || w.get_pos(ch) == POSITION_FIGHTING {
        return false;
    }
    let mut max_evil = 300;
    let mut evil = None;
    for t in w.room(w[ch].in_room).people.clone() {
        if let Some(f) = w[t].specials.fighting {
            if w.ch_exists(f) && w[t].specials.alignment < max_evil && (w.is_npc(t) || w.is_npc(f)) {
                max_evil = w[t].specials.alignment;
                evil = Some(t);
            }
        }
    }
    if let Some(e) = evil {
        if let Some(f) = w[e].specials.fighting {
            if !w.is_evil(f) {
                act(
                    w,
                    "$n screams 'PROTECT THE INNOCENT!  BANZAI!!! CHARGE!!! ARARARAGGGHH!'",
                    false,
                    ch,
                    None,
                    Vict::None,
                    TO_ROOM,
                );
                hit(w, ch, e, TYPE_UNDEFINED);
                return true;
            }
        }
    }
    false
}

pub fn pet_shops(w: &mut World, ch: CharId, cmd: i32, arg: &str) -> bool {
    let pet_room = w[ch].in_room + 1;
    if pet_room as usize >= w.world.len() {
        return false;
    }
    if cmd == 59 {
        send_to_char(w, "Available pets are:\n\r", ch);
        for pet in w.room(pet_room).people.clone() {
            let b = format!("{:8} - {}\n\r", 3 * w[pet].points.exp, w[pet].player.short_descr);
            send_to_char(w, &b, ch);
        }
        return true;
    }
    if cmd != 56 {
        return false;
    }
    let (buf, rest) = one_argument(arg);
    let (pet_name, _) = one_argument(rest);
    let Some(pet) = get_char_room(w, &buf, pet_room) else {
        send_to_char(w, "There is no such pet!\n\r", ch);
        return true;
    };
    let price = w[pet].points.exp * 3;
    if w[ch].points.gold < price {
        send_to_char(w, "You don't have enough gold!\n\r", ch);
        return true;
    }
    w[ch].points.gold -= price;
    let nr = w[pet].nr;
    let Some(pet) = read_mobile(w, nr, REAL) else { return true };
    w[pet].points.exp = 0;
    w[pet].specials.affected_by |= AFF_CHARM;
    if !pet_name.is_empty() {
        let n = format!("{} {}", w[pet].player.name, pet_name);
        w[pet].player.name = n;
        let d = format!(
            "{}A small sign on a chain around the neck says 'My Name is {}'\n\r",
            w[pet].player.description, pet_name
        );
        w[pet].player.description = d;
    }
    let r = w[ch].in_room;
    char_to_room(w, pet, r);
    add_follower(w, pet, ch);
    w[pet].specials.carry_weight = 1000;
    w[pet].specials.carry_items = 100;
    send_to_char(w, "May you enjoy your pet.\n\r", ch);
    act(w, "$n bought $N as a pet.", false, ch, None, Vict::Ch(pet), TO_ROOM);
    true
}

/// Odin's altar: pray to get back items left "for" you in the next room.
pub fn pray_for_items(w: &mut World, ch: CharId, cmd: i32, _arg: &str) -> bool {
    if cmd != 176 {
        return false;
    }
    let key_room = w[ch].in_room + 1;
    if key_room as usize >= w.world.len() {
        return false;
    }
    let buf = format!("item_for_{}", w[ch].player.name);
    let mut gold = 0;
    let mut found = false;
    for t in w.room(key_room).contents.clone() {
        let matches = w[t].ex_description.iter().filter(|e| str_cmp(&buf, &e.keyword) == 0).count();
        for _ in 0..matches {
            if gold == 0 {
                gold = 1;
                act(w, "$n kneels and at the altar and chants a prayer to Odin.", false, ch, None, Vict::None, TO_ROOM);
                act(w, "You notice a faint light in Odin's eye.", false, ch, None, Vict::None, TO_CHAR);
            }
            let nr = w[t].item_number;
            if let Some(obj) = read_object(w, nr, REAL) {
                let r = w[ch].in_room;
                obj_to_room(w, obj, r);
                act(w, "$p slowly fades into existence.", false, ch, Some(obj), Vict::None, TO_ROOM);
                act(w, "$p slowly fades into existence.", false, ch, Some(obj), Vict::None, TO_CHAR);
                gold += w[obj].obj_flags.cost;
                found = true;
            }
        }
    }
    if found {
        w[ch].points.gold = (w[ch].points.gold - gold).max(0);
        return true;
    }
    false
}

/// The worm ritual: recite the right scroll with herbs and blood present.
pub fn worm_ritual(w: &mut World, ch: CharId, cmd: i32, arg: &str) -> bool {
    if cmd != 207 {
        return false;
    }
    let scroll_nr = real_object(w, 5012);
    let herbs_nr = real_object(w, 5002);
    let blood_nr = real_object(w, 5003);
    let to_room = real_room(w, 5040);
    let (buf, _) = one_argument(arg);
    let (scroll, equipped) = match get_obj_in_list_vis(w, ch, &buf, &w[ch].carrying) {
        Some(s) => (Some(s), false),
        None => (w[ch].equipment[HOLD], true),
    };
    let Some(scroll) = scroll.filter(|&s| w[s].item_number == scroll_nr) else { return false };
    act(w, "$n recites $p.", true, ch, Some(scroll), Vict::None, TO_ROOM);
    act(w, "You recite $p which dissolves.", false, ch, Some(scroll), Vict::None, TO_CHAR);
    let room = w[ch].in_room;
    let blood = get_obj_in_list_num(w, blood_nr, &w.room(room).contents);
    let herbs = get_obj_in_list_num(w, herbs_nr, &w.room(room).contents);
    let (Some(blood), Some(herbs)) = (blood, herbs) else { return true };
    if w[blood].obj_flags.value[1] < 2 || to_room < 0 {
        return true;
    }
    act(w, "$p dissolves into thin air.", true, ch, Some(herbs), Vict::None, TO_ROOM);
    act(w, "$p dissolves into thin air.", true, ch, Some(herbs), Vict::None, TO_CHAR);
    act(w, "$p is emptied from blood.", false, ch, Some(blood), Vict::None, TO_ROOM);
    act(w, "$p is emptied from blood.", false, ch, Some(blood), Vict::None, TO_CHAR);
    obj_from_room(w, herbs);
    extract_obj(w, herbs);
    w[blood].obj_flags.value[1] = 0;
    w[blood].obj_flags.weight -= 2;
    if equipped {
        unequip_char(w, ch, HOLD);
    }
    extract_obj(w, scroll);
    act(w, "You feel yanked downwards.....", false, ch, None, Vict::None, TO_ROOM);
    act(w, "You feel yanked downwards.....", false, ch, None, Vict::None, TO_CHAR);
    for p in w.room(room).people.clone() {
        char_from_room(w, p);
        char_to_room(w, p, to_room);
    }
    let _ = do_look; // (the original did not show the new room)
    true
}
