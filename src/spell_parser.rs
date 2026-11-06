//! Spell table, casting, saving throws and followers (spell_parser.c).

use crate::comm::*;
use crate::constants::SPELL_WEAR_OFF_MSG;
use crate::game::World;
use crate::handler::*;
use crate::interpreter::one_argument;
use crate::spells::*;
use crate::structs::*;
use crate::utils::*;

pub const SPELLS: &[&str] = &[
    "armor",
    "teleport",
    "bless",
    "blindness",
    "burning hands",
    "call lightning",
    "charm person",
    "chill touch",
    "clone",
    "colour spray",
    "control weather",
    "create food",
    "create water",
    "cure blind",
    "cure critic",
    "cure light",
    "curse",
    "detect evil",
    "detect invisibility",
    "detect magic",
    "detect poison",
    "dispel evil",
    "earthquake",
    "enchant weapon",
    "energy drain",
    "fireball",
    "harm",
    "heal",
    "invisibility",
    "lightning bolt",
    "locate object",
    "magic missile",
    "poison",
    "protection from evil",
    "remove curse",
    "sanctuary",
    "shocking grasp",
    "zzzzz",
    "strength",
    "summon",
    "ventriloquate",
    "word of recall",
    "remove poison",
    "sense life",
    "SKILL_SNEAK",
    "SKILL_HIDE",
    "SKILL_STEAL",
    "SKILL_BACKSTAB",
    "SKILL_PICK_LOCK",
    "SKILL_KICK",
    "SKILL_BASH",
    "SKILL_RESCUE",
    "identify",
];

pub const SAVING_THROWS: [[[i32; 25]; 5]; 4] = [
    [
        [16, 14, 14, 14, 14, 14, 13, 13, 13, 13, 13, 11, 11, 11, 11, 11, 10, 10, 10, 10, 10, 8, 6, 4, 0],
        [13, 11, 11, 11, 11, 11, 9, 9, 9, 9, 9, 7, 7, 7, 7, 7, 5, 5, 5, 5, 5, 3, 2, 1, 0],
        [15, 13, 13, 13, 13, 13, 11, 11, 11, 11, 11, 9, 9, 9, 9, 9, 7, 7, 7, 7, 7, 5, 4, 2, 0],
        [17, 15, 15, 15, 15, 15, 13, 13, 13, 13, 13, 11, 11, 11, 11, 11, 9, 9, 9, 9, 9, 7, 5, 3, 0],
        [14, 12, 12, 12, 12, 12, 10, 10, 10, 10, 10, 8, 8, 8, 8, 8, 6, 6, 6, 6, 6, 4, 3, 2, 0],
    ],
    [
        [11, 10, 10, 10, 9, 9, 9, 7, 7, 7, 6, 6, 6, 5, 5, 5, 4, 4, 4, 2, 2, 2, 2, 1, 0],
        [16, 14, 14, 14, 13, 13, 13, 11, 11, 11, 10, 10, 10, 9, 9, 9, 8, 8, 8, 6, 6, 5, 4, 3, 0],
        [15, 13, 13, 13, 12, 12, 12, 10, 10, 10, 9, 9, 9, 8, 8, 8, 7, 7, 7, 5, 5, 4, 3, 2, 0],
        [18, 16, 16, 16, 15, 15, 15, 13, 13, 13, 12, 12, 12, 11, 11, 11, 10, 10, 10, 8, 8, 7, 6, 5, 0],
        [17, 15, 15, 15, 14, 14, 14, 12, 12, 12, 11, 11, 11, 10, 10, 10, 9, 9, 9, 7, 7, 6, 5, 4, 0],
    ],
    [
        [15, 13, 13, 13, 13, 12, 12, 12, 12, 11, 11, 11, 11, 10, 10, 10, 10, 9, 9, 9, 9, 8, 7, 6, 0],
        [16, 14, 14, 14, 14, 12, 12, 12, 12, 10, 10, 10, 10, 8, 8, 8, 8, 6, 6, 6, 6, 4, 3, 2, 0],
        [14, 12, 12, 12, 12, 11, 11, 11, 11, 10, 10, 10, 10, 9, 9, 9, 9, 8, 8, 8, 8, 7, 5, 3, 0],
        [18, 16, 16, 16, 16, 15, 15, 15, 15, 14, 14, 14, 14, 13, 13, 13, 13, 12, 12, 12, 12, 11, 9, 5, 0],
        [17, 15, 15, 15, 15, 13, 13, 13, 13, 11, 11, 11, 11, 9, 9, 9, 9, 7, 7, 7, 7, 5, 3, 1, 0],
    ],
    [
        [16, 14, 14, 13, 13, 11, 11, 10, 10, 8, 8, 7, 7, 5, 5, 4, 4, 3, 3, 3, 3, 2, 2, 1, 0],
        [18, 16, 16, 15, 15, 13, 13, 12, 12, 10, 10, 9, 9, 7, 7, 6, 6, 5, 5, 5, 5, 4, 3, 2, 0],
        [17, 15, 15, 14, 14, 12, 12, 11, 11, 9, 9, 8, 8, 6, 6, 5, 5, 4, 4, 4, 4, 3, 2, 1, 0],
        [20, 17, 17, 16, 16, 13, 13, 12, 12, 9, 9, 8, 8, 5, 5, 4, 4, 4, 4, 4, 4, 3, 2, 1, 0],
        [19, 17, 17, 16, 16, 14, 14, 13, 13, 11, 11, 10, 10, 8, 8, 7, 7, 6, 6, 6, 6, 4, 2, 1, 0],
    ],
];

