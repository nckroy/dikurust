//! Special procedures for the world builders Marauder, Fizgig and Maze
//! (mar_fiz_maz.c).

use crate::comm::send_to_char;
use crate::game::World;
use crate::handler::*;
use crate::structs::*;

/// Anyone outside Marauder's zone (rooms below 8000) loses the zone's
/// treasure, whenever someone acts in the gate room.
pub fn mar_gate(w: &mut World, _ch: CharId, _cmd: i32, _arg: &str) -> bool {
    let players: Vec<CharId> = w
        .descriptors
        .iter()
        .filter(|d| d.connected == CON_PLYNG)
        .filter_map(|d| d.character)
        .collect();
    let vnum = |w: &World, o: ObjId| {
        let n = w[o].item_number;
        if n >= 0 {
            w.obj_index[n as usize].virtual_nr
        } else {
            -1
        }
    };
    for c in players {
        if !w.ch_exists(c) || w.is_npc(c) || w[c].in_room < 0 || w.room(w[c].in_room).number >= 8000 {
            continue;
        }
        let mut punished = false;
        for i in 0..MAX_WEAR {
            if let Some(o) = w[c].equipment[i] {
                let j = vnum(w, o);
                if (8500..8999).contains(&j) {
                    if let Some(o) = unequip_char(w, c, i) {
                        w[c].points.gold -= w[o].obj_flags.cost;
                        extract_obj(w, o);
                        punished = true;
                    }
                }
            }
        }
        for o in w[c].carrying.clone() {
            let j = vnum(w, o);
            if (8000..8999).contains(&j) {
                obj_from_char(w, o);
                w[c].points.gold -= w[o].obj_flags.cost;
                extract_obj(w, o);
                punished = true;
            }
        }
        if punished {
            send_to_char(w, "You have been punished by the Gods!\n\r", c);
            let p = &mut w[c].points;
            p.gold = p.gold.max(0);
            p.mov = p.mov.min(10);
            p.mana = 0;
        }
    }
    false
}
