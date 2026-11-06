//! Combat system and messages (fight.c).

use crate::act_offensive::do_flee;
use crate::comm::*;
use crate::constants::*;
use crate::db::{Reader, MESS_FILE};
use crate::game::World;
use crate::handler::*;
use crate::limits::{gain_exp, hit_limit};
use crate::spell_parser::stop_follower;
use crate::spells::*;
use crate::structs::*;
use crate::utils::*;

pub const ATTACK_HIT_TEXT: [(&str, &str); 9] = [
    ("hit", "hits"),
    ("pound", "pounds"),
    ("pierce", "pierces"),
    ("slash", "slashes"),
    ("whip", "whips"),
    ("claw", "claws"),
    ("bite", "bites"),
    ("sting", "stings"),
    ("crush", "crushes"),
];

pub fn appear(w: &mut World, ch: CharId) {
    act(w, "$n slowly fade into existence.", false, ch, None, Vict::None, TO_ROOM);
    if affected_by_spell(w, ch, SPELL_INVISIBLE) {
        affect_from_char(w, ch, SPELL_INVISIBLE);
    }
    w[ch].specials.affected_by &= !AFF_INVISIBLE;
}

pub fn load_messages(w: &mut World) {
    let mut f = match Reader::open(MESS_FILE) {
        Ok(f) => f,
        Err(e) => {
            log(&format!("read messages: {}", e));
            std::process::exit(1);
        }
    };
    let mut chk = f.word();
    while chk.starts_with('M') {
        let typ = f.int();
        f.line();
        let read = |f: &mut Reader| MsgType {
            attacker_msg: f.string(),
            victim_msg: f.string(),
            room_msg: f.string(),
        };
        let die_msg = read(&mut f);
        let miss_msg = read(&mut f);
        let hit_msg = read(&mut f);
        let god_msg = read(&mut f);
        let m = MessageType { die_msg, miss_msg, hit_msg, sanctuary_msg: MsgType::default(), god_msg };
        match w.fight_messages.iter_mut().find(|l| l.a_type == typ) {
            Some(l) => l.msg.insert(0, m),
            None => {
                if w.fight_messages.len() >= MAX_MESSAGES {
                    log("Too many combat messages.");
                    break;
                }
                w.fight_messages.push(MessageList { a_type: typ, msg: vec![m] });
            }
        }
        chk = f.word();
    }
}

pub fn update_pos(w: &mut World, victim: CharId) {
    let v = &mut w[victim];
    let hit = v.points.hit;
    if hit > 0 && v.specials.position > POSITION_STUNNED {
        return;
    }
    v.specials.position = if hit > 0 {
        POSITION_STANDING
    } else if hit <= -11 {
        POSITION_DEAD
    } else if hit <= -6 {
        POSITION_MORTALLYW
    } else if hit <= -3 {
        POSITION_INCAP
    } else {
        POSITION_STUNNED
    };
}

/// Start one char fighting another.
pub fn set_fighting(w: &mut World, ch: CharId, vict: CharId) {
    if w[ch].specials.fighting.is_some() {
        return;
    }
    w.combat_list.insert(0, ch);
    if w.is_affected(ch, AFF_SLEEP) {
        affect_from_char(w, ch, SPELL_SLEEP);
    }
    w[ch].specials.fighting = Some(vict);
    w[ch].specials.position = POSITION_FIGHTING;
}

/// Remove a char from the list of fighting chars.
pub fn stop_fighting(w: &mut World, ch: CharId) {
    w.combat_list.retain(|&c| c != ch);
    w[ch].specials.fighting = None;
    w[ch].specials.position = POSITION_STANDING;
    update_pos(w, ch);
}

const MAX_NPC_CORPSE_TIME: i32 = 5;
const MAX_PC_CORPSE_TIME: i32 = 10;

