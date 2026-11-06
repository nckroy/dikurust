//! Eating, drinking, wearing and wielding (act.obj2.c).

use crate::act_info::get_object_in_equip_vis;
use crate::comm::*;
use crate::constants::*;
use crate::game::World;
use crate::handler::*;
use crate::interpreter::{argument_interpreter, one_argument, search_block};
use crate::limits::gain_condition;
use crate::spells::SPELL_POISON;
use crate::structs::*;
use crate::utils::*;

pub fn weight_change_object(w: &mut World, obj: ObjId, weight: i32) {
    if w[obj].in_room != NOWHERE {
        w[obj].obj_flags.weight += weight;
    } else if let Some(c) = w[obj].carried_by {
        obj_from_char(w, obj);
        w[obj].obj_flags.weight += weight;
        obj_to_char(w, obj, c);
    } else if let Some(o) = w[obj].in_obj {
        obj_from_obj(w, obj);
        w[obj].obj_flags.weight += weight;
        obj_to_obj(w, obj, o);
    } else {
        // equipped, or nowhere at all
        w[obj].obj_flags.weight += weight;
        log("Unknown attempt to subtract weight from an object.");
    }
}

pub fn name_from_drinkcon(w: &mut World, obj: ObjId) {
    let n = w[obj].name.clone();
    if let Some(p) = n.find(' ') {
        w[obj].name = n[p + 1..].to_string();
    }
}

pub fn name_to_drinkcon(w: &mut World, obj: ObjId, typ: i32) {
    let n = format!("{} {}", DRINKNAMES[typ.clamp(0, 15) as usize], w[obj].name);
    w[obj].name = n;
}

fn liq(w: &World, o: ObjId) -> usize {
    w[o].obj_flags.value[2].clamp(0, 15) as usize
}

fn me(w: &mut World, ch: CharId, s: &str) {
    act(w, s, false, ch, None, Vict::None, TO_CHAR);
}

fn poison_af(duration: i32) -> Affect {
    Affect { type_: SPELL_POISON, duration, modifier: 0, location: APPLY_NONE, bitvector: AFF_POISON }
}

pub fn do_drink(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (buf, _) = one_argument(argument);
    let Some(temp) = get_obj_in_list_vis(w, ch, &buf, &w[ch].carrying) else {
        me(w, ch, "You can't find it!");
        return;
    };
    if w[temp].obj_flags.type_flag != ITEM_DRINKCON {
        me(w, ch, "You can't drink from that!");
        return;
    }
    let cond = w[ch].specials.conditions;
    if cond[DRUNK] > 10 && cond[THIRST] > 0 {
        me(w, ch, "You simply fail to reach your mouth!");
        act(w, "$n tried to drink but missed $s mouth!", true, ch, None, Vict::None, TO_ROOM);
        return;
    }
    if cond[FULL] > 20 && cond[THIRST] > 0 {
        me(w, ch, "Your stomach can't contain anymore!");
        return;
    }
    if w[temp].obj_flags.value[1] > 0 {
        let l = liq(w, temp);
        let b = format!("$n drinks {} from $p", DRINKS[l]);
        act(w, &b, true, ch, Some(temp), Vict::None, TO_ROOM);
        send_to_char(w, &format!("You drink the {}.\n\r", DRINKS[l]), ch);
        let mut amount = if DRINK_AFF[l][DRUNK] > 0 {
            (25 - w[ch].specials.conditions[THIRST]) / DRINK_AFF[l][DRUNK]
        } else {
            number(3, 10)
        };
        amount = amount.min(w[temp].obj_flags.value[1]);
        weight_change_object(w, temp, -amount);
        gain_condition(w, ch, DRUNK, DRINK_AFF[l][DRUNK] * amount / 4);
        gain_condition(w, ch, FULL, DRINK_AFF[l][FULL] * amount / 4);
        gain_condition(w, ch, THIRST, DRINK_AFF[l][THIRST] * amount / 4);
        let cond = w[ch].specials.conditions;
        if cond[DRUNK] > 10 {
            me(w, ch, "You feel drunk.");
        }
        if cond[THIRST] > 20 {
            me(w, ch, "You do not feel thirsty.");
        }
        if cond[FULL] > 20 {
            me(w, ch, "You are full.");
        }
        if w[temp].obj_flags.value[3] != 0 {
            me(w, ch, "Ooups, it tasted rather strange ?!!?");
            act(w, "$n chokes and utters some strange sounds.", true, ch, None, Vict::None, TO_ROOM);
            let mut af = poison_af(amount * 3);
            affect_join(w, ch, &mut af, false, false);
        }
        w[temp].obj_flags.value[1] -= amount;
        if w[temp].obj_flags.value[1] == 0 {
            w[temp].obj_flags.value[2] = 0;
            w[temp].obj_flags.value[3] = 0;
            name_from_drinkcon(w, temp);
        }
        return;
    }
    me(w, ch, "It's empty already.");
}

