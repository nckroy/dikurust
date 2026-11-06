//! Movement, doors, positions and following (act.movement.c).

use crate::act_info::do_look;
use crate::comm::*;
use crate::constants::*;
use crate::fight::death_cry;
use crate::game::World;
use crate::handler::*;
use crate::interpreter::{argument_interpreter, one_argument, search_block, special};
use crate::spell_parser::{add_follower, circle_follow, stop_follower};
use crate::spells::SKILL_PICK_LOCK;
use crate::structs::*;
use crate::utils::*;

/// Returns 1 on success, 0 on failure, -1 if the character died.
pub fn do_simple_move(w: &mut World, ch: CharId, cmd: usize, following: bool) -> i32 {
    if special(w, ch, cmd as i32 + 1, "") {
        return 0;
    }
    if !w.ch_exists(ch) {
        return -1;
    }
    let here = w[ch].in_room;
    let Some(to) = w.room(here).dir_option[cmd].as_ref().map(|e| e.to_room) else { return 0 };
    if to < 0 {
        return 0;
    }
    let need_movement =
        (MOVEMENT_LOSS[w.room(here).sector_type.clamp(0, 7) as usize] + MOVEMENT_LOSS[w.room(to).sector_type.clamp(0, 7) as usize]) / 2;

    if w.room(here).sector_type == SECT_WATER_NOSWIM || w.room(to).sector_type == SECT_WATER_NOSWIM {
        let has_boat = w[ch].carrying.iter().any(|&o| w[o].obj_flags.type_flag == ITEM_BOAT);
        if !has_boat {
            send_to_char(w, "You need a boat to go there.\n\r", ch);
            return 0;
        }
    }
    if w[ch].points.mov < need_movement && !w.is_npc(ch) {
        if !following {
            send_to_char(w, "You are too exhausted.\n\r", ch);
        } else {
            send_to_char(w, "You are too exhausted to follow.\n\r", ch);
        }
        return 0;
    }
    if w.level(ch) < 21 && !w.is_npc(ch) {
        w[ch].points.mov -= need_movement;
    }
    if !w.is_affected(ch, AFF_SNEAK) {
        let tmp = format!("$n leaves {}.", DIRS[cmd]);
        act(w, &tmp, true, ch, None, Vict::None, TO_ROOM);
    }
    char_from_room(w, ch);
    char_to_room(w, ch, to);
    if !w.is_affected(ch, AFF_SNEAK) {
        act(w, "$n has arrived.", true, ch, None, Vict::None, TO_ROOM);
    }
    do_look(w, ch, "", 15);
    if w.room(to).room_flags & DEATH != 0 && w.level(ch) < 21 {
        death_cry(w, ch);
        extract_char(w, ch);
        return -1;
    }
    1
}

pub fn do_move(w: &mut World, ch: CharId, argument: &str, cmd: i32) {
    let cmd = (cmd - 1).clamp(0, 5) as usize;
    let Some(exit) = w.exit(ch, cmd).cloned() else {
        send_to_char(w, "Alas, you cannot go that way...\n\r", ch);
        return;
    };
    if exit.exit_info & EX_CLOSED != 0 {
        if !exit.keyword.is_empty() {
            let tmp = format!("The {} seems to be closed.\n\r", fname(&exit.keyword));
            send_to_char(w, &tmp, ch);
        } else {
            send_to_char(w, "It seems to be closed.\n\r", ch);
        }
    } else if exit.to_room == NOWHERE {
        send_to_char(w, "Alas, you can't go that way.\n\r", ch);
    } else if w[ch].followers.is_empty() && w[ch].master.is_none() {
        do_simple_move(w, ch, cmd, false);
    } else if w.is_affected(ch, AFF_CHARM)
        && w[ch].master.map(|m| w.ch_exists(m) && w[m].in_room == w[ch].in_room).unwrap_or(false)
    {
        send_to_char(w, "The thought of leaving your master makes you weep.\n\r", ch);
        act(w, "$n bursts into tears.", false, ch, None, Vict::None, TO_ROOM);
    } else {
        let was_in = w[ch].in_room;
        if do_simple_move(w, ch, cmd, true) == 1 {
            for k in w[ch].followers.clone() {
                if !w.ch_exists(k) || !w.ch_exists(ch) {
                    continue;
                }
                if was_in == w[k].in_room && w.get_pos(k) >= POSITION_STANDING {
                    act(w, "You follow $N.", false, k, None, Vict::Ch(ch), TO_CHAR);
                    send_to_char(w, "\n\r", k);
                    do_move(w, k, argument, cmd as i32 + 1);
                }
            }
        }
    }
}