pub fn make_corpse(w: &mut World, ch: CharId) {
    let who = if w.is_npc(ch) { w[ch].player.short_descr.clone() } else { w[ch].player.name.clone() };
    let mut corpse = Obj {
        item_number: NOWHERE,
        in_room: NOWHERE,
        name: "corpse".into(),
        description: format!("Corpse of {} is lying here.", who),
        short_description: format!("Corpse of {}", who),
        ..Default::default()
    };
    corpse.obj_flags.type_flag = ITEM_CONTAINER;
    corpse.obj_flags.wear_flags = ITEM_TAKE;
    corpse.obj_flags.value[0] = 0;
    corpse.obj_flags.value[3] = 1;
    corpse.obj_flags.weight = w[ch].player.weight + w[ch].specials.carry_weight;
    corpse.obj_flags.cost_per_day = 100000;
    corpse.obj_flags.timer = if w.is_npc(ch) { MAX_NPC_CORPSE_TIME } else { MAX_PC_CORPSE_TIME };
    corpse.contains = std::mem::take(&mut w[ch].carrying);
    let corpse = w.objs.insert(corpse);
    for o in w[corpse].contains.clone() {
        w[o].in_obj = Some(corpse);
        w[o].carried_by = None;
    }

    if w[ch].points.gold > 0 && (w.is_npc(ch) || w[ch].desc.is_some()) {
        let money = create_money(w, w[ch].points.gold);
        w[ch].points.gold = 0;
        obj_to_obj(w, money, corpse);
    }
    for i in 0..MAX_WEAR {
        if w[ch].equipment[i].is_some() {
            if let Some(o) = unequip_char(w, ch, i) {
                obj_to_obj(w, o, corpse);
            }
        }
    }
    w[ch].specials.carry_items = 0;
    w[ch].specials.carry_weight = 0;
    w.object_list.insert(0, corpse);
    object_list_new_owner(w, &[corpse], None);
    let r = w[ch].in_room;
    obj_to_room(w, corpse, r);
}

/// When ch kills victim.
pub fn change_alignment(w: &mut World, ch: CharId, victim: CharId) {
    let align = w[ch].specials.alignment - w[victim].specials.alignment;
    let a = &mut w[ch].specials.alignment;
    if align > 0 {
        if align > 650 {
            *a = (*a + (align - 650) / 4).min(1000);
        } else {
            *a /= 2;
        }
    } else if align < -650 {
        *a = (*a + (align + 650) / 4).max(-1000);
    } else {
        *a /= 2;
    }
}

pub fn death_cry(w: &mut World, ch: CharId) {
    act(w, "Your blood freezes as you hear $ns death cry.", false, ch, None, Vict::None, TO_ROOM);
    let was_in = w[ch].in_room;
    for door in 0..6 {
        if w.can_go(ch, door) {
            let to = w.room(was_in).dir_option[door].as_ref().unwrap().to_room;
            w[ch].in_room = to;
            act(w, "Your blood freezes as you hear someones death cry.", false, ch, None, Vict::None, TO_ROOM);
            w[ch].in_room = was_in;
        }
    }
}

pub fn raw_kill(w: &mut World, ch: CharId) {
    if w[ch].specials.fighting.is_some() {
        stop_fighting(w, ch);
    }
    death_cry(w, ch);
    make_corpse(w, ch);
    affect_total(w, ch);
    extract_char(w, ch);
}

pub fn die(w: &mut World, ch: CharId) {
    let e = w[ch].points.exp;
    gain_exp(w, ch, -(e / 2));
    raw_kill(w, ch);
}

pub fn group_gain(w: &mut World, ch: CharId, victim: CharId) {
    let k = w[ch].master.unwrap_or(ch);
    let room = w[ch].in_room;
    let in_group = |w: &World, c: CharId| w.is_affected(c, AFF_GROUP) && w[c].in_room == room;
    let mut no_members = if in_group(w, k) { 1 } else { 0 };
    let followers = w[k].followers.clone();
    for &f in &followers {
        if in_group(w, f) {
            no_members += 1;
        }
    }
    let share = if no_members >= 1 { (450000 / no_members).min((w[victim].points.exp / 3) / no_members) } else { 0 };
    if in_group(w, k) {
        act(w, "You receive your share of experience.", false, k, None, Vict::None, TO_CHAR);
        gain_exp(w, k, share);
        change_alignment(w, k, victim);
    }
    for f in followers {
        if w.ch_exists(f) && in_group(w, f) {
            act(w, "You receive your share of experience.", false, f, None, Vict::None, TO_CHAR);
            gain_exp(w, f, share);
            change_alignment(w, f, victim);
        }
    }
}

