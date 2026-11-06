//! Getting, dropping, putting and giving (act.obj1.c).

use crate::comm::*;
use crate::game::World;
use crate::handler::*;
use crate::interpreter::{argument_interpreter, is_number, one_argument};
use crate::structs::*;
use crate::utils::*;

fn get(w: &mut World, ch: CharId, obj: ObjId, sub: Option<ObjId>) {
    if let Some(sub) = sub {
        obj_from_obj(w, obj);
        obj_to_char(w, obj, ch);
        act(w, "You get $p from $P.", false, ch, Some(obj), Vict::Obj(sub), TO_CHAR);
        if w[sub].carried_by == Some(ch) {
            act(w, "$n gets $p from $s $P.", true, ch, Some(obj), Vict::Obj(sub), TO_ROOM);
        } else {
            act(w, "$n gets $p from $P.", true, ch, Some(obj), Vict::Obj(sub), TO_ROOM);
        }
    } else {
        obj_from_room(w, obj);
        obj_to_char(w, obj, ch);
        act(w, "You get $p.", false, ch, Some(obj), Vict::None, TO_CHAR);
        act(w, "$n gets $p.", true, ch, Some(obj), Vict::None, TO_ROOM);
    }
    if w[obj].obj_flags.type_flag == ITEM_MONEY && w[obj].obj_flags.value[0] >= 1 {
        obj_from_char(w, obj);
        let v = w[obj].obj_flags.value[0];
        send_to_char(w, &format!("There was {} coins.\n\r", v), ch);
        w[ch].points.gold += v;
        extract_obj(w, obj);
    }
}

/// The weight/number/take checks shared by all the get variants.
/// `strict` reproduces the C code's `<` (vs `<=` for "get all").
fn try_get(w: &mut World, ch: CharId, obj: ObjId, sub: Option<ObjId>, strict: bool) -> bool {
    let n_ok = if strict {
        w[ch].specials.carry_items + 1 < w.can_carry_n(ch)
    } else {
        w[ch].specials.carry_items + 1 <= w.can_carry_n(ch)
    };
    let wt = w[ch].specials.carry_weight + w[obj].obj_flags.weight;
    let w_ok = if strict { wt < w.can_carry_w(ch) } else { wt <= w.can_carry_w(ch) };
    if !n_ok {
        let b = format!("{} : You can't carry that many items.\n\r", fname(&w[obj].name));
        send_to_char(w, &b, ch);
        false
    } else if !w_ok {
        let b = format!("{} : You can't carry that much weight.\n\r", fname(&w[obj].name));
        send_to_char(w, &b, ch);
        false
    } else if !w.can_wear(obj, ITEM_TAKE) {
        send_to_char(w, "You can't take that\n\r", ch);
        false
    } else {
        get(w, ch, obj, sub);
        true
    }
}

fn find_container(w: &mut World, ch: CharId, arg2: &str) -> Option<ObjId> {
    let room = w[ch].in_room;
    let sub = get_obj_in_list_vis(w, ch, arg2, &w.room(room).contents)
        .or_else(|| get_obj_in_list_vis(w, ch, arg2, &w[ch].carrying));
    let Some(sub) = sub else {
        send_to_char(w, &format!("You do not see or have the {}.\n\r", arg2), ch);
        return None;
    };
    if w[sub].obj_flags.type_flag != ITEM_CONTAINER {
        let b = format!("The {} is not a container.\n\r", fname(&w[sub].name));
        send_to_char(w, &b, ch);
        return None;
    }
    if w[sub].obj_flags.value[1] & CONT_CLOSED != 0 {
        send_to_char(w, "It's closed.\n\r", ch);
        return None;
    }
    Some(sub)
}

