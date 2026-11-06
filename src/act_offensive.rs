//! Offensive commands (act.offensive.c).

use crate::act_move::do_simple_move;
use crate::comm::*;
use crate::fight::*;
use crate::game::World;
use crate::handler::*;
use crate::interpreter::{command_interpreter, half_chop, one_argument};
use crate::limits::{gain_exp, hit_limit};
use crate::spell_parser::{saves_spell, stop_follower};
use crate::spells::*;
use crate::structs::*;
use crate::utils::*;

pub fn do_hit(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (arg, _) = one_argument(argument);
    if arg.is_empty() {
        send_to_char(w, "Hit who?\n\r", ch);
        return;
    }
    let Some(victim) = get_char_room_vis(w, ch, &arg) else {
        send_to_char(w, "They aren't here.\n\r", ch);
        return;
    };
    if victim == ch {
        send_to_char(w, "You hit yourself..OUCH!.\n\r", ch);
        act(w, "$n hits $mself, and says OUCH!", false, ch, None, Vict::Ch(victim), TO_ROOM);
        return;
    }
    if w.is_affected(ch, AFF_CHARM) && w[ch].master == Some(victim) {
        act(w, "$N is just such a good friend, you simply can't hit $M.", false, ch, None, Vict::Ch(victim), TO_CHAR);
        return;
    }
    if w.get_pos(ch) == POSITION_STANDING && Some(victim) != w[ch].specials.fighting {
        hit(w, ch, victim, TYPE_UNDEFINED);
        if w.ch_exists(ch) {
            w.wait_state(ch, PULSE_VIOLENCE as i32 + 2);
        }
    } else {
        send_to_char(w, "You do the best you can!\n\r", ch);
    }
}

pub fn do_kill(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.level(ch) < 24 || w.is_npc(ch) {
        do_hit(w, ch, argument, 0);
        return;
    }
    let (arg, _) = one_argument(argument);
    if arg.is_empty() {
        send_to_char(w, "Kill who?\n\r", ch);
        return;
    }
    match get_char_room_vis(w, ch, &arg) {
        None => send_to_char(w, "They aren't here.\n\r", ch),
        Some(v) if v == ch => send_to_char(w, "Your mother would be so sad.. :(\n\r", ch),
        Some(v) => {
            act(w, "You chop $M to pieces! Ah! The blood!", false, ch, None, Vict::Ch(v), TO_CHAR);
            act(w, "$N chops you to pieces!", false, v, None, Vict::Ch(ch), TO_CHAR);
            act(w, "$n brutally slays $N", false, ch, None, Vict::Ch(v), TO_NOTVICT);
            raw_kill(w, v);
        }
    }
}

pub fn do_backstab(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (name, _) = one_argument(argument);
    let Some(victim) = get_char_room_vis(w, ch, &name) else {
        send_to_char(w, "Backstab who?\n\r", ch);
        return;
    };
    if victim == ch {
        send_to_char(w, "How can you sneak up on yourself?\n\r", ch);
        return;
    }
    let Some(wield) = w[ch].equipment[WIELD] else {
        send_to_char(w, "You need to wield a weapon, to make it a succes.\n\r", ch);
        return;
    };
    if w[wield].obj_flags.value[3] != 11 {
        send_to_char(w, "Only piercing weapons can be used for backstabbing.\n\r", ch);
        return;
    }
    if w[victim].specials.fighting.is_some() {
        send_to_char(w, "You can't backstab a fighting person, too alert!\n\r", ch);
        return;
    }
    let percent = number(1, 101);
    if w.awake(victim) && percent > w[ch].skills[SKILL_BACKSTAB as usize].learned {
        damage(w, ch, victim, 0, SKILL_BACKSTAB);
    } else {
        hit(w, ch, victim, SKILL_BACKSTAB);
    }
}

/// The "4 saving throws" check used when ordering charmed followers.
fn order_breaks_charm(w: &mut World, ch: CharId, victim: CharId) -> bool {
    if saves_spell(w, victim, SAVING_SPELL) {
        if saves_spell(w, victim, SAVING_SPELL) && saves_spell(w, victim, SAVING_SPELL) && saves_spell(w, victim, SAVING_SPELL)
        {
            stop_follower(w, victim);
            if w.is_npc(victim) {
                hit(w, victim, ch, TYPE_UNDEFINED);
            }
            return true;
        } else {
            act(w, "$n has an indifferent look.", false, victim, None, Vict::None, TO_ROOM);
        }
    }
    false
}

