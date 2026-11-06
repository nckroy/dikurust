//! Informative commands: look, score, who, where ... (act.informative.c).

use crate::comm::*;
use crate::constants::*;
use crate::game::World;
use crate::handler::*;
use crate::interpreter::{one_argument, search_block, CMD_INFO};
use crate::limits::{hit_limit, mana_limit, move_limit};
use crate::modify::page_string;
use crate::structs::*;
use crate::utils::*;

fn page(w: &mut World, ch: CharId, s: &str) {
    if let Some(d) = w[ch].desc {
        page_string(w, d, s);
    }
}

/// argument_split_2(): two words, lower-cased, no fill-word skipping.
fn argument_split_2(argument: &str) -> (String, String) {
    let mut it = argument.split(|c: char| (c as u32) <= b' ' as u32).filter(|s| !s.is_empty());
    let a = it.next().unwrap_or("").to_ascii_lowercase();
    let b = it.next().unwrap_or("").to_ascii_lowercase();
    (a, b)
}

pub fn get_object_in_equip_vis(w: &World, ch: CharId, arg: &str) -> Option<(ObjId, usize)> {
    for j in 0..MAX_WEAR {
        if let Some(o) = w[ch].equipment[j] {
            if w.can_see_obj(ch, o) && isname(arg, &w[o].name) {
                return Some((o, j));
            }
        }
    }
    None
}

pub fn find_ex_description(word: &str, list: &[ExtraDescr]) -> Option<String> {
    list.iter().find(|e| isname(word, &e.keyword)).map(|e| e.description.clone())
}

pub fn show_obj_to_char(w: &mut World, obj: ObjId, ch: CharId, mode: i32) {
    let o = &w[obj];
    let mut buffer = String::new();
    if mode == 0 {
        buffer.push_str(&o.description);
    } else if (1..=4).contains(&mode) {
        buffer.push_str(&o.short_description);
    } else if mode == 5 {
        if o.obj_flags.type_flag == ITEM_NOTE {
            if !o.action_description.is_empty() {
                let b = format!("There is something written upon it:\n\r\n\r{}", o.action_description);
                page(w, ch, &b);
            } else {
                act(w, "It's blank.", false, ch, None, Vict::None, TO_CHAR);
            }
            return;
        } else if o.obj_flags.type_flag != ITEM_DRINKCON {
            buffer.push_str("You see nothing special..");
        } else {
            buffer.push_str("It looks like a drink container.");
        }
    }
    if mode != 3 {
        if w.is_obj_stat(obj, ITEM_INVISIBLE) {
            buffer.push_str("(invisible)");
        }
        if w.is_obj_stat(obj, ITEM_EVIL) && w.is_affected(ch, AFF_DETECT_EVIL) {
            buffer.push_str("..It glows red!");
        }
        if w.is_obj_stat(obj, ITEM_MAGIC) && w.is_affected(ch, AFF_DETECT_MAGIC) {
            buffer.push_str("..It glows blue!");
        }
        if w.is_obj_stat(obj, ITEM_GLOW) {
            buffer.push_str("..It has a soft glowing aura!");
        }
        if w.is_obj_stat(obj, ITEM_HUM) {
            buffer.push_str("..It emits a faint humming sound!");
        }
    }
    buffer.push_str("\n\r");
    page(w, ch, &buffer);
}

pub fn list_obj_to_char(w: &mut World, list: &[ObjId], ch: CharId, mode: i32, show: bool) {
    let mut found = false;
    for &i in list {
        if w.can_see_obj(ch, i) {
            show_obj_to_char(w, i, ch, mode);
            found = true;
        }
    }
    if !found && show {
        send_to_char(w, "Nothing\n\r", ch);
    }
}

