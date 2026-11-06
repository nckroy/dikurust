//! Newer immortal commands (changes.c - linked into the original binary
//! although its Makefile did not mention it).

use crate::comm::*;
use crate::game::World;
use crate::handler::*;
use crate::interpreter::one_argument;
use crate::structs::*;
use crate::utils::*;

/// Shared shape of noemote / notell / freeze / log.
fn toggle_flag(w: &mut World, ch: CharId, argument: &str, flag: i32, on: (&str, &str), off: (&str, &str), strict: bool) {
    let (bits, vict, _) = generic_find(w, argument, FIND_CHAR_WORLD, ch);
    let Some(vict) = vict.filter(|_| bits != 0) else {
        send_to_char(w, "Couldn't find any such creature.\n\r", ch);
        return;
    };
    let too_high = if strict { w.level(vict) >= w.level(ch) } else { w.level(vict) > w.level(ch) };
    if w.is_npc(vict) {
        send_to_char(w, "Can't do that to a beast.\n\r", ch);
    } else if too_high {
        act(w, "$E might object to that.. better not.", false, ch, None, Vict::Ch(vict), TO_CHAR);
    } else if w[vict].specials.act & flag != 0 {
        if !off.0.is_empty() {
            send_to_char(w, off.0, vict);
        }
        send_to_char(w, off.1, ch);
        w[vict].specials.act &= !flag;
    } else {
        if !on.0.is_empty() {
            send_to_char(w, on.0, vict);
        }
        send_to_char(w, on.1, ch);
        w[vict].specials.act |= flag;
    }
}

pub fn do_noemote(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        return;
    }
    let (buf, _) = one_argument(argument);
    if buf.is_empty() {
        send_to_char(w, "NOEMOTE who?\n\r", ch);
        return;
    }
    toggle_flag(
        w,
        ch,
        argument,
        PLR_NOEMOTE,
        ("The gods take away your ability to emote!\n\r", "NOEMOTE set.\n\r"),
        ("You can emote again.\n\r", "NOEMOTE removed.\n\r"),
        false,
    );
}

pub fn do_notell(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        return;
    }
    let (buf, _) = one_argument(argument);
    if buf.is_empty() {
        if w[ch].specials.act & PLR_NOTELL != 0 {
            send_to_char(w, "You can now hear tells again.\n\r", ch);
            w[ch].specials.act &= !PLR_NOTELL;
        } else {
            send_to_char(w, "From now on, you can't use tell.\n\r", ch);
            w[ch].specials.act |= PLR_NOTELL;
        }
        return;
    }
    toggle_flag(
        w,
        ch,
        argument,
        PLR_NOTELL,
        ("The gods take away your ability to use telepatic communication!\n\r", "NOTELL set.\n\r"),
        ("You can use telepatic communication again.\n\r", "NOTELL removed.\n\r"),
        false,
    );
}

pub fn do_freeze(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        return;
    }
    let (buf, _) = one_argument(argument);
    if buf.is_empty() {
        send_to_char(w, "Freeze who?\n\r", ch);
        return;
    }
    toggle_flag(
        w,
        ch,
        argument,
        PLR_FREEZE,
        ("The gods take away your ability to ...\n\r", "FREEZE set.\n\r"),
        ("You now can do things again.\n\r", "FREEZE removed.\n\r"),
        false,
    );
}

pub fn do_log(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        return;
    }
    let (buf, _) = one_argument(argument);
    if buf.is_empty() {
        send_to_char(w, "Log who?\n\r", ch);
        return;
    }
    toggle_flag(w, ch, argument, PLR_LOG, ("", "LOG set.\n\r"), ("", "LOG removed.\n\r"), true);
}

pub fn do_wizlock(w: &mut World, ch: CharId, _argument: &str, _cmd: i32) {
    w.wizlock = !w.wizlock;
    if w.wizlock {
        log(&format!("Game has been wizlocked by {}.", w[ch].player.name));
        send_to_char(w, "Game wizlocked.\n\r", ch);
    } else {
        log(&format!("Game has been un-wizlocked by {}.", w[ch].player.name));
        send_to_char(w, "Game un-wizlocked.\n\r", ch);
    }
}

const SET_SPELLS: [&str; 52] = [
    "armor", "teleport", "bless", "blindness", "burning hands", "call lightning", "charm person", "chill touch",
    "clone", "colour spray", "control weather", "create food", "create water", "cure blind", "cure critic",
    "cure light", "curse", "detect evil", "detect invisibility", "detect magic", "detect poison", "dispel evil",
    "earthquake", "enchant weapon", "energy drain", "fireball", "harm", "heal", "invisibility", "lightning bolt",
    "locate object", "magic missile", "poison", "protection from evil", "remove curse", "sanctuary",
    "shocking grasp", "zzzzz", "strength", "summon", "ventriloquate", "word of recall", "remove poison",
    "sense life", "sneak", "hide", "steal", "backstab", "pick lock", "kick", "bash", "rescue",
];