pub fn do_order(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (name, message) = half_chop(argument);
    if name.is_empty() || message.is_empty() {
        send_to_char(w, "Order who to do what?\n\r", ch);
        return;
    }
    let victim = get_char_room_vis(w, ch, &name);
    if victim.is_none() && str_cmp("follower", &name) != 0 && str_cmp("followers", &name) != 0 {
        send_to_char(w, "That person isn't here.\n\r", ch);
        return;
    }
    if victim == Some(ch) {
        send_to_char(w, "You obviously suffer from skitzofrenia.\n\r", ch);
        return;
    }
    if w.is_affected(ch, AFF_CHARM) {
        send_to_char(w, "Your superior would not aprove of you giving orders.\n\r", ch);
        return;
    }
    if let Some(victim) = victim {
        let buf = format!("$N orders you to '{}'", message);
        act(w, &buf, false, victim, None, Vict::Ch(ch), TO_CHAR);
        act(w, "$n gives $N an order.", false, ch, None, Vict::Ch(victim), TO_ROOM);
        if w[victim].master != Some(ch) || !w.is_affected(victim, AFF_CHARM) {
            act(w, "$n has an indifferent look.", false, victim, None, Vict::None, TO_ROOM);
        } else {
            order_breaks_charm(w, ch, victim);
            send_to_char(w, "Ok.\n\r", ch);
            if w.ch_exists(victim) {
                command_interpreter(w, victim, &message);
            }
        }
    } else {
        let buf = format!("$n issues the order '{}'.", message);
        act(w, &buf, false, ch, None, Vict::None, TO_ROOM);
        let org_room = w[ch].in_room;
        let mut found = false;
        for k in w[ch].followers.clone() {
            if !w.ch_exists(k) || !w.ch_exists(ch) {
                continue;
            }
            if org_room == w[k].in_room && w.is_affected(k, AFF_CHARM) {
                found = true;
                // (the C code tested the NULL victim here; the follower is meant)
                order_breaks_charm(w, ch, k);
                if w.ch_exists(k) {
                    command_interpreter(w, k, &message);
                }
            }
        }
        if !w.ch_exists(ch) {
            return;
        }
        if found {
            send_to_char(w, "Ok.\n\r", ch);
        } else {
            send_to_char(w, "Nobody here are loyal subjects of yours!\n\r", ch);
        }
    }
}

pub fn do_flee(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    let fighting = w[ch].specials.fighting;
    for _ in 0..6 {
        let attempt = number(0, 5) as usize;
        if !w.can_go(ch, attempt) {
            continue;
        }
        let to = w.exit(ch, attempt).unwrap().to_room;
        if w.room(to).room_flags & DEATH != 0 {
            continue;
        }
        act(w, "$n panics, and attempts to flee.", true, ch, None, Vict::None, TO_ROOM);
        let die = do_simple_move(w, ch, attempt, false);
        if die == 1 {
            if let Some(f) = fighting {
                if w.ch_exists(f) {
                    let loose = (hit_limit(w, f) - w[f].points.hit) * w.level(f);
                    if !w.is_npc(ch) {
                        gain_exp(w, ch, -loose);
                    }
                }
            }
            send_to_char(w, "You flee head over heels.\n\r", ch);
            if let Some(f) = fighting {
                if w.ch_exists(f) && w[f].specials.fighting == Some(ch) {
                    stop_fighting(w, f);
                }
                if w[ch].specials.fighting.is_some() {
                    stop_fighting(w, ch);
                }
            }
        } else if die == 0 {
            act(w, "$n tries to flee, but is too exhausted!", true, ch, None, Vict::None, TO_ROOM);
        }
        return;
    }
    send_to_char(w, "PANIC! You couldn't escape!\n\r", ch);
}

fn target_or_opponent(w: &mut World, ch: CharId, name: &str, who: &str) -> Option<CharId> {
    if let Some(v) = get_char_room_vis(w, ch, name) {
        return Some(v);
    }
    if let Some(f) = w[ch].specials.fighting {
        return Some(f);
    }
    send_to_char(w, who, ch);
    None
}