pub fn show_char_to_char(w: &mut World, i: CharId, ch: CharId, mode: i32) {
    match mode {
        0 => {
            if w.is_affected(i, AFF_HIDE) || !w.can_see(ch, i) {
                if w.is_affected(ch, AFF_SENSE_LIFE) {
                    send_to_char(w, "You sense a hidden life form in the room.\n\r", ch);
                }
                return;
            }
            let c = &w[i];
            if c.player.long_descr.is_empty() || c.specials.position != c.specials.default_pos {
                let mut buffer = if !w.is_npc(i) {
                    format!("{} {}", c.player.name, c.player.title)
                } else {
                    cap(&c.player.short_descr)
                };
                if w.is_affected(i, AFF_INVISIBLE) {
                    buffer.push_str(" (invisible)");
                }
                let s = match c.specials.position {
                    POSITION_STUNNED => " is lying here, stunned.".to_string(),
                    POSITION_INCAP => " is lying here, incapacitated.".to_string(),
                    POSITION_MORTALLYW => " is lying here, mortally wounded.".to_string(),
                    POSITION_DEAD => " is lying here, dead.".to_string(),
                    POSITION_STANDING => " is standing here.".to_string(),
                    POSITION_SITTING => " is sitting here.".to_string(),
                    POSITION_RESTING => " is resting here.".to_string(),
                    POSITION_SLEEPING => " is sleeping here.".to_string(),
                    POSITION_FIGHTING => match c.specials.fighting {
                        Some(f) if w.ch_exists(f) => {
                            let mut s = " is here, fighting ".to_string();
                            if f == ch {
                                s.push_str(" YOU!");
                            } else if c.in_room == w[f].in_room {
                                if w.is_npc(f) {
                                    s.push_str(&w[f].player.short_descr);
                                } else {
                                    s.push_str(&w[f].player.name);
                                }
                            } else {
                                s.push_str("someone who has already left.");
                            }
                            s
                        }
                        _ => " is here struggling with thin air.".to_string(),
                    },
                    _ => " is floating here.".to_string(),
                };
                buffer.push_str(&s);
                if w.is_affected(ch, AFF_DETECT_EVIL) && w.is_evil(i) {
                    buffer.push_str(" (Red Aura)");
                }
                buffer.push_str("\n\r");
                send_to_char(w, &buffer, ch);
            } else {
                let mut buffer = String::new();
                if w.is_affected(i, AFF_INVISIBLE) {
                    buffer.push('*');
                }
                if w.is_affected(ch, AFF_DETECT_EVIL) && w.is_evil(i) {
                    buffer.push_str(" (Red Aura)");
                }
                buffer.push_str(&w[i].player.long_descr);
                send_to_char(w, &buffer, ch);
            }
            if w.is_affected(i, AFF_SANCTUARY) {
                act(w, "$n glows with a bright light!", false, i, None, Vict::Ch(ch), TO_VICT);
            }
        }
        1 => {
            if !w[i].player.description.is_empty() {
                let d = w[i].player.description.clone();
                send_to_char(w, &d, ch);
            } else {
                act(w, "You see nothing special about $m.", false, i, None, Vict::Ch(ch), TO_VICT);
            }
            let maxh = hit_limit(w, i);
            let percent = if maxh > 0 { (100 * w[i].points.hit) / maxh } else { -1 };
            let mut buffer = if w.is_npc(i) { w[i].player.short_descr.clone() } else { w[i].player.name.clone() };
            buffer.push_str(if percent >= 100 {
                " is in an excellent condition.\n\r"
            } else if percent >= 90 {
                " has a few scratches.\n\r"
            } else if percent >= 75 {
                " has some small wounds and bruises.\n\r"
            } else if percent >= 50 {
                " has quite a few wounds.\n\r"
            } else if percent >= 30 {
                " has some big nasty wounds and scratches.\n\r"
            } else if percent >= 15 {
                " looks pretty hurt.\n\r"
            } else if percent >= 0 {
                " is in an awful condition.\n\r"
            } else {
                " is bleeding awfully from big wounds.\n\r"
            });
            send_to_char(w, &buffer, ch);

            let visible_eq: Vec<(usize, ObjId)> = (0..MAX_WEAR)
                .filter_map(|j| w[i].equipment[j].map(|o| (j, o)))
                .filter(|&(_, o)| w.can_see_obj(ch, o))
                .collect();
            if !visible_eq.is_empty() {
                act(w, "\n\r$n is using:", false, i, None, Vict::Ch(ch), TO_VICT);
                for (j, o) in visible_eq {
                    send_to_char(w, WHERE[j], ch);
                    show_obj_to_char(w, o, ch, 1);
                }
            }
            if w[ch].player.class == CLASS_THIEF && ch != i {
                let mut found = false;
                send_to_char(w, "\n\rYou attempt to peek at the inventory:\n\r", ch);
                for o in w[i].carrying.clone() {
                    if w.can_see_obj(ch, o) && number(0, 20) < w.level(ch) {
                        show_obj_to_char(w, o, ch, 1);
                        found = true;
                    }
                }
                if !found {
                    send_to_char(w, "You can't see anything.\n\r", ch);
                }
            }
        }
        2 => {
            act(w, "$n is carrying:", false, i, None, Vict::Ch(ch), TO_VICT);
            let c = w[i].carrying.clone();
            list_obj_to_char(w, &c, ch, 1, true);
        }
        _ => {}
    }
}