const VALUES: [&str; 22] = [
    "age", "sex", "class", "level", "height", "weight", "str", "stradd", "int", "wis", "dex", "con", "gold", "exp",
    "mana", "hit", "move", "sessions", "alignment", "thirst", "drunk", "full",
];

pub fn do_set(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let (name, rest) = one_argument(argument);
    if name.is_empty() {
        send_to_char(w, "Syntax:\n\rset <name> skill '<skill>' <value> <value>\n\r", ch);
        send_to_char(w, "or:\n\rset <name> value <field> <value>\n\r", ch);
        let mut help = String::from("Skill being one of the following:\n\r");
        for (i, s) in SET_SPELLS.iter().enumerate().skip(1) {
            help.push_str(&format!("{:>18}", s));
            if i % 4 == 0 {
                help.push_str("\n\r");
            }
        }
        help.push_str("\n\rField being one of the following:\n\r");
        for (i, s) in VALUES.iter().enumerate().skip(1) {
            help.push_str(&format!("{:>18}", s));
            if i % 4 == 0 {
                help.push_str("\n\r");
            }
        }
        help.push_str("\n\r");
        send_to_char(w, &help, ch);
        return;
    }
    let Some(vict) = get_char_vis(w, ch, &name) else {
        send_to_char(w, "No living thing by that name.\n\r", ch);
        return;
    };
    let (kind, rest) = one_argument(rest);
    let is_skill = str_cmp(&kind, "skill") == 0 || str_cmp(&kind, "s") == 0;
    let is_value = str_cmp(&kind, "value") == 0 || str_cmp(&kind, "v") == 0;
    if !is_skill && !is_value {
        send_to_char(w, "'skill' or 'value' expected.\n\r", ch);
        return;
    }
    if is_skill {
        let a = rest.trim_start_matches(' ');
        if a.is_empty() {
            send_to_char(w, "Skill name expected.\n\r", ch);
            return;
        }
        if !a.starts_with('\'') {
            send_to_char(w, "Skill must be enclosed in: ''\n\r", ch);
            return;
        }
        let Some(qend) = a[1..].find('\'').map(|p| p + 1) else {
            send_to_char(w, "Skill must be enclosed in: ''\n\r", ch);
            return;
        };
        let words = a[1..qend].to_ascii_lowercase();
        let Some(pos) = SET_SPELLS.iter().position(|s| s.starts_with(&words)) else {
            send_to_char(w, "Unrecognized skill.\n\r", ch);
            return;
        };
        // (the C code used the 0-based index here, setting the skill below
        //  the one named; spell/skill numbers start at 1)
        let skill = pos + 1;
        let (vbuf, rest) = one_argument(&a[qend + 1..]);
        if vbuf.is_empty() {
            send_to_char(w, "Learned value expected.\n\r", ch);
            return;
        }
        let value = atoi(&vbuf);
        if value < 0 {
            send_to_char(w, "Minimum value for learned is 0.\n\r", ch);
            return;
        }
        if value > 100 {
            send_to_char(w, "Max value for learned is 100.\n\r", ch);
            return;
        }
        let (rbuf, _) = one_argument(rest);
        if rbuf.is_empty() {
            send_to_char(w, "Recognise value expected.\n\r", ch);
            return;
        }
        if str_cmp(&rbuf, "y") != 0 && str_cmp(&rbuf, "n") != 0 {
            send_to_char(w, "Recognice value must be 'y' or 'n'", ch);
            return;
        }
        log(&format!(
            "{} changes {}'s {} to {},{}.",
            w[ch].player.name, w[vict].player.name, SET_SPELLS[pos], value, rbuf
        ));
        if skill < MAX_SKILLS {
            w[vict].skills[skill].learned = value;
            w[vict].skills[skill].recognise = str_cmp(&rbuf, "y") == 0;
        }
    } else {
        let (field, rest) = one_argument(rest);
        if field.is_empty() {
            send_to_char(w, "Field name expected.\n\r", ch);
            return;
        }
        let Some(f) = VALUES.iter().position(|v| *v == field) else {
            send_to_char(w, "No such field is known. Try 'set' for list.\n\r", ch);
            return;
        };
        let (buf, _) = one_argument(rest);
        if buf.is_empty() {
            send_to_char(w, "Value for field expected.\n\r", ch);
            return;
        }
        let buf2 = format!("{} sets {}'s {} to {}.", w[ch].player.name, w[vict].player.name, VALUES[f], buf);
        let value = atoi(&buf);
        macro_rules! range {
            ($lo:expr, $hi:expr, $m1:expr, $m2:expr) => {
                if !($lo..=$hi).contains(&value) {
                    send_to_char(w, $m1, ch);
                    send_to_char(w, $m2, ch);
                    return;
                }
            };
        }
        match f {
            0 => {
                range!(16, 79, "Age must be more than 16 years\n\r", "and less than 80 years.\n\r");
                w[vict].player.time.birth = now() - value as i64 * SECS_PER_MUD_YEAR;
            }
            1 => {
                let sex = match buf.as_str() {
                    "m" => SEX_MALE,
                    "f" => SEX_FEMALE,
                    "n" => SEX_NEUTRAL,
                    _ => {
                        send_to_char(w, "Sex must be 'm','f' or 'n'.\n\r", ch);
                        return;
                    }
                };
                w[vict].player.sex = sex;
            }
            2 => {
                let class = match buf.as_str() {
                    "m" => CLASS_MAGIC_USER,
                    "c" => CLASS_CLERIC,
                    "w" => CLASS_WARRIOR,
                    "t" => CLASS_THIEF,
                    _ => {
                        send_to_char(w, "Class must be 'm','c','w' or 't'.\n\r", ch);
                        return;
                    }
                };
                w[vict].player.class = class;
            }
            3 => {
                range!(0, 24, "Level must be more than 0\n\r", "and less than 25.\n\r");
                w[vict].player.level = value;
            }
            4 => {
                range!(100, 250, "Height must be more than 100 cm\n\r", "and less than 251 cm.\n\r");
                w[vict].player.height = value;
            }
            5 => {
                range!(100, 250, "Weight must be more than 100 pound\n\r", "and less than 250 pound.\n\r");
                w[vict].player.weight = value;
            }
            6 => {
                range!(1, 18, "Strength must be more than 0\n\r", "and less than 19.\n\r");
                w[vict].abilities.str = value;
            }
            7 => {
                if !(1..=100).contains(&value) {
                    send_to_char(w, "Strength addition must be more\n\r", ch);
                    send_to_char(w, "than 0 and less than 101.\n\r", ch);
                }
                w[vict].abilities.str_add = value;
            }
            8 => {
                range!(1, 18, "Inteligence must be more than 0\n\r", "and less than 19.\n\r");
                w[vict].abilities.intel = value;
            }
            9 => {
                range!(1, 18, "Wisdom must be more than 0\n\r", "and less than 19.\n\r");
                w[vict].abilities.wis = value;
            }
            10 => {
                range!(1, 18, "Dexterity must be more than 0\n\r", "and less than 19.\n\r");
                w[vict].abilities.dex = value;
            }
            11 => {
                range!(1, 18, "Constitution must be more than 0\n\r", "and less than 19.\n\r");
                w[vict].abilities.con = value;
            }
            12 => w[vict].points.gold = value,
            13 => {
                range!(1, 7000000, "Experience-points must be more than 0\n\r", "and less than 7000000.\n\r");
                w[vict].points.exp = value;
            }
            14 => {
                range!(-99, 200, "Mana-points must be more than -100\n\r", "and less than 200.\n\r");
                w[vict].points.mana = value;
            }
            15 => {
                range!(-9, 30000, "Hit-points must be more than -10\n\r", "and less than 30000.\n\r");
                w[vict].points.hit = value;
            }
            16 => {
                range!(-99, 200, "Move-points must be more than -100\n\r", "and less than 200.\n\r");
                w[vict].points.mov = value;
            }
            17 => {
                range!(0, 100, "Sessions must be more than 0\n\r", "and less than 100.\n\r");
                w[vict].specials.spells_to_learn = value;
            }
            18 => {
                range!(-1000, 1000, "Alignment must be more than -1000\n\r", "and less than 1000.\n\r");
                w[vict].specials.alignment = value;
            }
            19 => {
                range!(-1, 100, "Thirst must be more than -2\n\r", "and less than 101.\n\r");
                w[vict].specials.conditions[THIRST] = value;
            }
            20 => {
                range!(-1, 100, "Drunk must be more than -2\n\r", "and less than 101.\n\r");
                w[vict].specials.conditions[DRUNK] = value;
            }
            21 => {
                range!(-1, 100, "Full must be more than -2\n\r", "and less than 101.\n\r");
                w[vict].specials.conditions[FULL] = value;
            }
            _ => {}
        }
        log(&buf2);
    }
    send_to_char(w, "Ok.\n\r", ch);
}

pub fn do_wiz(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    let a = argument.trim_start_matches(' ');
    if a.is_empty() {
        send_to_char(w, "What do you want to tell all gods and immortals?\n\r", ch);
        return;
    }
    send_to_char(w, "Ok.\n\r", ch);
    let buf = format!("::$n::{}", a);
    let targets: Vec<CharId> = w
        .descriptors
        .iter()
        .filter(|d| d.connected == CON_PLYNG)
        .filter_map(|d| d.character)
        .filter(|&c| c != ch && w.ch_exists(c) && w.level(c) > 20)
        .collect();
    for t in targets {
        act(w, &buf, false, ch, None, Vict::Ch(t), TO_VICT);
    }
}