pub fn do_eat(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (buf, _) = one_argument(argument);
    let Some(temp) = get_obj_in_list_vis(w, ch, &buf, &w[ch].carrying) else {
        me(w, ch, "You can't find it!");
        return;
    };
    if w[temp].obj_flags.type_flag != ITEM_FOOD && w.level(ch) < 22 {
        me(w, ch, "Your stomach refuses to eat that!?!");
        return;
    }
    if w[ch].specials.conditions[FULL] > 20 {
        me(w, ch, "You are to full to eat more!");
        return;
    }
    act(w, "$n eats $p", true, ch, Some(temp), Vict::None, TO_ROOM);
    act(w, "You eat the $o.", false, ch, Some(temp), Vict::None, TO_CHAR);
    let v0 = w[temp].obj_flags.value[0];
    gain_condition(w, ch, FULL, v0);
    if w[ch].specials.conditions[FULL] > 20 {
        me(w, ch, "You are full.");
    }
    if w[temp].obj_flags.value[3] != 0 && w.level(ch) < 21 {
        me(w, ch, "Ooups, it tasted rather strange ?!!?");
        act(w, "$n coughs and utters some strange sounds.", false, ch, None, Vict::None, TO_ROOM);
        let mut af = poison_af(v0 * 2);
        affect_join(w, ch, &mut af, false, false);
    }
    extract_obj(w, temp);
}

pub fn do_pour(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (arg1, arg2) = argument_interpreter(argument);
    if arg1.is_empty() {
        me(w, ch, "What do you want to pour from?");
        return;
    }
    let Some(from) = get_obj_in_list_vis(w, ch, &arg1, &w[ch].carrying) else {
        me(w, ch, "You can't find it!");
        return;
    };
    if w[from].obj_flags.type_flag != ITEM_DRINKCON {
        me(w, ch, "You can't pour from that!");
        return;
    }
    if w[from].obj_flags.value[1] == 0 {
        act(w, "The $p is empty.", false, ch, Some(from), Vict::None, TO_CHAR);
        return;
    }
    if arg2.is_empty() {
        me(w, ch, "Where do you want it? Out or in what?");
        return;
    }
    if str_cmp(&arg2, "out") == 0 {
        act(w, "$n empties $p", true, ch, Some(from), Vict::None, TO_ROOM);
        act(w, "You empty the $p.", false, ch, Some(from), Vict::None, TO_CHAR);
        let v1 = w[from].obj_flags.value[1];
        weight_change_object(w, from, -v1);
        w[from].obj_flags.value[1] = 0;
        w[from].obj_flags.value[2] = 0;
        w[from].obj_flags.value[3] = 0;
        name_from_drinkcon(w, from);
        return;
    }
    let Some(to) = get_obj_in_list_vis(w, ch, &arg2, &w[ch].carrying) else {
        me(w, ch, "You can't find it!");
        return;
    };
    if w[to].obj_flags.type_flag != ITEM_DRINKCON {
        me(w, ch, "You can't pour anything into that.");
        return;
    }
    if w[to].obj_flags.value[1] != 0 && w[to].obj_flags.value[2] != w[from].obj_flags.value[2] {
        me(w, ch, "There is already another liquid in it!");
        return;
    }
    if w[to].obj_flags.value[1] >= w[to].obj_flags.value[0] {
        me(w, ch, "There is no room for more.");
        return;
    }
    let l = liq(w, from);
    send_to_char(w, &format!("You pour the {} into the {}.", DRINKS[l], arg2), ch);
    if w[to].obj_flags.value[1] == 0 {
        name_to_drinkcon(w, to, l as i32);
    }
    w[to].obj_flags.value[2] = w[from].obj_flags.value[2];
    let amount = (w[to].obj_flags.value[0] - w[to].obj_flags.value[1]).min(w[from].obj_flags.value[1]);
    w[from].obj_flags.value[1] -= amount;
    weight_change_object(w, from, -amount);
    w[to].obj_flags.value[1] += amount;
    weight_change_object(w, to, amount);
    let p = (w[to].obj_flags.value[3] != 0 || w[from].obj_flags.value[3] != 0) as i32;
    w[to].obj_flags.value[3] = p;
    // (the C code had `if (value[1] = 0)` here, which never fired)
    if w[from].obj_flags.value[1] == 0 {
        w[from].obj_flags.value[2] = 0;
        w[from].obj_flags.value[3] = 0;
        name_from_drinkcon(w, from);
    }
}