pub fn do_get(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (arg1, arg2) = argument_interpreter(argument);
    let all1 = str_cmp(&arg1, "all") == 0;
    let all2 = str_cmp(&arg2, "all") == 0;
    if arg1.is_empty() {
        send_to_char(w, "Get what?\n\r", ch);
    } else if arg2.is_empty() && all1 {
        // get all
        let mut found = false;
        let mut fail = false;
        for o in w.room(w[ch].in_room).contents.clone() {
            if w.obj_exists(o) && w.can_see_obj(ch, o) {
                if try_get(w, ch, o, None, false) {
                    found = true;
                } else {
                    fail = true;
                }
            }
        }
        // (the C code printed this even after a successful "get all")
        if !found && !fail {
            send_to_char(w, "You see nothing here.\n\r", ch);
        }
    } else if arg2.is_empty() {
        let room = w[ch].in_room;
        match get_obj_in_list_vis(w, ch, &arg1, &w.room(room).contents) {
            Some(o) => {
                try_get(w, ch, o, None, true);
            }
            None => send_to_char(w, &format!("You do not see a {} here.\n\r", arg1), ch),
        }
    } else if all1 && all2 {
        send_to_char(w, "You must be joking?!\n\r", ch);
    } else if all1 {
        let Some(sub) = find_container(w, ch, &arg2) else { return };
        let mut found = false;
        let mut fail = false;
        for o in w[sub].contains.clone() {
            if w.obj_exists(o) && w.can_see_obj(ch, o) {
                if try_get(w, ch, o, Some(sub), true) {
                    found = true;
                } else {
                    fail = true;
                }
            }
        }
        if !found && !fail {
            let b = format!("You do not see anything in the {}.\n\r", fname(&w[sub].name));
            send_to_char(w, &b, ch);
        }
    } else if all2 {
        send_to_char(w, "You can't take a thing from more than one container.\n\r", ch);
    } else {
        let Some(sub) = find_container(w, ch, &arg2) else { return };
        match get_obj_in_list_vis(w, ch, &arg1, &w[sub].contains) {
            Some(o) => {
                try_get(w, ch, o, Some(sub), true);
            }
            None => {
                let b = format!("The {} does not contain the {}.\n\r", fname(&w[sub].name), arg1);
                send_to_char(w, &b, ch);
            }
        }
    }
}

pub fn do_drop(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (arg, rest) = one_argument(argument);
    if is_number(&arg) {
        let amount = atoi(&arg);
        let (arg, _) = one_argument(rest);
        if str_cmp("coins", &arg) != 0 && str_cmp("coin", &arg) != 0 {
            send_to_char(w, "Sorry, you can't do that (yet)...\n\r", ch);
            return;
        }
        if amount < 0 {
            send_to_char(w, "Sorry, you can't do that!\n\r", ch);
            return;
        }
        if w[ch].points.gold < amount {
            send_to_char(w, "You haven't got that many coins!\n\r", ch);
            return;
        }
        send_to_char(w, "OK.\n\r", ch);
        if amount == 0 {
            return;
        }
        act(w, "$n drops some gold.", false, ch, None, Vict::None, TO_ROOM);
        let m = create_money(w, amount);
        let r = w[ch].in_room;
        obj_to_room(w, m, r);
        w[ch].points.gold -= amount;
        return;
    }
    if arg.is_empty() {
        send_to_char(w, "Drop what?\n\r", ch);
        return;
    }
    if str_cmp(&arg, "all") == 0 {
        let mut test = false;
        for o in w[ch].carrying.clone() {
            if !w.is_obj_stat(o, ITEM_NODROP) {
                if w.can_see_obj(ch, o) {
                    let b = format!("You drop the {}.\n\r", fname(&w[o].name));
                    send_to_char(w, &b, ch);
                } else {
                    send_to_char(w, "You drop something.\n\r", ch);
                }
                act(w, "$n drops $p.", true, ch, Some(o), Vict::None, TO_ROOM);
                obj_from_char(w, o);
                let r = w[ch].in_room;
                obj_to_room(w, o, r);
                test = true;
            } else if w.can_see_obj(ch, o) {
                let b = format!("You can't drop the {}, it must be CURSED!\n\r", fname(&w[o].name));
                send_to_char(w, &b, ch);
                test = true;
            }
        }
        if !test {
            send_to_char(w, "You do not seem to have anything.\n\r", ch);
        }
    } else {
        match get_obj_in_list_vis(w, ch, &arg, &w[ch].carrying) {
            Some(o) => {
                if !w.is_obj_stat(o, ITEM_NODROP) {
                    let b = format!("You drop the {}.\n\r", fname(&w[o].name));
                    send_to_char(w, &b, ch);
                    act(w, "$n drops $p.", true, ch, Some(o), Vict::None, TO_ROOM);
                    obj_from_char(w, o);
                    let r = w[ch].in_room;
                    obj_to_room(w, o, r);
                } else {
                    send_to_char(w, "You can't drop it, it must be CURSED!\n\r", ch);
                }
            }
            None => send_to_char(w, "You do not have that item.\n\r", ch),
        }
    }
}