pub fn find_door(w: &mut World, ch: CharId, typ: &str, dir: &str) -> i32 {
    if !dir.is_empty() {
        let door = search_block(dir, DIRS, false);
        if door == -1 {
            send_to_char(w, "That's not a direction.\n\r", ch);
            return -1;
        }
        match w.exit(ch, door as usize) {
            Some(e) => {
                if !e.keyword.is_empty() {
                    if isname(typ, &e.keyword) {
                        door
                    } else {
                        let buf = format!("I see no {} there.\n\r", typ);
                        send_to_char(w, &buf, ch);
                        -1
                    }
                } else {
                    door
                }
            }
            None => {
                send_to_char(w, "I really don't see how you can close anything there.\n\r", ch);
                -1
            }
        }
    } else {
        for door in 0..6 {
            if let Some(e) = w.exit(ch, door) {
                if !e.keyword.is_empty() && isname(typ, &e.keyword) {
                    return door as i32;
                }
            }
        }
        let buf = format!("I see no {} here.\n\r", typ);
        send_to_char(w, &buf, ch);
        -1
    }
}

/// The exit on the other side of `door`, if it leads back here.
fn back_exit(w: &World, ch: CharId, door: usize) -> Option<(i32, usize)> {
    let other = w.exit(ch, door)?.to_room;
    if other == NOWHERE {
        return None;
    }
    let rd = REV_DIR[door] as usize;
    let back = w.room(other).dir_option[rd].as_ref()?;
    if back.to_room == w[ch].in_room {
        Some((other, rd))
    } else {
        None
    }
}

fn exit_info(w: &World, ch: CharId, door: usize) -> i32 {
    w.exit(ch, door).map(|e| e.exit_info).unwrap_or(0)
}

fn exit_keyword(w: &World, ch: CharId, door: usize) -> String {
    w.exit(ch, door).map(|e| e.keyword.clone()).unwrap_or_default()
}

fn find_container(w: &World, ch: CharId, argument: &str) -> Option<ObjId> {
    let (bits, _, obj) = generic_find(w, argument, FIND_OBJ_INV | FIND_OBJ_ROOM, ch);
    if bits != 0 {
        obj
    } else {
        None
    }
}

pub fn do_open(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (typ, dir) = argument_interpreter(argument);
    if typ.is_empty() {
        send_to_char(w, "Open what?\n\r", ch);
    } else if let Some(obj) = find_container(w, ch, argument) {
        let v1 = w[obj].obj_flags.value[1];
        if w[obj].obj_flags.type_flag != ITEM_CONTAINER {
            send_to_char(w, "That's not a container.\n\r", ch);
        } else if v1 & CONT_CLOSED == 0 {
            send_to_char(w, "But it's already open!\n\r", ch);
        } else if v1 & CONT_CLOSEABLE == 0 {
            send_to_char(w, "You can't do that.\n\r", ch);
        } else if v1 & CONT_LOCKED != 0 {
            send_to_char(w, "It seems to be locked.\n\r", ch);
        } else {
            w[obj].obj_flags.value[1] &= !CONT_CLOSED;
            send_to_char(w, "Ok.\n\r", ch);
            act(w, "$n opens $p.", false, ch, Some(obj), Vict::None, TO_ROOM);
        }
    } else {
        let door = find_door(w, ch, &typ, &dir);
        if door < 0 {
            return;
        }
        let d = door as usize;
        let info = exit_info(w, ch, d);
        if info & EX_ISDOOR == 0 {
            send_to_char(w, "That's impossible, I'm afraid.\n\r", ch);
        } else if info & EX_CLOSED == 0 {
            send_to_char(w, "It's already open!\n\r", ch);
        } else if info & EX_LOCKED != 0 {
            send_to_char(w, "It seems to be locked.\n\r", ch);
        } else {
            w.exit_mut(ch, d).unwrap().exit_info &= !EX_CLOSED;
            let kw = exit_keyword(w, ch, d);
            if !kw.is_empty() {
                act(w, "$n opens the $F.", false, ch, None, Vict::Str(kw), TO_ROOM);
            } else {
                act(w, "$n opens the door.", false, ch, None, Vict::None, TO_ROOM);
            }
            send_to_char(w, "Ok.\n\r", ch);
            if let Some((other, rd)) = back_exit(w, ch, d) {
                let back = w.room_mut(other).dir_option[rd].as_mut().unwrap();
                back.exit_info &= !EX_CLOSED;
                let msg = if !back.keyword.is_empty() {
                    format!("The {} is opened from the other side.\n\r", fname(&back.keyword))
                } else {
                    "The door is opened from the other side.\n\r".to_string()
                };
                send_to_room(w, &msg, other);
            }
        }
    }
}