pub fn do_sip(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (arg, _) = one_argument(argument);
    let Some(temp) = get_obj_in_list_vis(w, ch, &arg, &w[ch].carrying) else {
        me(w, ch, "You can't find it!");
        return;
    };
    if w[temp].obj_flags.type_flag != ITEM_DRINKCON {
        me(w, ch, "You can't sip from that!");
        return;
    }
    if w[ch].specials.conditions[DRUNK] > 10 {
        me(w, ch, "You simply fail to reach your mouth!");
        act(w, "$n tries to sip, but fails!", true, ch, None, Vict::None, TO_ROOM);
        return;
    }
    if w[temp].obj_flags.value[1] == 0 {
        me(w, ch, "But there is nothing in it?");
        return;
    }
    act(w, "$n sips from the $o", true, ch, Some(temp), Vict::None, TO_ROOM);
    let l = liq(w, temp);
    send_to_char(w, &format!("It tastes like {}.\n\r", DRINKS[l]), ch);
    gain_condition(w, ch, DRUNK, DRINK_AFF[l][DRUNK] / 4);
    gain_condition(w, ch, FULL, DRINK_AFF[l][FULL] / 4);
    gain_condition(w, ch, THIRST, DRINK_AFF[l][THIRST] / 4);
    weight_change_object(w, temp, -1);
    let cond = w[ch].specials.conditions;
    if cond[DRUNK] > 10 {
        me(w, ch, "You feel drunk.");
    }
    if cond[THIRST] > 20 {
        me(w, ch, "You do not feel thirsty.");
    }
    if cond[FULL] > 20 {
        me(w, ch, "You are full.");
    }
    if w[temp].obj_flags.value[3] != 0 && !w.is_affected(ch, AFF_POISON) {
        me(w, ch, "But it also had a strange taste!");
        affect_to_char(w, ch, &poison_af(3));
    }
    w[temp].obj_flags.value[1] -= 1;
    if w[temp].obj_flags.value[1] == 0 {
        w[temp].obj_flags.value[2] = 0;
        w[temp].obj_flags.value[3] = 0;
        name_from_drinkcon(w, temp);
    }
}

pub fn do_taste(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (arg, _) = one_argument(argument);
    let Some(temp) = get_obj_in_list_vis(w, ch, &arg, &w[ch].carrying) else {
        me(w, ch, "You can't find it!");
        return;
    };
    if w[temp].obj_flags.type_flag == ITEM_DRINKCON {
        do_sip(w, ch, argument, 0);
        return;
    }
    if w[temp].obj_flags.type_flag != ITEM_FOOD {
        me(w, ch, "Taste that?!? Your stomach refuses!");
        return;
    }
    act(w, "$n tastes the $o", false, ch, Some(temp), Vict::None, TO_ROOM);
    act(w, "You taste the $o", false, ch, Some(temp), Vict::None, TO_CHAR);
    gain_condition(w, ch, FULL, 1);
    if w[ch].specials.conditions[FULL] > 20 {
        me(w, ch, "You are full.");
    }
    if w[temp].obj_flags.value[3] != 0 && !w.is_affected(ch, AFF_POISON) {
        me(w, ch, "Ooups, it did not taste good at all!");
        affect_to_char(w, ch, &poison_af(2));
    }
    w[temp].obj_flags.value[0] -= 1;
    if w[temp].obj_flags.value[0] == 0 {
        me(w, ch, "There is nothing left now.");
        extract_obj(w, temp);
    }
}