pub fn list_char_to_char(w: &mut World, list: &[CharId], ch: CharId, mode: i32) {
    for &i in list {
        if ch != i && (w.is_affected(ch, AFF_SENSE_LIFE) || (w.can_see(ch, i) && !w.is_affected(i, AFF_HIDE))) {
            show_char_to_char(w, i, ch, mode);
        }
    }
}

pub fn do_look(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    const KEYWORDS: [&str; 9] = ["north", "east", "south", "west", "up", "down", "in", "at", ""];
    if w[ch].desc.is_none() {
        return;
    }
    if w.get_pos(ch) < POSITION_SLEEPING {
        send_to_char(w, "You can't see anything but stars!\n\r", ch);
        return;
    }
    if w.get_pos(ch) == POSITION_SLEEPING {
        send_to_char(w, "You can't see anything, you're sleeping!\n\r", ch);
        return;
    }
    if w.is_affected(ch, AFF_BLIND) {
        send_to_char(w, "You can't see a damn thing, you're blinded!\n\r", ch);
        return;
    }
    if w.is_dark(w[ch].in_room) {
        send_to_char(w, "It is pitch black...\n\r", ch);
        return;
    }
    let (arg1, mut arg2) = argument_split_2(argument);
    let mut keyword_no = search_block(&arg1, &KEYWORDS, false);
    if keyword_no == -1 && !arg1.is_empty() {
        keyword_no = 7;
        arg2 = arg1.clone();
    }
    let room = w[ch].in_room;
    match keyword_no {
        0..=5 => {
            let d = keyword_no as usize;
            match w.exit(ch, d).cloned() {
                Some(e) => {
                    if !e.general_description.is_empty() {
                        send_to_char(w, &e.general_description, ch);
                    } else {
                        send_to_char(w, "You see nothing special.\n\r", ch);
                    }
                    if e.exit_info & EX_CLOSED != 0 && !e.keyword.is_empty() {
                        send_to_char(w, &format!("The {} is closed.\n\r", fname(&e.keyword)), ch);
                    } else if e.exit_info & EX_ISDOOR != 0 && !e.keyword.is_empty() {
                        send_to_char(w, &format!("The {} is open.\n\r", fname(&e.keyword)), ch);
                    }
                }
                None => send_to_char(w, "Nothing special there...\n\r", ch),
            }
        }
        6 => {
            if arg2.is_empty() {
                send_to_char(w, "Look in what?!\n\r", ch);
                return;
            }
            let (bits, _, obj) = generic_find(w, &arg2, FIND_OBJ_INV | FIND_OBJ_ROOM | FIND_OBJ_EQUIP, ch);
            let Some(obj) = obj.filter(|_| bits != 0) else {
                send_to_char(w, "You do not see that item here.\n\r", ch);
                return;
            };
            let v = w[obj].obj_flags.value;
            match w[obj].obj_flags.type_flag {
                ITEM_DRINKCON => {
                    if v[1] <= 0 {
                        act(w, "It is empty.", false, ch, None, Vict::None, TO_CHAR);
                    } else {
                        let temp = if v[0] > 0 { ((v[1] * 3) / v[0]).clamp(0, 3) } else { 3 } as usize;
                        let b = format!(
                            "It's {}full of a {} liquid.\n\r",
                            FULLNESS[temp],
                            COLOR_LIQUID[v[2].clamp(0, 15) as usize]
                        );
                        send_to_char(w, &b, ch);
                    }
                }
                ITEM_CONTAINER => {
                    if v[1] & CONT_CLOSED == 0 {
                        let n = fname(&w[obj].name);
                        send_to_char(w, &n, ch);
                        match bits {
                            FIND_OBJ_INV => send_to_char(w, " (carried) : \n\r", ch),
                            FIND_OBJ_ROOM => send_to_char(w, " (here) : \n\r", ch),
                            FIND_OBJ_EQUIP => send_to_char(w, " (used) : \n\r", ch),
                            _ => {}
                        }
                        let c = w[obj].contains.clone();
                        list_obj_to_char(w, &c, ch, 2, true);
                    } else {
                        send_to_char(w, "It is closed.\n\r", ch);
                    }
                }
                _ => send_to_char(w, "That is not a container.\n\r", ch),
            }
        }
        7 => {
            if arg2.is_empty() {
                send_to_char(w, "Look at what?\n\r", ch);
                return;
            }
            let (bits, tmp_char, found_object) =
                generic_find(w, &arg2, FIND_OBJ_INV | FIND_OBJ_ROOM | FIND_OBJ_EQUIP | FIND_CHAR_ROOM, ch);
            if let Some(tc) = tmp_char {
                show_char_to_char(w, tc, ch, 1);
                if ch != tc {
                    act(w, "$n looks at you.", true, ch, None, Vict::Ch(tc), TO_VICT);
                    act(w, "$n looks at $N.", true, ch, None, Vict::Ch(tc), TO_NOTVICT);
                }
                return;
            }
            if let Some(d) = find_ex_description(&arg2, &w.room(room).ex_description) {
                page(w, ch, &d);
                return;
            }
            let mut found = false;
            for j in 0..MAX_WEAR {
                if found {
                    break;
                }
                if let Some(o) = w[ch].equipment[j] {
                    if w.can_see_obj(ch, o) {
                        if let Some(d) = find_ex_description(&arg2, &w[o].ex_description) {
                            page(w, ch, &d);
                            found = true;
                        }
                    }
                }
            }
            if !found {
                for o in w[ch].carrying.clone() {
                    if w.can_see_obj(ch, o) {
                        if let Some(d) = find_ex_description(&arg2, &w[o].ex_description) {
                            page(w, ch, &d);
                            found = true;
                            break;
                        }
                    }
                }
            }
            if !found {
                for o in w.room(room).contents.clone() {
                    if w.can_see_obj(ch, o) {
                        if let Some(d) = find_ex_description(&arg2, &w[o].ex_description) {
                            page(w, ch, &d);
                            found = true;
                            break;
                        }
                    }
                }
            }
            if bits != 0 {
                if let Some(o) = found_object {
                    show_obj_to_char(w, o, ch, if found { 6 } else { 5 });
                }
            } else if !found {
                send_to_char(w, "You do not see that here.\n\r", ch);
            }
        }
        8 => {
            let name = w.room(room).name.clone();
            send_to_char(w, &name, ch);
            send_to_char(w, "\n\r", ch);
            if w[ch].specials.act & PLR_BRIEF == 0 {
                let d = w.room(room).description.clone();
                send_to_char(w, &d, ch);
            }
            let contents = w.room(room).contents.clone();
            list_obj_to_char(w, &contents, ch, 0, false);
            let people = w.room(room).people.clone();
            list_char_to_char(w, &people, ch, 0);
        }
        _ => send_to_char(w, "Sorry, I didn't understand that!\n\r", ch),
    }
}