fn replace_string(s: &str, weapon: &str) -> String {
    let mut out = String::new();
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c == '#' {
            match it.next() {
                Some('W') => out.push_str(weapon),
                _ => out.push('#'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub fn dam_message(w: &mut World, dam: i32, ch: CharId, victim: CharId, w_type: i32) {
    const DAM_WEAPONS: [(&str, &str, &str); 8] = [
        ("$n misses $N with $s #W.", "You miss $N with your #W.", "$n miss you with $s #W."),
        ("$n tickles $N with $s #W.", "You tickle $N as you #W $M.", "$n tickle you as $e #W you."),
        ("$n barely #W $N.", "You barely #W $N.", "$n barely #W you."),
        ("$n #W $N.", "You #W $N.", "$n #W you."),
        ("$n #W $N hard.", "You #W $N hard.", "$n #W you hard."),
        ("$n #W $N very hard.", "You #W $N very hard.", "$n #W you very hard."),
        ("$n #W $N extremely hard.", "You #W $N extremely hard.", "$n #W you extremely hard."),
        (
            "$n massacre $N to small fragments with $s #W.",
            "You massacre $N to small fragments with your #W.",
            "$n massacre you to small fragments with $s #W.",
        ),
    ];
    let wt = ((w_type - TYPE_HIT).clamp(0, 8)) as usize;
    let (sing, plur) = ATTACK_HIT_TEXT[wt];
    let wield = w[ch].equipment[WIELD];
    // (index, room uses plural?, victim uses plural?)
    let (i, room_pl, vict_pl) = if dam == 0 {
        (0, false, false)
    } else if dam <= 2 {
        (1, false, true)
    } else if dam <= 4 {
        (2, false, true)
    } else if dam <= 6 {
        (3, true, true)
    } else if dam <= 10 {
        (4, true, false)
    } else if dam <= 15 {
        (5, true, true)
    } else if dam <= 20 {
        (6, false, false)
    } else {
        (7, false, false)
    };
    let (to_room, to_char, to_victim) = DAM_WEAPONS[i];
    let b = replace_string(to_room, if room_pl { plur } else { sing });
    act(w, &b, false, ch, wield, Vict::Ch(victim), TO_NOTVICT);
    let b = replace_string(to_char, sing);
    act(w, &b, false, ch, wield, Vict::Ch(victim), TO_CHAR);
    let b = replace_string(to_victim, if vict_pl { plur } else { sing });
    act(w, &b, false, ch, wield, Vict::Ch(victim), TO_VICT);
}

pub fn damage(w: &mut World, ch: CharId, victim: CharId, dam: i32, attacktype: i32) {
    if !w.ch_exists(ch) || !w.ch_exists(victim) || w.get_pos(victim) <= POSITION_DEAD {
        return;
    }
    let mut dam = dam;
    if w.level(victim) > 20 && !w.is_npc(victim) {
        dam = 0;
    }

    if victim != ch {
        if w.get_pos(victim) > POSITION_STUNNED {
            if w[victim].specials.fighting.is_none() {
                set_fighting(w, victim, ch);
            }
            w[victim].specials.position = POSITION_FIGHTING;
        }
        if w.get_pos(ch) > POSITION_STUNNED {
            if w[ch].specials.fighting.is_none() {
                set_fighting(w, ch, victim);
            }
            if w.is_npc(ch) && w.is_npc(victim) {
                if let Some(m) = w[victim].master {
                    if number(0, 10) == 0 && w.is_affected(victim, AFF_CHARM) && w[m].in_room == w[ch].in_room {
                        if w[ch].specials.fighting.is_some() {
                            stop_fighting(w, ch);
                        }
                        hit(w, ch, m, TYPE_UNDEFINED);
                        return;
                    }
                }
            }
        }
    }

    if w[victim].master == Some(ch) {
        stop_follower(w, victim);
    }
    if w.is_affected(ch, AFF_INVISIBLE) {
        appear(w, ch);
    }
    if w.is_affected(victim, AFF_SANCTUARY) {
        dam = dam.min(18);
    }
    dam = dam.min(100).max(0);

    w[victim].points.hit -= dam;
    if ch != victim {
        let l = w.level(victim);
        gain_exp(w, ch, l * dam);
    }
    update_pos(w, victim);

    if (TYPE_HIT..=TYPE_SLASH).contains(&attacktype) {
        if w[ch].equipment[WIELD].is_none() {
            dam_message(w, dam, ch, victim, TYPE_HIT);
        } else {
            dam_message(w, dam, ch, victim, attacktype);
        }
    } else if let Some(list) = w.fight_messages.iter().find(|l| l.a_type == attacktype) {
        if !list.msg.is_empty() {
            let nr = dice(1, list.msg.len() as i32) as usize;
            let m = list.msg[nr - 1].clone();
            let wield = w[ch].equipment[WIELD];
            let msgs = if !w.is_npc(victim) && w.level(victim) > 20 {
                &m.god_msg
            } else if dam != 0 {
                if w.get_pos(victim) == POSITION_DEAD {
                    &m.die_msg
                } else {
                    &m.hit_msg
                }
            } else {
                &m.miss_msg
            };
            act(w, &msgs.attacker_msg, false, ch, wield, Vict::Ch(victim), TO_CHAR);
            act(w, &msgs.victim_msg, false, ch, wield, Vict::Ch(victim), TO_VICT);
            act(w, &msgs.room_msg, false, ch, wield, Vict::Ch(victim), TO_NOTVICT);
        }
    }

    match w.get_pos(victim) {
        POSITION_MORTALLYW => {
            act(w, "$n is mortally wounded, and will die soon, if not aided.", true, victim, None, Vict::None, TO_ROOM);
            act(w, "You are mortally wounded, and will die soon, if not aided.", false, victim, None, Vict::None, TO_CHAR);
        }
        POSITION_INCAP => {
            act(w, "$n is incapacitated and will slowly die, if not aided.", true, victim, None, Vict::None, TO_ROOM);
            act(w, "You are incapacitated an will slowly die, if not aided.", false, victim, None, Vict::None, TO_CHAR);
        }
        POSITION_STUNNED => {
            act(w, "$n is stunned, but will probably regain conscience again.", true, victim, None, Vict::None, TO_ROOM);
            act(w, "You're stunned, but will probably regain conscience again.", false, victim, None, Vict::None, TO_CHAR);
        }
        POSITION_DEAD => {
            act(w, "$n is dead! R.I.P.", true, victim, None, Vict::None, TO_ROOM);
            act(w, "You are dead!  Sorry...", false, victim, None, Vict::None, TO_CHAR);
        }
        _ => {
            let max_hit = hit_limit(w, victim);
            if dam > max_hit / 5 {
                act(w, "That Really did HURT!", false, victim, None, Vict::None, TO_CHAR);
            }
            if w[victim].points.hit < max_hit / 5 {
                act(w, "You wish that your wounds would stop BLEEDING that much!", false, victim, None, Vict::None, TO_CHAR);
                if w.is_npc(victim) && w[victim].specials.act & ACT_WIMPY != 0 {
                    do_flee(w, victim, "", 0);
                }
            }
        }
    }
    if !w.ch_exists(victim) || !w.ch_exists(ch) {
        return;
    }

    if !w.is_npc(victim) && w[victim].desc.is_none() {
        do_flee(w, victim, "", 0);
        if w.ch_exists(victim) && w[victim].specials.fighting.is_none() {
            act(w, "$n is rescued by divine forces.", false, victim, None, Vict::None, TO_ROOM);
            w[victim].specials.was_in_room = w[victim].in_room;
            char_from_room(w, victim);
            char_to_room(w, victim, 0);
        }
    }
    if !w.ch_exists(victim) || !w.ch_exists(ch) {
        return;
    }

    if w.get_pos(victim) < POSITION_STUNNED && w[ch].specials.fighting == Some(victim) {
        stop_fighting(w, ch);
    }
    if !w.awake(victim) && w[victim].specials.fighting.is_some() {
        stop_fighting(w, victim);
    }

    if w.get_pos(victim) == POSITION_DEAD {
        if w.is_npc(victim) || w[victim].desc.is_some() {
            if w.is_affected(ch, AFF_GROUP) {
                group_gain(w, ch, victim);
            } else {
                let mut exp = w[victim].points.exp / 3;
                let ldiff = w.level(victim) - w.level(ch);
                if w.is_npc(ch) {
                    exp += (exp * ldiff.min(4)) >> 3;
                } else {
                    exp += (exp * ldiff.min(8)) >> 3;
                }
                exp = exp.max(1);
                gain_exp(w, ch, exp);
                change_alignment(w, ch, victim);
            }
        }
        if !w.is_npc(victim) {
            let killer = if w.is_npc(ch) { w[ch].player.short_descr.clone() } else { w[ch].player.name.clone() };
            log(&format!(
                "{} killed by {} at {}",
                w[victim].player.name,
                killer,
                w.room(w[victim].in_room).name
            ));
        }
        die(w, victim);
    }
}

pub fn hit(w: &mut World, ch: CharId, victim: CharId, typ: i32) {
    if !w.ch_exists(ch) || !w.ch_exists(victim) {
        return;
    }
    if w[ch].in_room != w[victim].in_room {
        log("NOT SAME ROOM WHEN FIGHTING!");
        return;
    }

    let mut wielded = None;
    let w_type = match w[ch].equipment[WIELD] {
        Some(o) if w[o].obj_flags.type_flag == ITEM_WEAPON => {
            wielded = Some(o);
            match w[o].obj_flags.value[3] {
                0..=2 => TYPE_WHIP,
                3 => TYPE_SLASH,
                4..=6 => TYPE_CRUSH,
                7 => TYPE_BLUDGEON,
                8..=11 => TYPE_PIERCE,
                _ => TYPE_HIT,
            }
        }
        _ => {
            if w.is_npc(ch) && w[ch].specials.attack_type >= TYPE_HIT {
                w[ch].specials.attack_type
            } else {
                TYPE_HIT
            }
        }
    };

    let mut calc_thaco = if !w.is_npc(ch) {
        let cls = (w[ch].player.class - 1).clamp(0, 3) as usize;
        THACO[cls][w.level(ch).clamp(0, 24) as usize]
    } else {
        20
    };
    calc_thaco -= STR_APP[w.strength_apply_index(ch)].tohit;
    calc_thaco -= w[ch].points.hitroll;

    let diceroll = number(1, 20);
    let mut victim_ac = w[victim].points.armor / 10;
    if w.awake(victim) {
        victim_ac += DEX_APP[w[victim].tmpabilities.dex.clamp(0, 25) as usize].defensive;
    }
    victim_ac = victim_ac.max(-10);

    if diceroll < 20 && w.awake(victim) && (diceroll == 1 || (calc_thaco - diceroll) > victim_ac) {
        if typ == SKILL_BACKSTAB {
            damage(w, ch, victim, 0, SKILL_BACKSTAB);
        } else {
            damage(w, ch, victim, 0, w_type);
        }
    } else {
        let mut dam = STR_APP[w.strength_apply_index(ch)].todam;
        dam += w[ch].points.damroll;
        match wielded {
            None => {
                if w.is_npc(ch) {
                    dam += dice(w[ch].specials.damnodice, w[ch].specials.damsizedice);
                } else {
                    dam += number(0, 2);
                }
            }
            Some(o) => dam += dice(w[o].obj_flags.value[1], w[o].obj_flags.value[2]),
        }
        if w.get_pos(victim) < POSITION_FIGHTING {
            dam *= 1 + (POSITION_FIGHTING - w.get_pos(victim)) / 3;
        }
        dam = dam.max(1);
        if typ == SKILL_BACKSTAB {
            dam *= BACKSTAB_MULT[w.level(ch).clamp(0, 24) as usize];
            damage(w, ch, victim, dam, SKILL_BACKSTAB);
        } else {
            damage(w, ch, victim, dam, w_type);
        }
    }
}

/// Control the fights going on.
pub fn perform_violence(w: &mut World) {
    let list = w.combat_list.clone();
    for ch in list {
        if !w.ch_exists(ch) || !w.combat_list.contains(&ch) {
            continue;
        }
        let Some(v) = w[ch].specials.fighting else {
            w.combat_list.retain(|&c| c != ch);
            continue;
        };
        if w.awake(ch) && w.ch_exists(v) && w[ch].in_room == w[v].in_room {
            hit(w, ch, v, TYPE_UNDEFINED);
        } else {
            stop_fighting(w, ch);
        }
    }
}