#[derive(Clone, Copy)]
pub struct SpellInfo {
    pub spell_pointer: Option<CastFn>,
    pub minimum_position: i32,
    pub min_usesmana: i32,
    pub beats: i32,
    pub min_level_cleric: i32,
    pub min_level_magic: i32,
    pub targets: i32,
}

const NONE: SpellInfo = SpellInfo {
    spell_pointer: None,
    minimum_position: 0,
    min_usesmana: 0,
    beats: 0,
    min_level_cleric: 0,
    min_level_magic: 0,
    targets: 0,
};

/// SPELLO(nr, beat, pos, mlev, clev, mana, tar, func)
pub fn spell_info(nr: i32) -> SpellInfo {
    macro_rules! s {
        ($beat:expr, $pos:expr, $mlev:expr, $clev:expr, $mana:expr, $tar:expr, $f:expr) => {
            SpellInfo {
                spell_pointer: $f,
                minimum_position: $pos,
                min_usesmana: $mana,
                beats: $beat,
                min_level_cleric: $clev,
                min_level_magic: $mlev,
                targets: $tar,
            }
        };
    }
    const F: i32 = POSITION_FIGHTING;
    const S: i32 = POSITION_STANDING;
    const CV: i32 = TAR_CHAR_ROOM | TAR_FIGHT_VICT;
    match nr {
        32 => s!(12, F, 1, 21, 15, CV, Some(cast_magic_missile)),
        8 => s!(12, F, 3, 21, 15, CV, Some(cast_chill_touch)),
        5 => s!(12, F, 5, 21, 15, CV, Some(cast_burning_hands)),
        37 => s!(12, F, 7, 21, 15, CV, Some(cast_shocking_grasp)),
        30 => s!(12, F, 9, 21, 15, CV, Some(cast_lightning_bolt)),
        10 => s!(12, F, 11, 21, 15, CV, Some(cast_colour_spray)),
        25 => s!(12, F, 13, 21, 35, CV, Some(cast_energy_drain)),
        26 => s!(12, F, 15, 21, 15, CV, Some(cast_fireball)),
        23 => s!(12, F, 21, 7, 15, TAR_IGNORE, Some(cast_earthquake)),
        22 => s!(12, F, 21, 10, 15, CV, Some(cast_dispel_evil)),
        6 => s!(12, F, 21, 12, 15, CV, Some(cast_call_lightning)),
        27 => s!(12, F, 21, 15, 35, CV, Some(cast_harm)),
        1 => s!(12, S, 5, 1, 5, TAR_CHAR_ROOM, Some(cast_armor)),
        2 => s!(12, F, 8, 21, 35, TAR_SELF_ONLY, Some(cast_teleport)),
        3 => s!(12, S, 21, 5, 5, TAR_OBJ_INV | TAR_OBJ_EQUIP | TAR_CHAR_ROOM, Some(cast_bless)),
        4 => s!(12, S, 8, 6, 5, TAR_CHAR_ROOM, Some(cast_blindness)),
        7 => s!(12, S, 14, 21, 5, TAR_CHAR_ROOM | TAR_SELF_NONO, Some(cast_charm_person)),
        9 => s!(12, S, 15, 21, 40, TAR_CHAR_ROOM, Some(cast_clone)),
        11 => s!(12, S, 10, 13, 25, TAR_IGNORE, Some(cast_control_weather)),
        12 => s!(12, S, 21, 3, 5, TAR_IGNORE, Some(cast_create_food)),
        13 => s!(12, S, 21, 2, 5, TAR_OBJ_INV | TAR_OBJ_EQUIP, Some(cast_create_water)),
        14 => s!(12, S, 21, 4, 5, TAR_CHAR_ROOM, Some(cast_cure_blind)),
        15 => s!(12, F, 21, 9, 20, TAR_CHAR_ROOM, Some(cast_cure_critic)),
        16 => s!(12, F, 21, 1, 15, TAR_CHAR_ROOM, Some(cast_cure_light)),
        17 => s!(12, S, 12, 21, 20, TAR_CHAR_ROOM | TAR_OBJ_ROOM | TAR_OBJ_INV | TAR_OBJ_EQUIP, Some(cast_curse)),
        18 => s!(12, S, 21, 4, 5, TAR_CHAR_ROOM | TAR_SELF_ONLY, Some(cast_detect_evil)),
        19 => s!(12, S, 2, 5, 5, TAR_CHAR_ROOM | TAR_SELF_ONLY, Some(cast_detect_invisibility)),
        20 => s!(12, S, 2, 3, 5, TAR_CHAR_ROOM | TAR_SELF_ONLY, Some(cast_detect_magic)),
        21 => s!(12, S, 21, 2, 5, TAR_CHAR_ROOM | TAR_OBJ_INV | TAR_OBJ_EQUIP, Some(cast_detect_poison)),
        24 => s!(12, S, 12, 21, 100, TAR_OBJ_INV | TAR_OBJ_EQUIP, Some(cast_enchant_weapon)),
        28 => s!(12, F, 21, 14, 50, TAR_CHAR_ROOM, Some(cast_heal)),
        29 => s!(12, S, 4, 21, 5, TAR_CHAR_ROOM | TAR_OBJ_INV | TAR_OBJ_ROOM | TAR_OBJ_EQUIP, Some(cast_invisibility)),
        31 => s!(12, S, 6, 10, 20, TAR_OBJ_WORLD, Some(cast_locate_object)),
        33 => s!(12, S, 21, 8, 10, TAR_CHAR_ROOM | TAR_SELF_NONO | TAR_OBJ_INV | TAR_OBJ_EQUIP, Some(cast_poison)),
        34 => s!(12, S, 21, 6, 5, TAR_CHAR_ROOM | TAR_SELF_ONLY, Some(cast_protection_from_evil)),
        35 => s!(12, S, 21, 12, 5, TAR_CHAR_ROOM | TAR_OBJ_INV | TAR_OBJ_EQUIP | TAR_OBJ_ROOM, Some(cast_remove_curse)),
        36 => s!(12, S, 21, 13, 75, TAR_CHAR_ROOM, Some(cast_sanctuary)),
        38 => s!(12, S, 14, 21, 15, TAR_CHAR_ROOM, Some(cast_sleep)),
        39 => s!(12, S, 7, 21, 20, TAR_CHAR_ROOM | TAR_SELF_ONLY, Some(cast_strength)),
        40 => s!(12, S, 21, 8, 50, TAR_CHAR_WORLD, Some(cast_summon)),
        41 => s!(12, S, 1, 21, 5, TAR_CHAR_ROOM | TAR_OBJ_ROOM | TAR_SELF_NONO, Some(cast_ventriloquate)),
        42 => s!(12, S, 21, 11, 5, TAR_CHAR_ROOM | TAR_SELF_ONLY, Some(cast_word_of_recall)),
        43 => s!(12, S, 21, 9, 5, TAR_CHAR_ROOM | TAR_OBJ_INV | TAR_OBJ_ROOM, Some(cast_remove_poison)),
        44 => s!(12, S, 21, 7, 5, TAR_CHAR_ROOM | TAR_SELF_ONLY, Some(cast_sense_life)),
        45..=52 => s!(0, S, 25, 25, 200, TAR_IGNORE, None),
        53 => s!(1, S, 25, 25, 100, TAR_IGNORE, Some(cast_identify)),
        _ => NONE,
    }
}