pub fn do_read(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let buf = format!("at {}", argument);
    do_look(w, ch, &buf, 15);
}

pub fn do_examine(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let buf = format!("at {}", argument);
    do_look(w, ch, &buf, 15);
    let (name, _) = one_argument(argument);
    if name.is_empty() {
        send_to_char(w, "Examine what?\n\r", ch);
        return;
    }
    let (_, _, obj) = generic_find(w, &name, FIND_OBJ_INV | FIND_OBJ_ROOM | FIND_OBJ_EQUIP, ch);
    if let Some(o) = obj {
        let t = w[o].obj_flags.type_flag;
        if t == ITEM_DRINKCON || t == ITEM_CONTAINER {
            send_to_char(w, "When you look inside, you see:\n\r", ch);
            let buf = format!("in {}", argument);
            do_look(w, ch, &buf, 15);
        }
    }
}

pub fn do_exits(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    const EXITS: [&str; 6] = ["North", "East ", "South", "West ", "Up   ", "Down "];
    let mut buf = String::new();
    for door in 0..6 {
        if let Some(e) = w.exit(ch, door) {
            if e.to_room != NOWHERE && e.exit_info & EX_CLOSED == 0 {
                if w.is_dark(e.to_room) {
                    buf.push_str(&format!("{} - Too dark to tell\n\r", EXITS[door]));
                } else {
                    buf.push_str(&format!("{} - {}\n\r", EXITS[door], w.room(e.to_room).name));
                }
            }
        }
    }
    send_to_char(w, "Obvious exits:\n\r", ch);
    if !buf.is_empty() {
        send_to_char(w, &buf, ch);
    } else {
        send_to_char(w, "None.\n\r", ch);
    }
}

