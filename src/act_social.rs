//! Social commands and poses (act.social.c).

use crate::comm::*;
use crate::db::{Reader, POSEMESS_FILE, SOCMESS_FILE};
use crate::game::{PoseType, SocialMessg, World};
use crate::handler::*;
use crate::interpreter::one_argument;
use crate::structs::*;
use crate::utils::*;

/// fread_action(): one line; a line starting with '#' means "none".
fn fread_action(f: &mut Reader) -> String {
    match f.line() {
        Some(l) if l.starts_with('#') => String::new(),
        Some(l) => l,
        None => {
            log("Fread_action - unexpected EOF.");
            String::new()
        }
    }
}

pub fn boot_social_messages(w: &mut World) {
    let mut f = match Reader::open(SOCMESS_FILE) {
        Ok(f) => f,
        Err(e) => {
            log(&format!("boot_social_messages: {}", e));
            std::process::exit(1);
        }
    };
    let mut min_pos = 0;
    loop {
        let tmp = f.int();
        if tmp < 0 || f.eof() {
            break;
        }
        let hide = f.int();
        // one entry in the original data lacks this number; fscanf then
        // left the previous value in place, and so do we
        if let Some(p) = f.try_int() {
            min_pos = p;
        }
        f.skip_ws();
        let mut m = SocialMessg { act_nr: tmp, hide: hide != 0, min_victim_position: min_pos, ..Default::default() };
        m.char_no_arg = fread_action(&mut f);
        m.others_no_arg = fread_action(&mut f);
        m.char_found = fread_action(&mut f);
        if !m.char_found.is_empty() {
            m.others_found = fread_action(&mut f);
            m.vict_found = fread_action(&mut f);
            m.not_found = fread_action(&mut f);
            m.char_auto = fread_action(&mut f);
            m.others_auto = fread_action(&mut f);
        }
        w.soc_mess_list.push(m);
    }
}

pub fn find_action(w: &World, cmd: i32) -> Option<usize> {
    w.soc_mess_list.iter().position(|m| m.act_nr == cmd)
}

pub fn do_action(w: &mut World, ch: CharId, argument: &str, cmd: i32) {
    let Some(n) = find_action(w, cmd) else {
        send_to_char(w, "That action is not supported.\n\r", ch);
        return;
    };
    let action = w.soc_mess_list[n].clone();
    let buf = if !action.char_found.is_empty() { one_argument(argument).0 } else { String::new() };
    if buf.is_empty() {
        send_to_char(w, &action.char_no_arg, ch);
        send_to_char(w, "\n\r", ch);
        act(w, &action.others_no_arg, action.hide, ch, None, Vict::None, TO_ROOM);
        return;
    }
    match get_char_room_vis(w, ch, &buf) {
        None => {
            send_to_char(w, &action.not_found, ch);
            send_to_char(w, "\n\r", ch);
        }
        Some(vict) if vict == ch => {
            send_to_char(w, &action.char_auto, ch);
            send_to_char(w, "\n\r", ch);
            act(w, &action.others_auto, action.hide, ch, None, Vict::None, TO_ROOM);
        }
        Some(vict) => {
            if w.get_pos(vict) < action.min_victim_position {
                act(w, "$N is not in a proper position for that.", false, ch, None, Vict::Ch(vict), TO_CHAR);
            } else {
                act(w, &action.char_found, false, ch, None, Vict::Ch(vict), TO_CHAR);
                act(w, &action.others_found, action.hide, ch, None, Vict::Ch(vict), TO_NOTVICT);
                act(w, &action.vict_found, action.hide, ch, None, Vict::Ch(vict), TO_VICT);
            }
        }
    }
}

pub fn do_insult(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (arg, _) = one_argument(argument);
    if arg.is_empty() {
        send_to_char(w, "Sure you don't want to insult everybody.\n\r", ch);
        return;
    }
    let Some(victim) = get_char_room_vis(w, ch, &arg) else {
        send_to_char(w, "Can't hear you!\n\r", ch);
        return;
    };
    if victim == ch {
        send_to_char(w, "You feel insulted.\n\r", ch);
        return;
    }
    let buf = format!("You insult {}.\n\r", w[victim].player.name);
    send_to_char(w, &buf, ch);
    let msg = match random() % 3 {
        0 => {
            if w[ch].player.sex == SEX_MALE {
                if w[victim].player.sex == SEX_MALE {
                    "$n accuses you of fighting like a woman!"
                } else {
                    "$n says that women can't fight."
                }
            } else if w[victim].player.sex == SEX_MALE {
                "$n accuses you of having the smallest.... (brain?)"
            } else {
                "$n tells you that you'd loose a beautycontest against a troll."
            }
        }
        1 => "$n calls your mother a bitch!",
        _ => "$n tells you to get lost!",
    };
    act(w, msg, false, ch, None, Vict::Ch(victim), TO_VICT);
    act(w, "$n insults $N.", true, ch, None, Vict::Ch(victim), TO_NOTVICT);
}

pub fn boot_pose_messages(w: &mut World) {
    let mut f = match Reader::open(POSEMESS_FILE) {
        Ok(f) => f,
        Err(e) => {
            log(&format!("boot_pose_messages: {}", e));
            std::process::exit(1);
        }
    };
    loop {
        let level = f.int();
        if level < 0 || f.eof() {
            break;
        }
        f.skip_ws();
        let mut p = PoseType { level, ..Default::default() };
        for class in 0..4 {
            p.poser_msg[class] = fread_action(&mut f);
            p.room_msg[class] = fread_action(&mut f);
        }
        w.pose_messages.push(p);
    }
}

pub fn do_pose(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    if w.pose_messages.is_empty() || w.level(ch) < w.pose_messages[0].level || w.is_npc(ch) {
        send_to_char(w, "You can't do that.\n\r", ch);
        return;
    }
    let lvl = w.level(ch);
    let counter = w.pose_messages.iter().take_while(|p| p.level <= lvl && p.level >= 0).count() as i32 - 1;
    let to_pose = number(0, counter.max(0)) as usize;
    let cls = (w[ch].player.class - 1).clamp(0, 3) as usize;
    let p = w.pose_messages[to_pose].clone();
    act(w, &p.poser_msg[cls], false, ch, None, Vict::None, TO_CHAR);
    act(w, &p.room_msg[cls], false, ch, None, Vict::None, TO_ROOM);
}
