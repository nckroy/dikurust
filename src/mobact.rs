//! Mobile AI (mobact.c).

use crate::act_move::do_move;
use crate::comm::*;
use crate::fight::hit;
use crate::game::World;
use crate::handler::*;
use crate::structs::*;
use crate::utils::*;

pub fn mobile_activity(w: &mut World) {
    for ch in w.character_list.clone() {
        if !w.ch_exists(ch) || !w.is_mob(ch) || w[ch].in_room < 0 {
            continue;
        }
        if w[ch].specials.act & ACT_SPEC != 0 && !w.no_specials {
            let nr = w[ch].nr as usize;
            match w.mob_index[nr].func {
                None => {
                    log(&format!(
                        "Non-Existing MOB[{}] SPEC procedure (mobact.c)",
                        w.mob_index[nr].virtual_nr
                    ));
                    w[ch].specials.act &= !ACT_SPEC;
                }
                Some(f) => {
                    if f(w, ch, 0, "") {
                        continue;
                    }
                }
            }
        }
        if !w.ch_exists(ch) || w[ch].in_room < 0 {
            continue;
        }

        if !(w.awake(ch) && w[ch].specials.fighting.is_none()) {
            continue;
        }

        if w[ch].specials.act & ACT_SCAVENGER != 0 {
            let room = w[ch].in_room;
            if !w.room(room).contents.is_empty() && number(0, 10) == 0 {
                let mut max = 1;
                let mut best = None;
                for o in w.room(room).contents.clone() {
                    if w.can_get_obj(ch, o) && w[o].obj_flags.cost > max {
                        best = Some(o);
                        max = w[o].obj_flags.cost;
                    }
                }
                if let Some(b) = best {
                    obj_from_room(w, b);
                    obj_to_char(w, b, ch);
                    act(w, "$n gets $p.", false, ch, Some(b), Vict::None, TO_ROOM);
                }
            }
        }

        if w[ch].specials.act & ACT_SENTINEL == 0 && w.get_pos(ch) == POSITION_STANDING {
            let door = number(0, 45);
            if door <= 5 && w.can_go(ch, door as usize) {
                let to = w.exit(ch, door as usize).unwrap().to_room;
                let flags = w.room(to).room_flags;
                if flags & NO_MOB == 0 && flags & DEATH == 0 {
                    if w[ch].specials.last_direction == door {
                        w[ch].specials.last_direction = -1;
                    } else if w[ch].specials.act & ACT_STAY_ZONE == 0
                        || w.room(to).zone == w.room(w[ch].in_room).zone
                    {
                        w[ch].specials.last_direction = door;
                        do_move(w, ch, "", door + 1);
                    }
                }
            }
        }
        if !w.ch_exists(ch) || w[ch].in_room < 0 {
            continue;
        }

        if w[ch].specials.act & ACT_AGGRESSIVE != 0 {
            for t in w.room(w[ch].in_room).people.clone() {
                if w.ch_exists(t) && !w.is_npc(t) && w.can_see(ch, t) {
                    if w[ch].specials.act & ACT_WIMPY == 0 || !w.awake(t) {
                        hit(w, ch, t, 0);
                        break;
                    }
                }
            }
        }
    }
}