pub fn do_score(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    let a = age(&w[ch]);
    let mut buf = format!("You are {} years old.", a.year);
    if a.month == 0 && a.day == 0 {
        buf.push_str(" It's your birthday today.\n\r");
    } else {
        buf.push_str("\n\r");
    }
    send_to_char(w, &buf, ch);
    let cond = w[ch].specials.conditions;
    if cond[DRUNK] > 10 {
        send_to_char(w, "You are intoxicated.\n\r", ch);
    }
    if cond[THIRST] == 0 {
        send_to_char(w, "You are thirsty.\n\r", ch);
    }
    if cond[FULL] == 0 {
        send_to_char(w, "You are hungry.\n\r", ch);
    }
    let p = w[ch].points;
    let buf = format!(
        "You have {}({}) hit, {}({}) mana and {}({}) movement points.\n\r",
        p.hit,
        hit_limit(w, ch),
        p.mana,
        mana_limit(w, ch),
        p.mov,
        move_limit(w, ch)
    );
    send_to_char(w, &buf, ch);
    let buf = format!("You have scored {} exp, and have {} gold coins.\n\r", p.exp, p.gold);
    send_to_char(w, &buf, ch);
    let t = &w[ch].player.time;
    let playing = real_time_passed((now() - t.logon) + t.played, 0);
    let buf = format!("You have been playing for {} days and {} hours.\n\r", playing.day, playing.hours);
    send_to_char(w, &buf, ch);
    let buf = format!(
        "This ranks you as {} {} (level {}).\n\r",
        w[ch].player.name, w[ch].player.title, w[ch].player.level
    );
    send_to_char(w, &buf, ch);
    match w.get_pos(ch) {
        POSITION_DEAD => send_to_char(w, "You are DEAD!\n\r", ch),
        POSITION_MORTALLYW => send_to_char(w, "You are mortally wounded!, you should seek help!\n\r", ch),
        POSITION_INCAP => send_to_char(w, "You are incapacitated, slowly fading away\n\r", ch),
        POSITION_STUNNED => send_to_char(w, "You are stunned! You can't move\n\r", ch),
        POSITION_SLEEPING => send_to_char(w, "You are sleeping.\n\r", ch),
        POSITION_RESTING => send_to_char(w, "You are resting.\n\r", ch),
        POSITION_SITTING => send_to_char(w, "You are sitting.\n\r", ch),
        POSITION_FIGHTING => match w[ch].specials.fighting {
            Some(f) => act(w, "You are fighting $N.\n\r", false, ch, None, Vict::Ch(f), TO_CHAR),
            None => send_to_char(w, "You are fighting thin air.\n\r", ch),
        },
        POSITION_STANDING => send_to_char(w, "You are standing.\n\r", ch),
        _ => send_to_char(w, "You are floating.\n\r", ch),
    }
}