pub fn do_put(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (arg1, arg2) = argument_interpreter(argument);
    if arg1.is_empty() {
        send_to_char(w, "Put what in what?\n\r", ch);
        return;
    }
    if arg2.is_empty() {
        send_to_char(w, &format!("Put {} in what?\n\r", arg1), ch);
        return;
    }
    let Some(obj) = get_obj_in_list_vis(w, ch, &arg1, &w[ch].carrying) else {
        send_to_char(w, &format!("You dont have the {}.\n\r", arg1), ch);
        return;
    };
    let (bits, _, sub) = generic_find(w, &arg2, FIND_OBJ_INV | FIND_OBJ_ROOM, ch);
    let Some(sub) = sub else {
        send_to_char(w, &format!("You dont have the {}.\n\r", arg2), ch);
        return;
    };
    if w[sub].obj_flags.type_flag != ITEM_CONTAINER {
        let b = format!("The {} is not a container.\n\r", fname(&w[sub].name));
        send_to_char(w, &b, ch);
        return;
    }
    if w[sub].obj_flags.value[1] & CONT_CLOSED != 0 {
        send_to_char(w, "It seems to be closed.\n\r", ch);
        return;
    }
    if obj == sub {
        send_to_char(w, "You attempt to fold it into itself, but fail.\n\r", ch);
        return;
    }
    if w[sub].obj_flags.weight + w[obj].obj_flags.weight < w[sub].obj_flags.value[0] {
        send_to_char(w, "Ok.\n\r", ch);
        obj_from_char(w, obj);
        if bits == FIND_OBJ_INV {
            let wt = w[obj].obj_flags.weight;
            w[ch].specials.carry_weight += wt;
        }
        obj_to_obj(w, obj, sub);
        act(w, "$n puts $p in $s $P", true, ch, Some(obj), Vict::Obj(sub), TO_ROOM);
    } else {
        send_to_char(w, "It won't fit.\n\r", ch);
    }
}

pub fn do_give(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (obj_name, rest) = one_argument(argument);
    if is_number(&obj_name) {
        let amount = atoi(&obj_name);
        let (arg, rest) = one_argument(rest);
        if str_cmp("coins", &arg) != 0 && str_cmp("coin", &arg) != 0 {
            send_to_char(w, "Sorry, you can't do that (yet)...\n\r", ch);
            return;
        }
        if amount < 0 {
            send_to_char(w, "Sorry, you can't do that!\n\r", ch);
            return;
        }
        let mortal = w.is_npc(ch) || w.level(ch) < 22;
        if w[ch].points.gold < amount && mortal {
            send_to_char(w, "You haven't got that many coins!\n\r", ch);
            return;
        }
        let (vict_name, _) = one_argument(rest);
        if vict_name.is_empty() {
            send_to_char(w, "To who?\n\r", ch);
            return;
        }
        let Some(vict) = get_char_room_vis(w, ch, &vict_name) else {
            send_to_char(w, "To who?\n\r", ch);
            return;
        };
        send_to_char(w, "Ok.\n\r", ch);
        let buf = format!("{} gives you {} gold coins.\n\r", w.pers(ch, vict), amount);
        send_to_char(w, &buf, vict);
        act(w, "$n gives some gold to $N.", true, ch, None, Vict::Ch(vict), TO_NOTVICT);
        if mortal {
            w[ch].points.gold -= amount;
        }
        if w.level(ch) > 20 && !w.is_npc(ch) && !w.is_npc(vict) {
            log(&format!("{} gives {} {} gold coins.", w[ch].player.name, w[vict].player.name, amount));
        }
        w[vict].points.gold += amount;
        return;
    }
    let (vict_name, _) = one_argument(rest);
    if obj_name.is_empty() || vict_name.is_empty() {
        send_to_char(w, "Give what to who?\n\r", ch);
        return;
    }
    let Some(obj) = get_obj_in_list_vis(w, ch, &obj_name, &w[ch].carrying) else {
        send_to_char(w, "You do not seem to have anything like that.\n\r", ch);
        return;
    };
    if w.is_obj_stat(obj, ITEM_NODROP) {
        send_to_char(w, "You can't let go of it! Yeech!!\n\r", ch);
        return;
    }
    let Some(vict) = get_char_room_vis(w, ch, &vict_name) else {
        send_to_char(w, "No one by that name around here.\n\r", ch);
        return;
    };
    if 1 + w[vict].specials.carry_items > w.can_carry_n(vict) {
        act(w, "$N seems to have $S hands full.", false, ch, None, Vict::Ch(vict), TO_CHAR);
        return;
    }
    if w[obj].obj_flags.weight + w[vict].specials.carry_weight > w.can_carry_w(vict) {
        act(w, "$E can't carry that much weight.", false, ch, None, Vict::Ch(vict), TO_CHAR);
        return;
    }
    obj_from_char(w, obj);
    obj_to_char(w, obj, vict);
    act(w, "$n gives $p to $N.", true, ch, Some(obj), Vict::Ch(vict), TO_NOTVICT);
    act(w, "$n gives you $p.", false, ch, Some(obj), Vict::Ch(vict), TO_VICT);
    send_to_char(w, "Ok.\n\r", ch);
}