fn perform_wear(w: &mut World, ch: CharId, obj: ObjId, keyword: i32) {
    let (msg, hide) = match keyword {
        0 => ("$n light $p and holds it.", false),
        1 => ("$n wears $p on $s finger.", true),
        2 => ("$n wears $p around $s neck.", true),
        3 => ("$n wears $p on $s body.", true),
        4 => ("$n wears $p on $s head.", true),
        5 => ("$n wears $p on $s legs.", true),
        6 => ("$n wears $p on $s feet.", true),
        7 => ("$n wears $p on $s hands.", true),
        8 => ("$n wears $p on $s arms.", true),
        9 => ("$n wears $p about $s body.", true),
        10 => ("$n wears $p about $s waist.", true),
        11 => ("$n wears $p around $s wrist.", true),
        12 => ("$n wields $p.", true),
        13 => ("$n grabs $p.", true),
        14 => ("$n starts using $p as shield.", true),
        _ => return,
    };
    act(w, msg, hide, ch, Some(obj), Vict::None, TO_ROOM);
}

/// Wear an object in a single fixed slot (the common case).
fn wear_single(w: &mut World, ch: CharId, obj: ObjId, keyword: i32, flag: i32, pos: usize, occupied: &str, cant: &str) {
    if !w.can_wear(obj, flag) {
        send_to_char(w, cant, ch);
    } else if w[ch].equipment[pos].is_some() {
        send_to_char(w, occupied, ch);
    } else {
        send_to_char(w, "OK.\n\r", ch);
        perform_wear(w, ch, obj, keyword);
        obj_from_char(w, obj);
        equip_char(w, ch, obj, pos);
    }
}