fn spell_level(w: &World, ch: CharId, sn: i32) -> i32 {
    let si = spell_info(sn);
    if w[ch].player.class == CLASS_CLERIC {
        si.min_level_cleric
    } else {
        si.min_level_magic
    }
}

/// USE_MANA(ch, sn)
pub fn use_mana(w: &World, ch: CharId, sn: i32) -> i32 {
    let div = (2 + w.level(ch) - spell_level(w, ch, sn)).max(1);
    spell_info(sn).min_usesmana.max(100 / div)
}

pub fn affect_update(w: &mut World) {
    for i in w.character_list.clone() {
        if !w.ch_exists(i) {
            continue;
        }
        let mut idx = 0;
        while idx < w[i].affected.len() {
            let af = w[i].affected[idx];
            if af.duration >= 1 {
                w[i].affected[idx].duration -= 1;
                idx += 1;
            } else {
                if af.type_ > 0 && af.type_ <= 52 {
                    let next = w[i].affected.get(idx + 1).copied();
                    let show = match next {
                        None => true,
                        Some(n) => n.type_ != af.type_ || n.duration > 0,
                    };
                    if show {
                        let msg = SPELL_WEAR_OFF_MSG[af.type_ as usize];
                        if !msg.is_empty() {
                            send_to_char(w, msg, i);
                            send_to_char(w, "\n\r", i);
                        }
                    }
                }
                affect_remove(w, i, idx);
            }
        }
    }
}