pub fn do_close(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (typ, dir) = argument_interpreter(argument);
    if typ.is_empty() {
        send_to_char(w, "Close what?\n\r", ch);
    } else if let Some(obj) = find_container(w, ch, argument) {
        let v1 = w[obj].obj_flags.value[1];
        if w[obj].obj_flags.type_flag != ITEM_CONTAINER {
            send_to_char(w, "That's not a container.\n\r", ch);
        } else if v1 & CONT_CLOSED != 0 {
            send_to_char(w, "But it's already closed!\n\r", ch);
        } else if v1 & CONT_CLOSEABLE == 0 {
            send_to_char(w, "That's impossible.\n\r", ch);
        } else {
            w[obj].obj_flags.value[1] |= CONT_CLOSED;
            send_to_char(w, "Ok.\n\r", ch);
            act(w, "$n closes $p.", false, ch, Some(obj), Vict::None, TO_ROOM);
        }
    } else {
        let door = find_door(w, ch, &typ, &dir);
        if door < 0 {
            return;
        }
        let d = door as usize;
        let info = exit_info(w, ch, d);
        if info & EX_ISDOOR == 0 {
            send_to_char(w, "That's absurd.\n\r", ch);
        } else if info & EX_CLOSED != 0 {
            send_to_char(w, "It's already closed!\n\r", ch);
        } else {
            w.exit_mut(ch, d).unwrap().exit_info |= EX_CLOSED;
            let kw = exit_keyword(w, ch, d);
            if !kw.is_empty() {
                act(w, "$n closes the $F.", false, ch, None, Vict::Str(kw), TO_ROOM);
            } else {
                act(w, "$n closes the door.", false, ch, None, Vict::None, TO_ROOM);
            }
            send_to_char(w, "Ok.\n\r", ch);
            if let Some((other, rd)) = back_exit(w, ch, d) {
                let back = w.room_mut(other).dir_option[rd].as_mut().unwrap();
                back.exit_info |= EX_CLOSED;
                let msg = if !back.keyword.is_empty() {
                    format!("The {} closes quietly.\n\r", back.keyword)
                } else {
                    "The door closes quietly.\n\r".to_string()
                };
                send_to_room(w, &msg, other);
            }
        }
    }
}

pub fn has_key(w: &World, ch: CharId, key: i32) -> bool {
    let vnum = |o: ObjId| {
        let n = w[o].item_number;
        if n >= 0 {
            w.obj_index[n as usize].virtual_nr
        } else {
            -1
        }
    };
    if w[ch].carrying.iter().any(|&o| vnum(o) == key) {
        return true;
    }
    matches!(w[ch].equipment[HOLD], Some(o) if vnum(o) == key)
}

fn set_back_lock(w: &mut World, ch: CharId, d: usize, lock: bool) {
    if let Some((other, rd)) = back_exit(w, ch, d) {
        let back = w.room_mut(other).dir_option[rd].as_mut().unwrap();
        if lock {
            back.exit_info |= EX_LOCKED;
        } else {
            back.exit_info &= !EX_LOCKED;
        }
    }
}