pub fn do_time(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    let t = w.time_info;
    let h = if t.hours % 12 == 0 { 12 } else { t.hours % 12 };
    let weekday = ((35 * t.month) + t.day + 1).rem_euclid(7) as usize;
    let buf = format!("It is {} o'clock {}, on {}\n\r", h, if t.hours >= 12 { "pm" } else { "am" }, WEEKDAYS[weekday]);
    send_to_char(w, &buf, ch);
    let day = t.day + 1;
    let suf = if day == 1 {
        "st"
    } else if day == 2 {
        "nd"
    } else if day == 3 {
        "rd"
    } else if day < 20 {
        "th"
    } else if day % 10 == 1 {
        "st"
    } else if day % 10 == 2 {
        "nd"
    } else if day % 10 == 3 {
        "rd"
    } else {
        "th"
    };
    let buf = format!(
        "The {}{} Day of the {}, Year {}.\n\r",
        day,
        suf,
        MONTH_NAME[t.month.clamp(0, 16) as usize],
        t.year
    );
    send_to_char(w, &buf, ch);
}

pub fn do_weather(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    const SKY_LOOK: [&str; 4] = ["cloudless", "cloudy", "rainy", "lit by flashes of lightning"];
    if w.outside(ch) {
        let buf = format!(
            "The sky is {} and {}.\n\r",
            SKY_LOOK[w.weather_info.sky.clamp(0, 3) as usize],
            if w.weather_info.change >= 0 {
                "you feel a warm wind from south"
            } else {
                "your foot tells you bad weather is due"
            }
        );
        send_to_char(w, &buf, ch);
    } else {
        send_to_char(w, "You have no feeling about the weather at all.\n\r", ch);
    }
}

pub fn do_help(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w[ch].desc.is_none() {
        return;
    }
    let argument = argument.trim_start();
    if !argument.is_empty() {
        if w.help_index.is_empty() {
            send_to_char(w, "No help available.\n\r", ch);
            return;
        }
        // same binary search (on a prefix) as the original
        let (mut bot, mut top) = (0i32, w.help_index.len() as i32 - 1);
        loop {
            let mid = (bot + top) / 2;
            let kw = &w.help_index[mid as usize].0;
            let chk = strn_cmp(argument, kw, argument.len());
            if chk == 0 {
                let text = w.help_index[mid as usize].1.clone();
                page(w, ch, &text);
                return;
            } else if bot >= top {
                send_to_char(w, "There is no help on that word.\n\r", ch);
                return;
            } else if chk > 0 {
                bot = mid + 1;
            } else {
                top = mid - 1;
            }
            if top < bot {
                send_to_char(w, "There is no help on that word.\n\r", ch);
                return;
            }
        }
    }
    let h = w.help.clone();
    send_to_char(w, &h, ch);
}