/// Would making `ch` follow `victim` create a loop?
pub fn circle_follow(w: &World, ch: CharId, victim: CharId) -> bool {
    let mut k = Some(victim);
    let mut guard = 0;
    while let Some(c) = k {
        if c == ch {
            return true;
        }
        if !w.ch_exists(c) || guard > 1000 {
            break;
        }
        k = w[c].master;
        guard += 1;
    }
    false
}

/// Stop following someone, or stop being charmed.
pub fn stop_follower(w: &mut World, ch: CharId) {
    let Some(master) = w[ch].master else { return };
    if w.ch_exists(master) {
        if w.is_affected(ch, AFF_CHARM) {
            act(w, "You realize that $N is a jerk!", false, ch, None, Vict::Ch(master), TO_CHAR);
            act(w, "$n realizes that $N is a jerk!", false, ch, None, Vict::Ch(master), TO_NOTVICT);
            act(w, "$n hates your guts!", false, ch, None, Vict::Ch(master), TO_VICT);
            if affected_by_spell(w, ch, SPELL_CHARM_PERSON) {
                affect_from_char(w, ch, SPELL_CHARM_PERSON);
            }
        } else {
            act(w, "You stop following $N.", false, ch, None, Vict::Ch(master), TO_CHAR);
            act(w, "$n stops following $N.", false, ch, None, Vict::Ch(master), TO_NOTVICT);
            act(w, "$n stops following you.", false, ch, None, Vict::Ch(master), TO_VICT);
        }
        w[master].followers.retain(|&f| f != ch);
    }
    w[ch].master = None;
    w[ch].specials.affected_by &= !(AFF_CHARM | AFF_GROUP);
}