pub fn wear(w: &mut World, ch: CharId, obj: ObjId, keyword: i32) {
    let eq = |w: &World, p: usize| w[ch].equipment[p].is_some();
    match keyword {
        0 => {
            if eq(w, WEAR_LIGHT) {
                send_to_char(w, "You are already holding a light source.\n\r", ch);
            } else {
                send_to_char(w, "Ok.\n\r", ch);
                perform_wear(w, ch, obj, keyword);
                obj_from_char(w, obj);
                equip_char(w, ch, obj, WEAR_LIGHT);
                if w[obj].obj_flags.value[2] != 0 {
                    let r = w[ch].in_room;
                    w.room_mut(r).light += 1;
                }
            }
        }
        1 => {
            if !w.can_wear(obj, ITEM_WEAR_FINGER) {
                send_to_char(w, "You can't wear that on your finger.\n\r", ch);
            } else if eq(w, WEAR_FINGER_L) && eq(w, WEAR_FINGER_R) {
                send_to_char(w, "You are already wearing something on your fingers.\n\r", ch);
            } else {
                perform_wear(w, ch, obj, keyword);
                let (side, pos) = if eq(w, WEAR_FINGER_L) { ("right", WEAR_FINGER_R) } else { ("left", WEAR_FINGER_L) };
                let b = format!("You put the {} on your {} finger.\n\r", fname(&w[obj].name), side);
                send_to_char(w, &b, ch);
                obj_from_char(w, obj);
                equip_char(w, ch, obj, pos);
            }
        }
        2 => {
            if !w.can_wear(obj, ITEM_WEAR_NECK) {
                send_to_char(w, "You can't wear that around your neck.\n\r", ch);
            } else if eq(w, WEAR_NECK_1) && eq(w, WEAR_NECK_2) {
                send_to_char(w, "You can't wear any more around your neck.\n\r", ch);
            } else {
                send_to_char(w, "OK.\n\r", ch);
                perform_wear(w, ch, obj, keyword);
                let pos = if eq(w, WEAR_NECK_1) { WEAR_NECK_2 } else { WEAR_NECK_1 };
                obj_from_char(w, obj);
                equip_char(w, ch, obj, pos);
            }
        }
        3 => wear_single(w, ch, obj, 3, ITEM_WEAR_BODY, WEAR_BODY, "You already wear something on your body.\n\r", "You can't wear that on your body.\n\r"),
        4 => wear_single(w, ch, obj, 4, ITEM_WEAR_HEAD, WEAR_HEAD, "You already wear something on your head.\n\r", "You can't wear that on your head.\n\r"),
        5 => wear_single(w, ch, obj, 5, ITEM_WEAR_LEGS, WEAR_LEGS, "You already wear something on your legs.\n\r", "You can't wear that on your legs.\n\r"),
        6 => wear_single(w, ch, obj, 6, ITEM_WEAR_FEET, WEAR_FEET, "You already wear something on your feet.\n\r", "You can't wear that on your feet.\n\r"),
        7 => wear_single(w, ch, obj, 7, ITEM_WEAR_HANDS, WEAR_HANDS, "You already wear something on your hands.\n\r", "You can't wear that on your hands.\n\r"),
        8 => wear_single(w, ch, obj, 8, ITEM_WEAR_ARMS, WEAR_ARMS, "You already wear something on your arms.\n\r", "You can't wear that on your arms.\n\r"),
        9 => wear_single(w, ch, obj, 9, ITEM_WEAR_ABOUT, WEAR_ABOUT, "You already wear something about your body.\n\r", "You can't wear that about your body.\n\r"),
        10 => wear_single(w, ch, obj, 10, ITEM_WEAR_WAISTE, WEAR_WAISTE, "You already wear something about your waiste.\n\r", "You can't wear that about your waist.\n\r"),
        11 => {
            if !w.can_wear(obj, ITEM_WEAR_WRIST) {
                send_to_char(w, "You can't wear that around your wrist.\n\r", ch);
            } else if eq(w, WEAR_WRIST_L) && eq(w, WEAR_WRIST_R) {
                send_to_char(w, "You already wear something around both your wrists.\n\r", ch);
            } else {
                perform_wear(w, ch, obj, keyword);
                obj_from_char(w, obj);
                let (side, pos) = if eq(w, WEAR_WRIST_L) { ("right", WEAR_WRIST_R) } else { ("left", WEAR_WRIST_L) };
                let b = format!("You wear the {} around your {} wrist.\n\r", fname(&w[obj].name), side);
                send_to_char(w, &b, ch);
                equip_char(w, ch, obj, pos);
            }
        }
        12 => {
            if !w.can_wear(obj, ITEM_WIELD) {
                send_to_char(w, "You can't wield that.\n\r", ch);
            } else if eq(w, WIELD) {
                send_to_char(w, "You are already wielding something.\n\r", ch);
            } else if w[obj].obj_flags.weight > STR_APP[w.strength_apply_index(ch)].wield_w {
                send_to_char(w, "It is too heavy for you to use.\n\r", ch);
            } else {
                send_to_char(w, "OK.\n\r", ch);
                perform_wear(w, ch, obj, keyword);
                obj_from_char(w, obj);
                equip_char(w, ch, obj, WIELD);
            }
        }
        13 => wear_single(w, ch, obj, 13, ITEM_HOLD, HOLD, "You are already holding something.\n\r", "You can't hold this.\n\r"),
        14 => {
            if !w.can_wear(obj, ITEM_WEAR_SHIELD) {
                send_to_char(w, "You can't use that as a shield.\n\r", ch);
            } else if eq(w, WEAR_SHIELD) {
                send_to_char(w, "You are already using a shield\n\r", ch);
            } else {
                perform_wear(w, ch, obj, keyword);
                let b = format!("You start using the {}.\n\r", fname(&w[obj].name));
                send_to_char(w, &b, ch);
                obj_from_char(w, obj);
                equip_char(w, ch, obj, WEAR_SHIELD);
            }
        }
        -1 => {
            let b = format!("Wear {} where?.\n\r", fname(&w[obj].name));
            send_to_char(w, &b, ch);
        }
        -2 => {
            let b = format!("You can't wear the {}.\n\r", fname(&w[obj].name));
            send_to_char(w, &b, ch);
        }
        _ => log("Unknown type called in wear."),
    }
}

