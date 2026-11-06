//! System-to-player communication (the public half of comm.c).
//!
//! Output is queued on the descriptor; the network layer in `net.rs`
//! flushes it to the browser.

#![allow(dead_code)]

use crate::game::World;
use crate::structs::*;
use crate::utils::log;

pub const TO_ROOM: i32 = 0;
pub const TO_VICT: i32 = 1;
pub const TO_NOTVICT: i32 = 2;
pub const TO_CHAR: i32 = 3;

/// The `void *vict_obj` argument of act().
#[derive(Clone, Debug)]
pub enum Vict {
    None,
    Ch(CharId),
    Obj(ObjId),
    Str(String),
}

impl Vict {
    fn ch(&self) -> Option<CharId> {
        if let Vict::Ch(c) = self {
            Some(*c)
        } else {
            None
        }
    }
}

pub fn write_to_q(w: &mut World, d: DescId, txt: &str) {
    if let Some(d) = w.desc_mut(d) {
        d.output.push(txt.to_string());
    }
}

pub fn send_to_char(w: &mut World, messg: &str, ch: CharId) {
    if !w.ch_exists(ch) {
        return;
    }
    if let Some(d) = w[ch].desc {
        write_to_q(w, d, messg);
    }
}

pub fn send_to_all(w: &mut World, messg: &str) {
    for d in w.descriptors.iter_mut() {
        if d.connected == CON_PLYNG {
            d.output.push(messg.to_string());
        }
    }
}

pub fn send_to_outdoor(w: &mut World, messg: &str) {
    let targets: Vec<DescId> = w
        .descriptors
        .iter()
        .filter(|d| d.connected == CON_PLYNG)
        .filter(|d| match d.character {
            Some(c) => w.ch_exists(c) && w[c].in_room >= 0 && w.outside(c),
            None => false,
        })
        .map(|d| d.id)
        .collect();
    for d in targets {
        write_to_q(w, d, messg);
    }
}

pub fn send_to_except(w: &mut World, messg: &str, ch: CharId) {
    let skip = w[ch].desc;
    for d in w.descriptors.iter_mut() {
        if Some(d.id) != skip && d.connected == CON_PLYNG {
            d.output.push(messg.to_string());
        }
    }
}

pub fn send_to_room(w: &mut World, messg: &str, room: i32) {
    let people = w.room(room).people.clone();
    for i in people {
        send_to_char(w, messg, i);
    }
}

pub fn send_to_room_except(w: &mut World, messg: &str, room: i32, ch: CharId) {
    let people = w.room(room).people.clone();
    for i in people {
        if i != ch {
            send_to_char(w, messg, i);
        }
    }
}

pub fn send_to_room_except_two(w: &mut World, messg: &str, room: i32, ch1: CharId, ch2: CharId) {
    let people = w.room(room).people.clone();
    for i in people {
        if i != ch1 && i != ch2 {
            send_to_char(w, messg, i);
        }
    }
}

/// The central message formatter: expands $n, $N, $p ... for each viewer.
pub fn act(w: &mut World, s: &str, hide_invisible: bool, ch: CharId, obj: Option<ObjId>, vict: Vict, typ: i32) {
    if s.is_empty() || !w.ch_exists(ch) {
        return;
    }
    let targets: Vec<CharId> = match typ {
        TO_VICT => match vict.ch() {
            Some(v) => vec![v],
            None => return,
        },
        TO_CHAR => vec![ch],
        _ => {
            if w[ch].in_room < 0 {
                return;
            }
            w.room(w[ch].in_room).people.clone()
        }
    };

    for to in targets {
        if !w.ch_exists(to) {
            continue;
        }
        let ok = w[to].desc.is_some()
            && (to != ch || typ == TO_CHAR)
            && (!hide_invisible || w.can_see(to, ch))
            && w.awake(to)
            && !(typ == TO_NOTVICT && Some(to) == vict.ch());
        if ok {
            let mut buf = String::with_capacity(s.len() + 32);
            let mut it = s.chars();
            while let Some(c) = it.next() {
                if c != '$' {
                    buf.push(c);
                    continue;
                }
                let code = it.next().unwrap_or('$');
                let rep: String = match code {
                    'n' => w.pers(ch, to),
                    'N' => vict.ch().map(|v| w.pers(v, to)).unwrap_or_default(),
                    'm' => w.hmhr(ch).into(),
                    'M' => vict.ch().map(|v| w.hmhr(v).to_string()).unwrap_or_default(),
                    's' => w.hshr(ch).into(),
                    'S' => vict.ch().map(|v| w.hshr(v).to_string()).unwrap_or_default(),
                    'e' => w.hssh(ch).into(),
                    'E' => vict.ch().map(|v| w.hssh(v).to_string()).unwrap_or_default(),
                    'o' => obj.map(|o| w.objn(o, to)).unwrap_or_default(),
                    'O' => match &vict {
                        Vict::Obj(o) => w.objn(*o, to),
                        _ => String::new(),
                    },
                    'p' => obj.map(|o| w.objs(o, to)).unwrap_or_default(),
                    'P' => match &vict {
                        Vict::Obj(o) => w.objs(*o, to),
                        _ => String::new(),
                    },
                    'a' => obj.map(|o| crate::utils::sana(&w[o].name).to_string()).unwrap_or_default(),
                    'A' => match &vict {
                        Vict::Obj(o) => crate::utils::sana(&w[*o].name).to_string(),
                        _ => String::new(),
                    },
                    'T' => match &vict {
                        Vict::Str(t) => t.clone(),
                        _ => String::new(),
                    },
                    'F' => match &vict {
                        Vict::Str(t) => crate::handler::fname(t),
                        _ => String::new(),
                    },
                    '$' => "$".into(),
                    _ => {
                        log("Illegal $-code to act():");
                        log(s);
                        String::new()
                    }
                };
                buf.push_str(&rep);
            }
            buf.push_str("\n\r");
            let buf = crate::utils::cap(&buf);
            if let Some(d) = w[to].desc {
                write_to_q(w, d, &buf);
            }
        }
        if typ == TO_VICT || typ == TO_CHAR {
            return;
        }
    }
}