/// Called when a character that follows / is followed dies or leaves.
pub fn die_follower(w: &mut World, ch: CharId) {
    if w[ch].master.is_some() {
        stop_follower(w, ch);
    }
    for f in w[ch].followers.clone() {
        if w.ch_exists(f) {
            stop_follower(w, f);
        }
    }
    w[ch].followers.clear();
}

pub fn add_follower(w: &mut World, ch: CharId, leader: CharId) {
    if w[ch].master.is_some() {
        return;
    }
    w[ch].master = Some(leader);
    w[leader].followers.insert(0, ch);
    act(w, "You now follow $N.", false, ch, None, Vict::Ch(leader), TO_CHAR);
    act(w, "$n starts following you.", true, ch, None, Vict::Ch(leader), TO_VICT);
    act(w, "$n now follows $N.", true, ch, None, Vict::Ch(leader), TO_NOTVICT);
}

pub fn say_spell(w: &mut World, ch: CharId, si: i32) {
    const SYLS: &[(&str, &str)] = &[
        (" ", " "),
        ("ar", "abra"),
        ("au", "kada"),
        ("bless", "fido"),
        ("blind", "nose"),
        ("bur", "mosa"),
        ("cu", "judi"),
        ("de", "oculo"),
        ("en", "unso"),
        ("light", "dies"),
        ("lo", "hi"),
        ("mor", "zak"),
        ("move", "sido"),
        ("ness", "lacri"),
        ("ning", "illa"),
        ("per", "duda"),
        ("ra", "gru"),
        ("re", "candus"),
        ("son", "sabru"),
        ("tect", "infra"),
        ("tri", "cula"),
        ("ven", "nofo"),
        ("a", "a"),
        ("b", "b"),
        ("c", "q"),
        ("d", "e"),
        ("e", "z"),
        ("f", "y"),
        ("g", "o"),
        ("h", "p"),
        ("i", "u"),
        ("j", "y"),
        ("k", "t"),
        ("l", "r"),
        ("m", "w"),
        ("n", "i"),
        ("o", "a"),
        ("p", "s"),
        ("q", "d"),
        ("r", "f"),
        ("s", "g"),
        ("t", "h"),
        ("u", "j"),
        ("v", "z"),
        ("w", "x"),
        ("x", "n"),
        ("y", "l"),
        ("z", "k"),
    ];
    let splwd = SPELLS[(si - 1) as usize];
    let mut buf = String::new();
    let mut offs = 0;
    while offs < splwd.len() {
        // Like the C loop: one pass over the table may match several times.
        let before = offs;
        for (org, new) in SYLS {
            if splwd[offs..].starts_with(org) {
                buf.push_str(new);
                offs += org.len();
            }
        }
        if offs == before {
            offs += 1; // unknown character (the C code would spin forever)
        }
    }
    let buf2 = format!("$n utters the words, '{}'", buf);
    let buf1 = format!("$n utters the words, '{}'", splwd);
    for t in w.room(w[ch].in_room).people.clone() {
        if t != ch {
            if w[ch].player.class == w[t].player.class {
                act(w, &buf1, false, ch, None, Vict::Ch(t), TO_VICT);
            } else {
                act(w, &buf2, false, ch, None, Vict::Ch(t), TO_VICT);
            }
        }
    }
}

pub fn saves_spell(w: &World, ch: CharId, save_type: usize) -> bool {
    let mut save = w[ch].specials.apply_saving_throw[save_type];
    if !w.is_npc(ch) {
        let cls = (w[ch].player.class - 1).clamp(0, 3) as usize;
        save += SAVING_THROWS[cls][save_type][w.level(ch).clamp(0, 24) as usize];
        if w.level(ch) > 20 {
            return true;
        }
    }
    save.max(1) < number(1, 20)
}