pub fn do_wear(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    const KEYWORDS: [&str; 12] = ["finger", "neck", "body", "head", "legs", "feet", "hands", "arms", "about", "waist", "wrist", "shield"];
    let (arg1, arg2) = argument_interpreter(argument);
    if arg1.is_empty() {
        send_to_char(w, "Wear what?\n\r", ch);
        return;
    }
    let Some(obj) = get_obj_in_list_vis(w, ch, &arg1, &w[ch].carrying) else {
        send_to_char(w, &format!("You do not seem to have the '{}'.\n\r", arg1), ch);
        return;
    };
    if !arg2.is_empty() {
        let k = search_block(&arg2, &KEYWORDS, false);
        if k == -1 {
            send_to_char(w, &format!("{} is an unknown body location.\n\r", arg2), ch);
        } else {
            // (keywords and wear() numbers line up except "shield", which is 14)
            let kw = if k == 11 { 14 } else { k + 1 };
            wear(w, ch, obj, kw);
        }
    } else {
        let mut keyword = -2;
        let order = [
            (ITEM_WEAR_SHIELD, 14),
            (ITEM_WEAR_FINGER, 1),
            (ITEM_WEAR_NECK, 2),
            (ITEM_WEAR_WRIST, 11),
            (ITEM_WEAR_WAISTE, 10),
            (ITEM_WEAR_ARMS, 8),
            (ITEM_WEAR_HANDS, 7),
            (ITEM_WEAR_FEET, 6),
            (ITEM_WEAR_LEGS, 5),
            (ITEM_WEAR_ABOUT, 9),
            (ITEM_WEAR_HEAD, 4),
            (ITEM_WEAR_BODY, 3),
        ];
        for (flag, k) in order {
            if w.can_wear(obj, flag) {
                keyword = k;
            }
        }
        wear(w, ch, obj, keyword);
    }
}

pub fn do_wield(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (arg1, _) = argument_interpreter(argument);
    if arg1.is_empty() {
        send_to_char(w, "Wield what?\n\r", ch);
        return;
    }
    match get_obj_in_list_vis(w, ch, &arg1, &w[ch].carrying) {
        Some(o) => wear(w, ch, o, 12),
        None => send_to_char(w, &format!("You do not seem to have the '{}'.\n\r", arg1), ch),
    }
}

pub fn do_grab(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (arg1, _) = argument_interpreter(argument);
    if arg1.is_empty() {
        send_to_char(w, "Hold what?\n\r", ch);
        return;
    }
    match get_obj_in_list(w, &arg1, &w[ch].carrying) {
        Some(o) => {
            if w[o].obj_flags.type_flag == ITEM_LIGHT {
                wear(w, ch, o, WEAR_LIGHT as i32);
            } else {
                wear(w, ch, o, 13);
            }
        }
        None => send_to_char(w, &format!("You do not seem to have the '{}'.\n\r", arg1), ch),
    }
}

pub fn do_remove(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (arg1, _) = one_argument(argument);
    if arg1.is_empty() {
        send_to_char(w, "Remove what?\n\r", ch);
        return;
    }
    let Some((obj, j)) = get_object_in_equip_vis(w, ch, &arg1) else {
        send_to_char(w, "You are not using it.\n\r", ch);
        return;
    };
    if w.can_carry_n(ch) != w[ch].specials.carry_items {
        if let Some(o) = unequip_char(w, ch, j) {
            obj_to_char(w, o, ch);
        }
        if w[obj].obj_flags.type_flag == ITEM_LIGHT && w[obj].obj_flags.value[2] != 0 {
            let r = w[ch].in_room;
            w.room_mut(r).light -= 1;
        }
        act(w, "You stop using $p.", false, ch, Some(obj), Vict::None, TO_CHAR);
        act(w, "$n stops using $p.", true, ch, Some(obj), Vict::None, TO_ROOM);
    } else {
        send_to_char(w, "You can't carry that many items.\n\r", ch);
    }
}