pub fn do_wizhelp(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        return;
    }
    send_to_char(w, "The following privileged comands are available:\n\r\n\r", ch);
    let mut buf = String::new();
    let mut no = 1;
    let lvl = w.level(ch);
    for (i, c) in CMD_INFO.iter().enumerate().skip(1) {
        // (cmd 218, "gol", was hidden from the list in the original)
        if lvl >= c.min_level && c.min_level >= 21 && i != 218 {
            buf.push_str(&format!("{:<10}", c.name));
            if no % 7 == 0 {
                buf.push_str("\n\r");
            }
            no += 1;
        }
    }
    buf.push_str("\n\r");
    page(w, ch, &buf);
}

pub fn do_who(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    send_to_char(w, "Players\n\r-------\n\r", ch);
    let lines: Vec<String> = w
        .descriptors
        .iter()
        .filter(|d| d.connected == CON_PLYNG)
        .filter_map(|d| {
            let c = d.character?;
            if !w.ch_exists(c) || !(w.can_see(ch, c) || w.level(ch) >= 23) {
                return None;
            }
            let who = d.original.filter(|o| w.ch_exists(*o)).unwrap_or(c);
            Some(format!("{} {}\n\r", w[who].player.name, w[who].player.title))
        })
        .collect();
    for l in lines {
        send_to_char(w, &l, ch);
    }
}

pub fn do_users(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    let mut buf = String::from("Connections:\n\r------------\n\r");
    for d in &w.descriptors {
        let name = match d.original.or(d.character) {
            Some(c) if w.ch_exists(c) && !w[c].player.name.is_empty() => w[c].player.name.clone(),
            _ => String::new(),
        };
        let mut line = if name.is_empty() { "UNDEFINED       : ".to_string() } else { format!("{:<16}: ", name) };
        if !d.host.is_empty() {
            line.push_str(&format!("[{}]\n\r", d.host));
        } else {
            line.push_str("[Hostname unknown]\n\r");
        }
        buf.push_str(&line);
    }
    send_to_char(w, &buf, ch);
}

pub fn do_inventory(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    send_to_char(w, "You are carrying:\n\r", ch);
    let c = w[ch].carrying.clone();
    list_obj_to_char(w, &c, ch, 1, true);
}

pub fn do_equipment(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    send_to_char(w, "You are using:\n\r", ch);
    let mut found = false;
    for j in 0..MAX_WEAR {
        if let Some(o) = w[ch].equipment[j] {
            send_to_char(w, WHERE[j], ch);
            if w.can_see_obj(ch, o) {
                show_obj_to_char(w, o, ch, 1);
            } else {
                send_to_char(w, "Something.\n\r", ch);
            }
            found = true;
        }
    }
    if !found {
        send_to_char(w, " Nothing.\n\r", ch);
    }
}

pub fn do_credits(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    let s = w.credits.clone();
    page(w, ch, &s);
}

pub fn do_news(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    let s = w.news.clone();
    page(w, ch, &s);
}

pub fn do_info(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    let s = w.info.clone();
    page(w, ch, &s);
}

pub fn do_wizlist(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    let s = w.wizlist.clone();
    page(w, ch, &s);
}

