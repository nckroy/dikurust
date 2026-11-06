//! Run-time string editing, paging and the reboot check (modify.c).

use crate::comm::*;
use crate::game::World;
use crate::handler::*;
use crate::interpreter::{is_abbrev, one_argument};
use crate::structs::*;
use crate::utils::*;

const STRING_FIELDS: [&str; 6] = ["name", "short", "long", "description", "title", "delete-description"];
const LENGTH: [usize; 5] = [15, 60, 256, 240, 60];

/// Resolve a string-editor target to the string it edits.
fn target_mut<'a>(w: &'a mut World, t: &StrTarget) -> Option<&'a mut String> {
    match t {
        StrTarget::CharDescription(c) => w.chars.get_mut(*c).map(|c| &mut c.player.description),
        StrTarget::CharTitle(c) => w.chars.get_mut(*c).map(|c| &mut c.player.title),
        StrTarget::CharName(c) => w.chars.get_mut(*c).map(|c| &mut c.player.name),
        StrTarget::CharShort(c) => w.chars.get_mut(*c).map(|c| &mut c.player.short_descr),
        StrTarget::CharLong(c) => w.chars.get_mut(*c).map(|c| &mut c.player.long_descr),
        StrTarget::ObjName(o) => w.objs.get_mut(*o).map(|o| &mut o.name),
        StrTarget::ObjShort(o) => w.objs.get_mut(*o).map(|o| &mut o.short_description),
        StrTarget::ObjLong(o) => w.objs.get_mut(*o).map(|o| &mut o.description),
        StrTarget::ObjAction(o) | StrTarget::Note(o) => w.objs.get_mut(*o).map(|o| &mut o.action_description),
        StrTarget::ObjExtra(o, kw) => {
            let o = w.objs.get_mut(*o)?;
            o.ex_description.iter_mut().find(|e| e.keyword == *kw).map(|e| &mut e.description)
        }
        StrTarget::Board(i) => w.board.msgs.get_mut(*i),
    }
}

/// Add user input to the string being edited.
pub fn string_add(w: &mut World, did: DescId, input: &str) {
    let Some(d) = w.desc(did) else { return };
    let Some(target) = d.str_target.clone() else { return };
    let max_str = d.max_str;
    let ch = d.character;

    let mut s = input.to_string();
    let mut terminator = false;
    if let Some(p) = s.find('@') {
        s.truncate(p);
        terminator = true;
    }
    let mut notice: Option<&str> = None;
    match target_mut(w, &target) {
        None => terminator = true,
        Some(cur) => {
            if cur.is_empty() {
                if s.len() > max_str {
                    notice = Some("String too long - Truncated.\n\r");
                    s.truncate(max_str);
                    terminator = true;
                }
                *cur = s;
            } else if s.len() + cur.len() > max_str {
                notice = Some("String too long. Last line skipped.\n\r");
                terminator = true;
            } else {
                cur.push_str(&s);
            }
            if !terminator {
                cur.push_str("\n\r");
            }
        }
    }
    if let (Some(n), Some(c)) = (notice, ch) {
        send_to_char(w, n, c);
    }
    if terminator {
        if let StrTarget::Board(_) = target {
            crate::board::board_save(w);
        }
        if let Some(d) = w.desc_mut(did) {
            d.str_target = None;
            if d.connected == CON_EXDSCR {
                d.output.push(crate::db::MENU.to_string());
                d.connected = CON_SLCT;
            }
        }
    }
}