pub fn do_bash(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (name, _) = one_argument(argument);
    if w[ch].player.class != CLASS_WARRIOR {
        send_to_char(w, "You better leave all the martial arts to fighters.\n\r", ch);
        return;
    }
    let Some(victim) = target_or_opponent(w, ch, &name, "Bash who?\n\r") else { return };
    if victim == ch {
        send_to_char(w, "Aren't we funny today...\n\r", ch);
        return;
    }
    if w[ch].equipment[WIELD].is_none() {
        send_to_char(w, "You need to wield a weapon, to make it a success.\n\r", ch);
        return;
    }
    let percent = number(1, 101);
    if percent > w[ch].skills[SKILL_BASH as usize].learned {
        damage(w, ch, victim, 0, SKILL_BASH);
        if w.ch_exists(ch) {
            w[ch].specials.position = POSITION_SITTING;
        }
    } else {
        damage(w, ch, victim, 1, SKILL_BASH);
        if w.ch_exists(victim) {
            w[victim].specials.position = POSITION_SITTING;
            w.wait_state(victim, PULSE_VIOLENCE as i32 * 2);
        }
    }
    if w.ch_exists(ch) {
        w.wait_state(ch, PULSE_VIOLENCE as i32 * 2);
    }
}

pub fn do_rescue(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (victim_name, _) = one_argument(argument);
    let Some(victim) = get_char_room_vis(w, ch, &victim_name) else {
        send_to_char(w, "Who do you want to rescue?\n\r", ch);
        return;
    };
    if victim == ch {
        send_to_char(w, "What about fleeing instead?\n\r", ch);
        return;
    }
    if w[ch].specials.fighting == Some(victim) {
        send_to_char(w, "How can you rescue someone you are trying to kill?\n\r", ch);
        return;
    }
    let tmp_ch = w.room(w[ch].in_room).people.iter().copied().find(|&t| w[t].specials.fighting == Some(victim));
    let Some(tmp_ch) = tmp_ch else {
        act(w, "But nobody is fighting $M?", false, ch, None, Vict::Ch(victim), TO_CHAR);
        return;
    };
    if w[ch].player.class != CLASS_WARRIOR {
        send_to_char(w, "But only true warriors can do this!", ch);
        return;
    }
    let percent = number(1, 101);
    if percent > w[ch].skills[SKILL_RESCUE as usize].learned {
        send_to_char(w, "You fail the rescue.\n\r", ch);
        return;
    }
    send_to_char(w, "Banzai! To the rescue...\n\r", ch);
    act(w, "You are rescued by $N, you are confused!", false, victim, None, Vict::Ch(ch), TO_CHAR);
    act(w, "$n heroically rescues $N.", false, ch, None, Vict::Ch(victim), TO_NOTVICT);
    if w[victim].specials.fighting == Some(tmp_ch) {
        stop_fighting(w, victim);
    }
    if w[tmp_ch].specials.fighting.is_some() {
        stop_fighting(w, tmp_ch);
    }
    if w[ch].specials.fighting.is_some() {
        stop_fighting(w, ch);
    }
    set_fighting(w, ch, tmp_ch);
    set_fighting(w, tmp_ch, ch);
    w.wait_state(victim, 2 * PULSE_VIOLENCE as i32);
}

pub fn do_kick(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w[ch].player.class != CLASS_WARRIOR {
        send_to_char(w, "You better leave all the martial arts to fighters.\n\r", ch);
        return;
    }
    let (name, _) = one_argument(argument);
    let Some(victim) = target_or_opponent(w, ch, &name, "Kick who?\n\r") else { return };
    if victim == ch {
        send_to_char(w, "Aren't we funny today...\n\r", ch);
        return;
    }
    let percent = ((10 - (w[victim].points.armor / 10)) << 1) + number(1, 101);
    if percent > w[ch].skills[SKILL_KICK as usize].learned {
        damage(w, ch, victim, 0, SKILL_KICK);
    } else {
        let d = w.level(ch) >> 1;
        damage(w, ch, victim, d, SKILL_KICK);
    }
    if w.ch_exists(ch) {
        w.wait_state(ch, PULSE_VIOLENCE as i32 * 3);
    }
}