pub fn do_cast(w: &mut World, ch: CharId, argument: &str, _cmd: i32) {
    if w.is_npc(ch) {
        return;
    }
    if w.level(ch) < 21 {
        match w[ch].player.class {
            CLASS_WARRIOR => {
                send_to_char(w, "Think you had better stick to fighting...\n\r", ch);
                return;
            }
            CLASS_THIEF => {
                send_to_char(w, "Think you should stick to robbing and killing...\n\r", ch);
                return;
            }
            _ => {}
        }
    }
    let argument = argument.trim_start_matches(' ');
    if argument.is_empty() {
        send_to_char(w, "Cast which what where?\n\r", ch);
        return;
    }
    if !argument.starts_with('\'') {
        send_to_char(w, "Magic must always be enclosed by the holy magic symbols : '\n\r", ch);
        return;
    }
    let Some(qend) = argument[1..].find('\'').map(|p| p + 1) else {
        send_to_char(w, "Magic must always be enclosed by the holy magic symbols : '\n\r", ch);
        return;
    };
    let words = argument[1..qend].to_ascii_lowercase();
    // old_search_block(argument, 1, qend-1, spells, 0): prefix match
    let spl: i32 = if words.is_empty() {
        0
    } else {
        SPELLS.iter().position(|s| s.starts_with(&words)).map(|p| p as i32 + 1).unwrap_or(-1)
    };
    if spl == 0 {
        send_to_char(w, "Your lips do not move, no magic appears.\n\r", ch);
        return;
    }
    let si = spell_info(spl);
    if spl > 0 && spl <= 44 && si.spell_pointer.is_some() {
        if w.get_pos(ch) < si.minimum_position {
            let msg = match w.get_pos(ch) {
                POSITION_SLEEPING => "You dream about great magical powers.\n\r",
                POSITION_RESTING => "You can't concentrate enough while resting.\n\r",
                POSITION_SITTING => "You can't do this sitting!\n\r",
                POSITION_FIGHTING => "Impossible! You can't concentrate enough!.\n\r",
                _ => "It seems like you're in a pretty bad shape!\n\r",
            };
            send_to_char(w, msg, ch);
            return;
        }
        if w.level(ch) < 21 {
            let cls = w[ch].player.class;
            if (cls == CLASS_MAGIC_USER && si.min_level_magic > w.level(ch))
                || (cls == CLASS_CLERIC && si.min_level_cleric > w.level(ch))
            {
                send_to_char(w, "Sorry, you can't do that.\n\r", ch);
                return;
            }
        }
        let rest = argument[qend + 1..].trim_start_matches(' ');
        let mut target_ok = false;
        let mut tar_char: Option<CharId> = None;
        let mut tar_obj: Option<ObjId> = None;
        let mut name = String::new();
        let mut rest_after = rest;

        if si.targets & TAR_IGNORE == 0 {
            let (n, r) = one_argument(rest);
            name = n;
            rest_after = r;
            if !name.is_empty() {
                if si.targets & TAR_CHAR_ROOM != 0 {
                    if let Some(c) = get_char_room_vis(w, ch, &name) {
                        tar_char = Some(c);
                        target_ok = true;
                    }
                }
                if !target_ok && si.targets & TAR_CHAR_WORLD != 0 {
                    if let Some(c) = get_char_vis(w, ch, &name) {
                        tar_char = Some(c);
                        target_ok = true;
                    }
                }
                if !target_ok && si.targets & TAR_OBJ_INV != 0 {
                    if let Some(o) = get_obj_in_list_vis(w, ch, &name, &w[ch].carrying) {
                        tar_obj = Some(o);
                        target_ok = true;
                    }
                }
                if !target_ok && si.targets & TAR_OBJ_ROOM != 0 {
                    if let Some(o) = get_obj_in_list_vis(w, ch, &name, &w.room(w[ch].in_room).contents) {
                        tar_obj = Some(o);
                        target_ok = true;
                    }
                }
                if !target_ok && si.targets & TAR_OBJ_WORLD != 0 {
                    if let Some(o) = get_obj_vis(w, ch, &name) {
                        tar_obj = Some(o);
                        target_ok = true;
                    }
                }
                if !target_ok && si.targets & TAR_OBJ_EQUIP != 0 {
                    for i in 0..MAX_WEAR {
                        if let Some(o) = w[ch].equipment[i] {
                            if str_cmp(&name, &w[o].name) == 0 {
                                tar_obj = Some(o);
                                target_ok = true;
                                break;
                            }
                        }
                    }
                }
                if !target_ok && si.targets & TAR_SELF_ONLY != 0 && str_cmp(&w[ch].player.name, &name) == 0 {
                    tar_char = Some(ch);
                    target_ok = true;
                }
            } else {
                if si.targets & TAR_FIGHT_SELF != 0 && w[ch].specials.fighting.is_some() {
                    tar_char = Some(ch);
                    target_ok = true;
                }
                if !target_ok && si.targets & TAR_FIGHT_VICT != 0 {
                    if let Some(f) = w[ch].specials.fighting {
                        tar_char = Some(f);
                        target_ok = true;
                    }
                }
                if !target_ok && si.targets & TAR_SELF_ONLY != 0 {
                    tar_char = Some(ch);
                    target_ok = true;
                }
            }
        } else {
            target_ok = true;
        }

        if !target_ok {
            if !name.is_empty() {
                let t = si.targets;
                let msg = if t & TAR_CHAR_ROOM != 0 {
                    "Nobody here by that name.\n\r"
                } else if t & TAR_CHAR_WORLD != 0 {
                    "Nobody playing by that name.\n\r"
                } else if t & TAR_OBJ_INV != 0 {
                    "You are not carrying anything like that.\n\r"
                } else if t & TAR_OBJ_ROOM != 0 {
                    "Nothing here by that name.\n\r"
                } else if t & TAR_OBJ_WORLD != 0 {
                    "Nothing at all by that name.\n\r"
                } else if t & TAR_OBJ_EQUIP != 0 {
                    "You are not wearing anything like that.\n\r"
                } else {
                    ""
                };
                send_to_char(w, msg, ch);
            } else if si.targets < TAR_OBJ_INV {
                send_to_char(w, "Who should the spell be cast upon?\n\r", ch);
            } else {
                send_to_char(w, "What should the spell be cast upon?\n\r", ch);
            }
            return;
        }
        if tar_char == Some(ch) && si.targets & TAR_SELF_NONO != 0 {
            send_to_char(w, "You can not cast this spell upon yourself.\n\r", ch);
            return;
        } else if tar_char != Some(ch) && si.targets & TAR_SELF_ONLY != 0 {
            send_to_char(w, "You can only cast this spell upon yourself.\n\r", ch);
            return;
        } else if w.is_affected(ch, AFF_CHARM) && w[ch].master == tar_char && tar_char.is_some() {
            send_to_char(w, "You are afraid that it could harm your master.\n\r", ch);
            return;
        }

        let mana = use_mana(w, ch, spl);
        if w.level(ch) < 21 && w[ch].points.mana < mana {
            send_to_char(w, "You can't summon enough energy to cast the spell.\n\r", ch);
            return;
        }
        if spl != SPELL_VENTRILOQUATE {
            say_spell(w, ch, spl);
        }
        w.wait_state(ch, si.beats);
        let Some(f) = si.spell_pointer else {
            send_to_char(w, "Sorry, this magic has not yet been implemented :(\n\r", ch);
            return;
        };
        if number(1, 101) > w[ch].skills[spl as usize].learned {
            send_to_char(w, "You lost your concentration!\n\r", ch);
            w[ch].points.mana -= mana >> 1;
            return;
        }
        send_to_char(w, "Ok.\n\r", ch);
        let level = w.level(ch);
        // ventriloquate needs the text after the target name
        let rest_owned = rest_after.to_string();
        f(w, level, ch, &rest_owned, SPELL_TYPE_SPELL, tar_char, tar_obj);
        if w.ch_exists(ch) {
            w[ch].points.mana -= mana;
        }
        return;
    }
    let msg = match number(1, 5) {
        1 => "Bylle Grylle Grop Gryf???\n\r",
        2 => "Olle Bolle Snop Snyf?\n\r",
        3 => "Olle Grylle Bolle Bylle?!?\n\r",
        4 => "Gryffe Olle Gnyffe Snop???\n\r",
        _ => "Bolle Snylle Gryf Bylle?!!?\n\r",
    };
    send_to_char(w, msg, ch);
}