fn exit_key(w: &World, ch: CharId, d: usize) -> i32 {
    w.exit(ch, d).map(|e| e.key).unwrap_or(-1)
}

pub fn do_lock(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (typ, dir) = argument_interpreter(argument);
    if typ.is_empty() {
        send_to_char(w, "Lock what?\n\r", ch);
    } else if let Some(obj) = find_container(w, ch, argument) {
        let v = w[obj].obj_flags.value;
        if w[obj].obj_flags.type_flag != ITEM_CONTAINER {
            send_to_char(w, "That's not a container.\n\r", ch);
        } else if v[1] & CONT_CLOSED == 0 {
            send_to_char(w, "Maybe you should close it first...\n\r", ch);
        } else if v[2] < 0 {
            send_to_char(w, "That thing can't be locked.\n\r", ch);
        } else if !has_key(w, ch, v[2]) {
            send_to_char(w, "You don't seem to have the proper key.\n\r", ch);
        } else if v[1] & CONT_LOCKED != 0 {
            send_to_char(w, "It is locked already.\n\r", ch);
        } else {
            w[obj].obj_flags.value[1] |= CONT_LOCKED;
            send_to_char(w, "*Cluck*\n\r", ch);
            act(w, "$n locks $p - 'cluck', it says.", false, ch, Some(obj), Vict::None, TO_ROOM);
        }
    } else {
        let door = find_door(w, ch, &typ, &dir);
        if door < 0 {
            return;
        }
        let d = door as usize;
        let info = exit_info(w, ch, d);
        let key = exit_key(w, ch, d);
        if info & EX_ISDOOR == 0 {
            send_to_char(w, "That's absurd.\n\r", ch);
        } else if info & EX_CLOSED == 0 {
            send_to_char(w, "You have to close it first, I'm afraid.\n\r", ch);
        } else if key < 0 {
            send_to_char(w, "There does not seem to be any keyholes.\n\r", ch);
        } else if !has_key(w, ch, key) {
            send_to_char(w, "You don't have the proper key.\n\r", ch);
        } else if info & EX_LOCKED != 0 {
            send_to_char(w, "It's already locked!\n\r", ch);
        } else {
            w.exit_mut(ch, d).unwrap().exit_info |= EX_LOCKED;
            let kw = exit_keyword(w, ch, d);
            if !kw.is_empty() {
                act(w, "$n locks the $F.", false, ch, None, Vict::Str(kw), TO_ROOM);
            } else {
                act(w, "$n locks the door.", false, ch, None, Vict::None, TO_ROOM);
            }
            send_to_char(w, "*Click*\n\r", ch);
            set_back_lock(w, ch, d, true);
        }
    }
}

pub fn do_unlock(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (typ, dir) = argument_interpreter(argument);
    if typ.is_empty() {
        send_to_char(w, "Unlock what?\n\r", ch);
    } else if let Some(obj) = find_container(w, ch, argument) {
        let v = w[obj].obj_flags.value;
        if w[obj].obj_flags.type_flag != ITEM_CONTAINER {
            send_to_char(w, "That's not a container.\n\r", ch);
        } else if v[1] & CONT_CLOSED == 0 {
            send_to_char(w, "Silly - it ain't even closed!\n\r", ch);
        } else if v[2] < 0 {
            send_to_char(w, "Odd - you can't seem to find a keyhole.\n\r", ch);
        } else if !has_key(w, ch, v[2]) {
            send_to_char(w, "You don't seem to have the proper key.\n\r", ch);
        } else if v[1] & CONT_LOCKED == 0 {
            send_to_char(w, "Oh.. it wasn't locked, after all.\n\r", ch);
        } else {
            w[obj].obj_flags.value[1] &= !CONT_LOCKED;
            send_to_char(w, "*Click*\n\r", ch);
            act(w, "$n unlocks $p.", false, ch, Some(obj), Vict::None, TO_ROOM);
        }
    } else {
        let door = find_door(w, ch, &typ, &dir);
        if door < 0 {
            return;
        }
        let d = door as usize;
        let info = exit_info(w, ch, d);
        let key = exit_key(w, ch, d);
        if info & EX_ISDOOR == 0 {
            send_to_char(w, "That's absurd.\n\r", ch);
        } else if info & EX_CLOSED == 0 {
            send_to_char(w, "Heck.. it ain't even closed!\n\r", ch);
        } else if key < 0 {
            send_to_char(w, "You can't seem to spot any keyholes.\n\r", ch);
        } else if !has_key(w, ch, key) {
            send_to_char(w, "You do not have the proper key for that.\n\r", ch);
        } else if info & EX_LOCKED == 0 {
            send_to_char(w, "It's already unlocked, it seems.\n\r", ch);
        } else {
            w.exit_mut(ch, d).unwrap().exit_info &= !EX_LOCKED;
            let kw = exit_keyword(w, ch, d);
            if !kw.is_empty() {
                act(w, "$n unlocks the $F.", false, ch, None, Vict::Str(kw), TO_ROOM);
            } else {
                act(w, "$n unlocks the door.", false, ch, None, Vict::None, TO_ROOM);
            }
            send_to_char(w, "*click*\n\r", ch);
            set_back_lock(w, ch, d, false);
        }
    }
}