pub fn do_string(w: &mut World, ch: CharId, arg: &str, _cmd: i32) {
    if w.is_npc(ch) {
        return;
    }
    let Some(did) = w[ch].desc else { return };
    // quad_arg()
    let (kind, rest) = one_argument(arg);
    let typ = if is_abbrev(&kind, "char") {
        0
    } else if is_abbrev(&kind, "obj") {
        1
    } else {
        send_to_char(w, "Syntax:\n\rstring ('obj'|'char') <name> <field> [<string>].", ch);
        return;
    };
    let (name, rest) = one_argument(rest);
    let (fieldname, rest) = one_argument(rest);
    let field = if fieldname.is_empty() {
        0
    } else {
        STRING_FIELDS.iter().position(|f| f.starts_with(&fieldname)).map(|p| p + 1).unwrap_or(0)
    };
    let mut string = rest.trim_start().to_string();
    if field == 0 {
        send_to_char(w, "No field by that name. Try 'help string'.\n\r", ch);
        return;
    }

    let target = if typ == 0 {
        let Some(mob) = get_char_vis(w, ch, &name) else {
            send_to_char(w, "I don't know anyone by that name...\n\r", ch);
            return;
        };
        match field {
            1 => {
                if !w.is_npc(mob) && w.level(ch) < 24 {
                    send_to_char(w, "You can't change that field for players.", ch);
                    return;
                }
                if !w.is_npc(mob) {
                    send_to_char(w, "WARNING: You have changed the name of a player.\n\r", ch);
                }
                StrTarget::CharName(mob)
            }
            2 | 3 => {
                if !w.is_npc(mob) {
                    send_to_char(w, "That field is for monsters only.\n\r", ch);
                    return;
                }
                if field == 2 {
                    StrTarget::CharShort(mob)
                } else {
                    StrTarget::CharLong(mob)
                }
            }
            4 => StrTarget::CharDescription(mob),
            5 => {
                if w.is_npc(mob) {
                    send_to_char(w, "Monsters have no titles.\n\r", ch);
                    return;
                }
                StrTarget::CharTitle(mob)
            }
            _ => {
                send_to_char(w, "That field is undefined for monsters.\n\r", ch);
                return;
            }
        }
    } else {
        let Some(obj) = get_obj_vis(w, ch, &name) else {
            send_to_char(w, "Can't find such a thing here..\n\r", ch);
            return;
        };
        match field {
            1 => StrTarget::ObjName(obj),
            2 => StrTarget::ObjShort(obj),
            3 => StrTarget::ObjLong(obj),
            4 => {
                if string.is_empty() {
                    send_to_char(w, "You have to supply a keyword.\n\r", ch);
                    return;
                }
                if let Some(ed) = w[obj].ex_description.iter_mut().find(|e| str_cmp(&e.keyword, &string) == 0) {
                    ed.description.clear();
                    let kw = ed.keyword.clone();
                    send_to_char(w, "Modifying description.\n\r", ch);
                    if let Some(d) = w.desc_mut(did) {
                        d.str_target = Some(StrTarget::ObjExtra(obj, kw));
                        d.max_str = MAX_STRING_LENGTH;
                    }
                } else {
                    w[obj].ex_description.insert(0, ExtraDescr { keyword: string.clone(), description: String::new() });
                    send_to_char(w, "New field.\n\r", ch);
                    if let Some(d) = w.desc_mut(did) {
                        d.str_target = Some(StrTarget::ObjExtra(obj, string));
                        d.max_str = MAX_STRING_LENGTH;
                    }
                }
                return;
            }
            6 => {
                if string.is_empty() {
                    send_to_char(w, "You must supply a field name.\n\r", ch);
                    return;
                }
                match w[obj].ex_description.iter().position(|e| str_cmp(&e.keyword, &string) == 0) {
                    Some(p) => {
                        w[obj].ex_description.remove(p);
                        send_to_char(w, "Field deleted.\n\r", ch);
                    }
                    None => send_to_char(w, "No field with that keyword.\n\r", ch),
                }
                return;
            }
            _ => {
                send_to_char(w, "That field is undefined for objects.\n\r", ch);
                return;
            }
        }
    };

    let maxlen = LENGTH[field - 1];
    if !string.is_empty() {
        if string.len() > maxlen {
            send_to_char(w, "String too long - truncated.\n\r", ch);
            string.truncate(maxlen);
        }
        if let Some(t) = target_mut(w, &target) {
            *t = string;
        }
        send_to_char(w, "Ok.\n\r", ch);
    } else {
        send_to_char(w, "Enter string. terminate with '@'.\n\r", ch);
        if let Some(t) = target_mut(w, &target) {
            t.clear();
        }
        if let Some(d) = w.desc_mut(did) {
            d.str_target = Some(target);
            d.max_str = maxlen;
        }
    }
}

/* ------------------------------------------------------------------ */
/* paging                                                              */
/* ------------------------------------------------------------------ */

pub fn page_string(w: &mut World, did: DescId, s: &str) {
    if let Some(d) = w.desc_mut(did) {
        d.showstr = Some((s.to_string(), 0));
    } else {
        return;
    }
    show_string(w, did, "");
}

pub fn show_string(w: &mut World, did: DescId, input: &str) {
    let (buf, _) = one_argument(input);
    let Some(d) = w.desc_mut(did) else { return };
    if !buf.is_empty() {
        d.showstr = None;
        return;
    }
    let Some((text, start)) = d.showstr.take() else { return };
    let bytes = text.as_bytes();
    let mut lines = 0;
    let mut toggle = 1i32;
    let mut i = start;
    loop {
        let c = bytes.get(i).copied().unwrap_or(0);
        if (c == b'\n' || c == b'\r') && {
            toggle = -toggle;
            toggle < 0
        } {
            lines += 1;
        } else if c == 0 || lines >= 22 {
            break;
        }
        i += 1;
    }
    let chunk = String::from_utf8_lossy(&bytes[start..i.min(bytes.len())]).to_string();
    d.output.push(chunk);
    let rest_empty = bytes[i.min(bytes.len())..].iter().all(|b| b.is_ascii_whitespace());
    if !rest_empty {
        d.showstr = Some((text, i));
    }
}

/* ------------------------------------------------------------------ */
/* automatic reboot                                                    */
/* ------------------------------------------------------------------ */

const REBOOT_AT: i64 = 10; // hour (UTC in this port) of the optional reboot

/// Once a day, if a file `reboot` exists in the data directory, warn the
/// players and reboot.  A non-empty file is run as a shell script first.
pub fn check_reboot(w: &mut World) {
    let t = now();
    let hour = (t / 3600) % 24;
    let min = (t / 60) % 60;
    if hour + 1 != REBOOT_AT || min <= 30 {
        return;
    }
    let Ok(meta) = std::fs::metadata("./reboot") else { return };
    if min > 50 {
        log("Reboot exists.");
        if meta.len() > 0 {
            log("Reboot is nonempty.");
            let ok = std::process::Command::new("sh").arg("./reboot").status().map(|s| s.success()).unwrap_or(false);
            if !ok {
                log("Reboot script terminated abnormally");
                send_to_all(w, "The reboot was cancelled.\n\r");
                let _ = std::fs::rename("./reboot", "reboot.FAILED");
                return;
            }
            let _ = std::fs::rename("./reboot", "reboot.SUCCEEDED");
        }
        send_to_all(w, "Automatic reboot. Come back in a little while.\n\r");
        w.shutdown = true;
        w.reboot = true;
    } else if min > 40 {
        send_to_all(w, "ATTENTION: DikuMUD will reboot in 10 minutes.\n\r");
    } else {
        send_to_all(w, "Warning: The game will close and reboot in 20 minutes.\n\r");
    }
}