pub fn do_where(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (name, _) = one_argument(argument);
    if name.is_empty() {
        if w.level(ch) < 21 {
            send_to_char(w, "What are you looking for?\n\r", ch);
            return;
        }
        // (the C code overwrote this header before sending it)
        let lines: Vec<String> = w
            .descriptors
            .iter()
            .filter(|d| d.connected == CON_PLYNG)
            .filter_map(|d| {
                let c = d.character?;
                if !w.ch_exists(c) || w[c].in_room == NOWHERE {
                    return None;
                }
                let r = w.room(w[c].in_room);
                Some(match d.original.filter(|o| w.ch_exists(*o)) {
                    Some(o) => format!(
                        "{:<20} - {} [{}] In body of {}\n\r",
                        w[o].player.name,
                        r.name,
                        r.number,
                        fname(&w[c].player.name)
                    ),
                    None => format!("{:<20} - {} [{}]\n\r", w[c].player.name, r.name, r.number),
                })
            })
            .collect();
        for l in lines {
            send_to_char(w, &l, ch);
        }
        return;
    }
    let mut any = false;
    let god = w.level(ch) > 20;
    for i in w.character_list.clone() {
        if isname(&name, &w[i].player.name) && w.can_see(ch, i) {
            let r = w[i].in_room;
            if r != NOWHERE && (god || w.room(r).zone == w.room(w[ch].in_room).zone) {
                let who = if w.is_npc(i) { w[i].player.short_descr.clone() } else { w[i].player.name.clone() };
                let mut buf = format!("{:<30}- {} ", who, w.room(r).name);
                if w.level(ch) >= 21 {
                    buf.push_str(&format!("[{}]\n\r", w.room(r).number));
                } else {
                    buf.push_str("\n\r");
                }
                send_to_char(w, &buf, ch);
                any = true;
                if w.level(ch) < 21 {
                    break;
                }
            }
        }
    }
    if god {
        for k in w.object_list.clone() {
            if isname(&name, &w[k].name) && w.can_see_obj(ch, k) && w[k].in_room != NOWHERE {
                let r = w.room(w[k].in_room);
                let buf = format!("{:<30}- {} [{}]\n\r", w[k].short_description, r.name, r.number);
                send_to_char(w, &buf, ch);
                any = true;
            }
        }
    }
    if !any {
        send_to_char(w, "Couldn't find any such thing.\n\r", ch);
    }
}

pub fn do_levels(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        send_to_char(w, "You ain't nothin' but a hound-dog.\n\r", ch);
        return;
    }
    let mut class = w[ch].player.class;
    let (arg, _) = one_argument(argument);
    if !arg.is_empty() {
        if str_cmp(&arg, "magic") == 0 {
            class = 1;
        } else if str_cmp(&arg, "cleric") == 0 {
            class = 2;
        } else if str_cmp(&arg, "thief") == 0 {
            class = 3;
        } else if str_cmp(&arg, "fighter") == 0 {
            class = 4;
        }
    }
    let cls = (class - 1).clamp(0, 3) as usize;
    let mut buf = String::new();
    for i in 1..21 {
        buf.push_str(&format!("{:7}-{:<7} : ", TITLES[cls][i].exp, TITLES[cls][i + 1].exp));
        match w[ch].player.sex {
            SEX_MALE => buf.push_str(TITLES[cls][i].title_m),
            SEX_FEMALE => buf.push_str(TITLES[cls][i].title_f),
            _ => send_to_char(w, "Oh dear.\n\r", ch),
        }
        buf.push_str("\n\r");
    }
    send_to_char(w, &buf, ch);
}

pub fn do_consider(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (name, _) = one_argument(argument);
    let Some(victim) = get_char_room_vis(w, ch, &name) else {
        send_to_char(w, "Consider killing who?\n\r", ch);
        return;
    };
    if victim == ch {
        send_to_char(w, "Easy! Very easy indeed!\n\r", ch);
        return;
    }
    if !w.is_npc(victim) {
        send_to_char(w, "Would you like to borrow a cross and a shovel?\n\r", ch);
        return;
    }
    let diff = w.level(victim) - w.level(ch);
    let msg = if diff <= -10 {
        "Now where did that chicken go?\n\r"
    } else if diff <= -5 {
        "You could do it with a needle!\n\r"
    } else if diff <= -2 {
        "Easy.\n\r"
    } else if diff <= -1 {
        "Fairly easy.\n\r"
    } else if diff == 0 {
        "The perfect match!\n\r"
    } else if diff <= 1 {
        "You would need some luck!\n\r"
    } else if diff <= 2 {
        "You would need a lot of luck!\n\r"
    } else if diff <= 3 {
        "You would need a lot of luck and great equipment!\n\r"
    } else if diff <= 5 {
        "Do you feel lucky, punk?\n\r"
    } else if diff <= 10 {
        "Are you mad!?\n\r"
    } else if diff <= 100 {
        "You ARE mad!\n\r"
    } else {
        ""
    };
    send_to_char(w, msg, ch);
}