pub fn do_pick(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (typ, dir) = argument_interpreter(argument);
    let percent = number(1, 101);
    if percent > w[ch].skills[SKILL_PICK_LOCK as usize].learned {
        send_to_char(w, "You failed to pick the lock.\n\r", ch);
        return;
    }
    if typ.is_empty() {
        send_to_char(w, "Pick what?\n\r", ch);
    } else if let Some(obj) = find_container(w, ch, argument) {
        let v = w[obj].obj_flags.value;
        if w[obj].obj_flags.type_flag != ITEM_CONTAINER {
            send_to_char(w, "That's not a container.\n\r", ch);
        } else if v[1] & CONT_CLOSED == 0 {
            send_to_char(w, "Silly - it ain't even closed!\n\r", ch);
        } else if v[2] < 0 {
            send_to_char(w, "Odd - you can't seem to find a keyhole.\n\r", ch);
        } else if v[1] & CONT_LOCKED == 0 {
            send_to_char(w, "Oho! This thing is NOT locked!\n\r", ch);
        } else if v[1] & CONT_PICKPROOF != 0 {
            send_to_char(w, "It resists your attempts at picking it.\n\r", ch);
        } else {
            w[obj].obj_flags.value[1] &= !CONT_LOCKED;
            send_to_char(w, "*Click*\n\r", ch);
            act(w, "$n fiddles with $p.", false, ch, Some(obj), Vict::None, TO_ROOM);
        }
    } else {
        let door = find_door(w, ch, &typ, &dir);
        if door < 0 {
            return;
        }
        let d = door as usize;
        let info = exit_info(w, ch, d);
        let key = exit_key(w, ch, d);
        if info & EX_ISDOOR == 0 {
            send_to_char(w, "That's absurd.\n\r", ch);
        } else if info & EX_CLOSED == 0 {
            send_to_char(w, "You realize that the door is already open.\n\r", ch);
        } else if key < 0 {
            send_to_char(w, "You can't seem to spot any lock to pick.\n\r", ch);
        } else if info & EX_LOCKED == 0 {
            send_to_char(w, "Oh.. it wasn't locked at all.\n\r", ch);
        } else if info & EX_PICKPROOF != 0 {
            send_to_char(w, "You seem to be unable to pick this lock.\n\r", ch);
        } else {
            w.exit_mut(ch, d).unwrap().exit_info &= !EX_LOCKED;
            let kw = exit_keyword(w, ch, d);
            if !kw.is_empty() {
                act(w, "$n skillfully picks the lock of the $F.", false, ch, None, Vict::Str(kw), TO_ROOM);
            } else {
                act(w, "$n picks the lock of the.", true, ch, None, Vict::None, TO_ROOM);
            }
            send_to_char(w, "The lock quickly yields to your skills.\n\r", ch);
            set_back_lock(w, ch, d, false);
        }
    }
}

