//! Talking (act.comm.c).

use crate::comm::*;
use crate::game::World;
use crate::handler::*;
use crate::interpreter::{argument_interpreter, half_chop};
use crate::structs::*;

pub fn do_say(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let a = argument.trim_start_matches(' ');
    if a.is_empty() {
        send_to_char(w, "Yes, but WHAT do you want to say?\n\r", ch);
    } else {
        let buf = format!("$n says '{}'", a);
        act(w, &buf, false, ch, None, Vict::None, TO_ROOM);
        send_to_char(w, "Ok.\n\r", ch);
    }
}

pub fn do_shout(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w[ch].specials.act & PLR_NOSHOUT != 0 {
        send_to_char(w, "You can't shout!!\n\r", ch);
        return;
    }
    if w[ch].points.mov < 10 {
        send_to_char(w, "You are too exhausted.\n\r", ch);
        return;
    }
    w[ch].points.mov -= 10;
    let a = argument.trim_start_matches(' ');
    if a.is_empty() {
        send_to_char(w, "Shout? Yes! Fine! Shout we must, but WHAT??\n\r", ch);
        return;
    }
    send_to_char(w, "Ok.\n\r", ch);
    let buf = format!("$n shouts '{}'", a);
    let targets: Vec<CharId> = w
        .descriptors
        .iter()
        .filter(|d| d.connected == CON_PLYNG)
        .filter_map(|d| d.character)
        .filter(|&c| c != ch && w.ch_exists(c) && w[c].specials.act & PLR_NOSHOUT == 0)
        .collect();
    for t in targets {
        act(w, &buf, false, ch, None, Vict::Ch(t), TO_VICT);
    }
}

pub fn do_tell(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w[ch].specials.act & PLR_NOTELL != 0 {
        send_to_char(w, "Your message didn't get through!!\n\r", ch);
        return;
    }
    let (name, message) = half_chop(argument);
    if name.is_empty() || message.is_empty() {
        send_to_char(w, "Who do you wish to tell what??\n\r", ch);
        return;
    }
    let Some(vict) = get_char_vis(w, ch, &name) else {
        send_to_char(w, "No-one by that name here..\n\r", ch);
        return;
    };
    if ch == vict {
        send_to_char(w, "You try to tell yourself something.\n\r", ch);
    } else if w.get_pos(vict) == POSITION_SLEEPING || w[vict].specials.act & PLR_NOTELL != 0 {
        act(w, "$E can't hear you.", false, ch, None, Vict::Ch(vict), TO_CHAR);
    } else {
        let who = if w.is_npc(ch) { w[ch].player.short_descr.clone() } else { w[ch].player.name.clone() };
        let buf = format!("{} tells you '{}'\n\r", who, message);
        send_to_char(w, &buf, vict);
        send_to_char(w, "Ok.\n\r", ch);
    }
}

pub fn do_whisper(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (name, message) = half_chop(argument);
    if name.is_empty() || message.is_empty() {
        send_to_char(w, "Who do you want to whisper to.. and what??\n\r", ch);
        return;
    }
    let Some(vict) = get_char_room_vis(w, ch, &name) else {
        send_to_char(w, "No-one by that name here..\n\r", ch);
        return;
    };
    if vict == ch {
        act(w, "$n whispers quietly to $mself.", false, ch, None, Vict::None, TO_ROOM);
        send_to_char(w, "You can't seem to get your mouth close enough to your ear...\n\r", ch);
    } else {
        let buf = format!("$n whispers to you, '{}'", message);
        act(w, &buf, false, ch, None, Vict::Ch(vict), TO_VICT);
        send_to_char(w, "Ok.\n\r", ch);
        act(w, "$n whispers something to $N.", false, ch, None, Vict::Ch(vict), TO_NOTVICT);
    }
}

pub fn do_ask(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (name, message) = half_chop(argument);
    if name.is_empty() || message.is_empty() {
        send_to_char(w, "Who do you want to ask something.. and what??\n\r", ch);
        return;
    }
    let Some(vict) = get_char_room_vis(w, ch, &name) else {
        send_to_char(w, "No-one by that name here..\n\r", ch);
        return;
    };
    if vict == ch {
        act(w, "$n quietly asks $mself a question.", false, ch, None, Vict::None, TO_ROOM);
        send_to_char(w, "You think about it for a while...\n\r", ch);
    } else {
        let buf = format!("$n asks you '{}'", message);
        act(w, &buf, false, ch, None, Vict::Ch(vict), TO_VICT);
        send_to_char(w, "Ok.\n\r", ch);
        act(w, "$n asks $N a question.", false, ch, None, Vict::Ch(vict), TO_NOTVICT);
    }
}

const MAX_NOTE_LENGTH: usize = 1000;

pub fn do_write(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (papername, penname) = argument_interpreter(argument);
    let Some(did) = w[ch].desc else { return };
    if papername.is_empty() {
        send_to_char(w, "Write? with what? ON what? what are you trying to do??\n\r", ch);
        return;
    }
    let paper;
    let pen;
    if !penname.is_empty() {
        let Some(p) = get_obj_in_list_vis(w, ch, &papername, &w[ch].carrying) else {
            send_to_char(w, &format!("You have no {}.\n\r", papername), ch);
            return;
        };
        let Some(q) = get_obj_in_list_vis(w, ch, &penname, &w[ch].carrying) else {
            send_to_char(w, &format!("You have no {}.\n\r", papername), ch);
            return;
        };
        paper = p;
        pen = q;
    } else {
        let Some(p) = get_obj_in_list_vis(w, ch, &papername, &w[ch].carrying) else {
            send_to_char(w, &format!("There is no {} in your inventory.\n\r", papername), ch);
            return;
        };
        let found_pen = w[p].obj_flags.type_flag == ITEM_PEN;
        if !found_pen && w[p].obj_flags.type_flag != ITEM_NOTE {
            send_to_char(w, "That thing has nothing to do with writing.\n\r", ch);
            return;
        }
        let Some(held) = w[ch].equipment[HOLD] else {
            send_to_char(w, &format!("You can't write with a {} alone.\n\r", papername), ch);
            return;
        };
        if !w.can_see_obj(ch, held) {
            send_to_char(w, "The stuff in your hand is invisible! Yeech!!\n\r", ch);
            return;
        }
        if found_pen {
            pen = p;
            paper = held;
        } else {
            paper = p;
            pen = held;
        }
    }
    if w[pen].obj_flags.type_flag != ITEM_PEN {
        act(w, "$p is no good for writing with.", false, ch, Some(pen), Vict::None, TO_CHAR);
    } else if w[paper].obj_flags.type_flag != ITEM_NOTE {
        act(w, "You can't write on $p.", false, ch, Some(paper), Vict::None, TO_CHAR);
    } else if !w[paper].action_description.is_empty() {
        send_to_char(w, "There's something written on it already.\n\r", ch);
    } else {
        send_to_char(w, "Ok.. go ahead and write.. end the note with a @.\n\r", ch);
        act(w, "$n begins to jot down a note.", true, ch, None, Vict::None, TO_ROOM);
        if let Some(d) = w.desc_mut(did) {
            d.str_target = Some(StrTarget::Note(paper));
            d.str_buf.clear();
            d.max_str = MAX_NOTE_LENGTH;
        }
    }
}