pub fn do_enter(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (buf, _) = one_argument(argument);
    if !buf.is_empty() {
        for door in 0..6 {
            if let Some(e) = w.exit(ch, door) {
                if !e.keyword.is_empty() && str_cmp(&e.keyword, &buf) == 0 {
                    do_move(w, ch, "", door as i32 + 1);
                    return;
                }
            }
        }
        let tmp = format!("There is no {} here.\n\r", buf);
        send_to_char(w, &tmp, ch);
    } else if w.room(w[ch].in_room).room_flags & INDOORS != 0 {
        send_to_char(w, "You are already indoors.\n\r", ch);
    } else {
        for door in 0..6 {
            if let Some(e) = w.exit(ch, door) {
                if e.to_room != NOWHERE && e.exit_info & EX_CLOSED == 0 && w.room(e.to_room).room_flags & INDOORS != 0 {
                    do_move(w, ch, "", door as i32 + 1);
                    return;
                }
            }
        }
        send_to_char(w, "You can't seem to find anything to enter.\n\r", ch);
    }
}

pub fn do_leave(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    if w.room(w[ch].in_room).room_flags & INDOORS == 0 {
        send_to_char(w, "You are outside.. where do you want to go?\n\r", ch);
        return;
    }
    for door in 0..6 {
        if let Some(e) = w.exit(ch, door) {
            if e.to_room != NOWHERE && e.exit_info & EX_CLOSED == 0 && w.room(e.to_room).room_flags & INDOORS == 0 {
                do_move(w, ch, "", door as i32 + 1);
                return;
            }
        }
    }
    send_to_char(w, "I see no obvious exits to the outside.\n\r", ch);
}

fn me(w: &mut World, ch: CharId, s: &str) {
    act(w, s, false, ch, None, Vict::None, TO_CHAR);
}
fn room(w: &mut World, ch: CharId, s: &str, hide: bool) {
    act(w, s, hide, ch, None, Vict::None, TO_ROOM);
}

pub fn do_stand(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    match w.get_pos(ch) {
        POSITION_STANDING => me(w, ch, "You are already standing."),
        POSITION_SITTING => {
            me(w, ch, "You stand up.");
            room(w, ch, "$n clambers on $s feet.", true);
            w[ch].specials.position = POSITION_STANDING;
        }
        POSITION_RESTING => {
            me(w, ch, "You stop resting, and stand up.");
            room(w, ch, "$n stops resting, and clambers on $s feet.", true);
            w[ch].specials.position = POSITION_STANDING;
        }
        POSITION_SLEEPING => me(w, ch, "You have to wake up first!"),
        POSITION_FIGHTING => me(w, ch, "Do you not consider fighting as standing?"),
        _ => {
            me(w, ch, "You stop floating around, and put your feet on the ground.");
            room(w, ch, "$n stops floating around, and puts $s feet on the ground.", true);
        }
    }
}

pub fn do_sit(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    match w.get_pos(ch) {
        POSITION_STANDING => {
            me(w, ch, "You sit down.");
            room(w, ch, "$n sits down.", false);
            w[ch].specials.position = POSITION_SITTING;
        }
        POSITION_SITTING => send_to_char(w, "You'r sitting already.\n\r", ch),
        POSITION_RESTING => {
            me(w, ch, "You stop resting, and sit up.");
            room(w, ch, "$n stops resting.", true);
            w[ch].specials.position = POSITION_SITTING;
        }
        POSITION_SLEEPING => me(w, ch, "You have to wake up first."),
        POSITION_FIGHTING => me(w, ch, "Sit down while fighting? are you MAD?"),
        _ => {
            me(w, ch, "You stop floating around, and sit down.");
            room(w, ch, "$n stops floating around, and sits down.", true);
            w[ch].specials.position = POSITION_SITTING;
        }
    }
}

pub fn do_rest(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    match w.get_pos(ch) {
        POSITION_STANDING => {
            me(w, ch, "You sit down and rest your tired bones.");
            room(w, ch, "$n sits down and rests.", true);
            w[ch].specials.position = POSITION_RESTING;
        }
        POSITION_SITTING => {
            me(w, ch, "You rest your tired bones.");
            room(w, ch, "$n rests.", true);
            w[ch].specials.position = POSITION_RESTING;
        }
        POSITION_RESTING => me(w, ch, "You are already resting."),
        POSITION_SLEEPING => me(w, ch, "You have to wake up first."),
        POSITION_FIGHTING => me(w, ch, "Rest while fighting? are you MAD?"),
        _ => {
            me(w, ch, "You stop floating around, and stop to rest your tired bones.");
            room(w, ch, "$n stops floating around, and rests.", false);
            w[ch].specials.position = POSITION_SITTING;
        }
    }
}

pub fn do_sleep(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    match w.get_pos(ch) {
        POSITION_STANDING | POSITION_SITTING | POSITION_RESTING => {
            send_to_char(w, "You go to sleep.\n\r", ch);
            room(w, ch, "$n lies down and falls asleep.", true);
            w[ch].specials.position = POSITION_SLEEPING;
        }
        POSITION_SLEEPING => send_to_char(w, "You are already sound asleep.\n\r", ch),
        POSITION_FIGHTING => send_to_char(w, "Sleep while fighting? are you MAD?\n\r", ch),
        _ => {
            me(w, ch, "You stop floating around, and lie down to sleep.");
            room(w, ch, "$n stops floating around, and lie down to sleep.", true);
            w[ch].specials.position = POSITION_SLEEPING;
        }
    }
}

pub fn do_wake(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (arg, _) = one_argument(argument);
    if !arg.is_empty() {
        if w.get_pos(ch) == POSITION_SLEEPING {
            me(w, ch, "You can't wake people up if you are asleep yourself!");
            return;
        }
        match get_char_room_vis(w, ch, &arg) {
            Some(t) if t == ch => me(w, ch, "If you want to wake yourself up, just type 'wake'"),
            Some(t) => {
                if w.get_pos(t) == POSITION_SLEEPING {
                    if w.is_affected(t, AFF_SLEEP) {
                        act(w, "You can not wake $M up!", false, ch, None, Vict::Ch(t), TO_CHAR);
                    } else {
                        act(w, "You wake $M up.", false, ch, None, Vict::Ch(t), TO_CHAR);
                        w[t].specials.position = POSITION_SITTING;
                        act(w, "You are awakened by $n.", false, ch, None, Vict::Ch(t), TO_VICT);
                    }
                } else {
                    act(w, "$N is already awake.", false, ch, None, Vict::Ch(t), TO_CHAR);
                }
            }
            None => send_to_char(w, "You do not see that person here.\n\r", ch),
        }
    } else if w.is_affected(ch, AFF_SLEEP) {
        send_to_char(w, "You can't wake up!\n\r", ch);
    } else if w.get_pos(ch) > POSITION_SLEEPING {
        send_to_char(w, "You are already awake...\n\r", ch);
    } else {
        send_to_char(w, "You wake, and sit up.\n\r", ch);
        room(w, ch, "$n awakens.", true);
        w[ch].specials.position = POSITION_SITTING;
    }
}

pub fn do_follow(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (name, _) = one_argument(argument);
    if name.is_empty() {
        send_to_char(w, "Who do you wish to follow?\n\r", ch);
        return;
    }
    let Some(leader) = get_char_room_vis(w, ch, &name) else {
        send_to_char(w, "I see no person by that name here!\n\r", ch);
        return;
    };
    if w.is_affected(ch, AFF_CHARM) && w[ch].master.is_some() {
        let m = w[ch].master.unwrap();
        act(w, "But you only feel like following $N!", false, ch, None, Vict::Ch(m), TO_CHAR);
    } else if leader == ch {
        if w[ch].master.is_none() {
            send_to_char(w, "You are already following yourself.\n\r", ch);
            return;
        }
        stop_follower(w, ch);
    } else {
        if circle_follow(w, ch, leader) {
            me(w, ch, "Sorry, but following in 'loops' is not allowed");
            return;
        }
        if w[ch].master.is_some() {
            stop_follower(w, ch);
        }
        add_follower(w, ch, leader);
    }
}
