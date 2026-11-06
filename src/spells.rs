//! Spell numbers (spells.h) and the `cast_*` front ends (spells1.c,
//! spells2.c) that decide what each spell does when cast, quaffed,
//! recited, zapped from a wand or used from a staff.

#![allow(dead_code)]

use crate::comm::*;
use crate::game::World;
use crate::handler::fname;
use crate::interpreter::one_argument;
use crate::magic::*;
use crate::spell_parser::saves_spell;
use crate::structs::*;
use crate::utils::*;

pub const TYPE_UNDEFINED: i32 = -1;
pub const SPELL_RESERVED_DBC: i32 = 0;
pub const SPELL_ARMOR: i32 = 1;
pub const SPELL_TELEPORT: i32 = 2;
pub const SPELL_BLESS: i32 = 3;
pub const SPELL_BLINDNESS: i32 = 4;
pub const SPELL_BURNING_HANDS: i32 = 5;
pub const SPELL_CALL_LIGHTNING: i32 = 6;
pub const SPELL_CHARM_PERSON: i32 = 7;
pub const SPELL_CHILL_TOUCH: i32 = 8;
pub const SPELL_CLONE: i32 = 9;
pub const SPELL_COLOUR_SPRAY: i32 = 10;
pub const SPELL_CONTROL_WEATHER: i32 = 11;
pub const SPELL_CREATE_FOOD: i32 = 12;
pub const SPELL_CREATE_WATER: i32 = 13;
pub const SPELL_CURE_BLIND: i32 = 14;
pub const SPELL_CURE_CRITIC: i32 = 15;
pub const SPELL_CURE_LIGHT: i32 = 16;
pub const SPELL_CURSE: i32 = 17;
pub const SPELL_DETECT_EVIL: i32 = 18;
pub const SPELL_DETECT_INVISIBLE: i32 = 19;
pub const SPELL_DETECT_MAGIC: i32 = 20;
pub const SPELL_DETECT_POISON: i32 = 21;
pub const SPELL_DISPEL_EVIL: i32 = 22;
pub const SPELL_EARTHQUAKE: i32 = 23;
pub const SPELL_ENCHANT_WEAPON: i32 = 24;
pub const SPELL_ENERGY_DRAIN: i32 = 25;
pub const SPELL_FIREBALL: i32 = 26;
pub const SPELL_HARM: i32 = 27;
pub const SPELL_HEAL: i32 = 28;
pub const SPELL_INVISIBLE: i32 = 29;
pub const SPELL_LIGHTNING_BOLT: i32 = 30;
pub const SPELL_LOCATE_OBJECT: i32 = 31;
pub const SPELL_MAGIC_MISSILE: i32 = 32;
pub const SPELL_POISON: i32 = 33;
pub const SPELL_PROTECT_FROM_EVIL: i32 = 34;
pub const SPELL_REMOVE_CURSE: i32 = 35;
pub const SPELL_SANCTUARY: i32 = 36;
pub const SPELL_SHOCKING_GRASP: i32 = 37;
pub const SPELL_SLEEP: i32 = 38;
pub const SPELL_STRENGTH: i32 = 39;
pub const SPELL_SUMMON: i32 = 40;
pub const SPELL_VENTRILOQUATE: i32 = 41;
pub const SPELL_WORD_OF_RECALL: i32 = 42;
pub const SPELL_REMOVE_POISON: i32 = 43;
pub const SPELL_SENSE_LIFE: i32 = 44;
pub const SKILL_SNEAK: i32 = 45;
pub const SKILL_HIDE: i32 = 46;
pub const SKILL_STEAL: i32 = 47;
pub const SKILL_BACKSTAB: i32 = 48;
pub const SKILL_PICK_LOCK: i32 = 49;
pub const SKILL_KICK: i32 = 50;
pub const SKILL_BASH: i32 = 51;
pub const SKILL_RESCUE: i32 = 52;
pub const SPELL_IDENTIFY: i32 = 53;
pub const SPELL_ANIMATE_DEAD: i32 = 54;
pub const SPELL_FEAR: i32 = 55;
pub const SPELL_FIRE_BREATH: i32 = 56;
pub const SPELL_GAS_BREATH: i32 = 57;
pub const SPELL_FROST_BREATH: i32 = 58;
pub const SPELL_ACID_BREATH: i32 = 59;
pub const SPELL_LIGHTNING_BREATH: i32 = 60;

pub const TYPE_HIT: i32 = 100;
pub const TYPE_BLUDGEON: i32 = 101;
pub const TYPE_PIERCE: i32 = 102;
pub const TYPE_SLASH: i32 = 103;
pub const TYPE_WHIP: i32 = 104;
pub const TYPE_CLAW: i32 = 105;
pub const TYPE_BITE: i32 = 106;
pub const TYPE_STING: i32 = 107;
pub const TYPE_CRUSH: i32 = 108;
pub const TYPE_SUFFERING: i32 = 200;

pub const MAX_TYPES: i32 = 70;

pub const SAVING_PARA: usize = 0;
pub const SAVING_ROD: usize = 1;
pub const SAVING_PETRI: usize = 2;
pub const SAVING_BREATH: usize = 3;
pub const SAVING_SPELL: usize = 4;

pub const MAX_SPL_LIST: usize = 127;

pub const TAR_IGNORE: i32 = 1;
pub const TAR_CHAR_ROOM: i32 = 2;
pub const TAR_CHAR_WORLD: i32 = 4;
pub const TAR_FIGHT_SELF: i32 = 8;
pub const TAR_FIGHT_VICT: i32 = 16;
pub const TAR_SELF_ONLY: i32 = 32;
pub const TAR_SELF_NONO: i32 = 64;
pub const TAR_OBJ_INV: i32 = 128;
pub const TAR_OBJ_ROOM: i32 = 256;
pub const TAR_OBJ_WORLD: i32 = 512;
pub const TAR_OBJ_EQUIP: i32 = 1024;

pub const SPELL_TYPE_SPELL: i32 = 0;
pub const SPELL_TYPE_POTION: i32 = 1;
pub const SPELL_TYPE_WAND: i32 = 2;
pub const SPELL_TYPE_STAFF: i32 = 3;
pub const SPELL_TYPE_SCROLL: i32 = 4;

/// `void (*spell_pointer)(byte level, ch, arg, type, tar_ch, tar_obj)`
pub type CastFn = fn(&mut World, i32, CharId, &str, i32, Option<CharId>, Option<ObjId>);

type SpellImpl = fn(&mut World, i32, CharId, Option<CharId>, Option<ObjId>);

/// Everybody in the caster's room except the caster (staff effects).
fn others(w: &World, ch: CharId) -> Vec<CharId> {
    w.room(w[ch].in_room).people.iter().copied().filter(|&c| c != ch).collect()
}

fn staff(w: &mut World, level: i32, ch: CharId, f: SpellImpl, filter: Option<fn(&World, CharId) -> bool>) {
    for v in others(w, ch) {
        if !w.ch_exists(v) || !w.ch_exists(ch) || w[v].in_room != w[ch].in_room {
            continue;
        }
        if let Some(fl) = filter {
            if !fl(w, v) {
                continue;
            }
        }
        f(w, level, ch, Some(v), None);
    }
}

fn screwup(what: &str) {
    log(&format!("Serious screw-up in {}!", what));
}

/* ---------------------------- spells1.c ---------------------------- */

pub fn cast_burning_hands(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, v: Option<CharId>, _o: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL => spell_burning_hands(w, level, ch, v, None),
        _ => screwup("burning hands"),
    }
}

pub fn cast_call_lightning(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, v: Option<CharId>, o: Option<ObjId>) {
    let ok = w.outside(ch) && w.weather_info.sky >= SKY_RAINING;
    match t {
        SPELL_TYPE_SPELL => {
            if ok {
                spell_call_lightning(w, level, ch, v, None);
            } else {
                send_to_char(w, "You fail to call upon the lightning from the sky!\n\r", ch);
            }
        }
        SPELL_TYPE_POTION => {
            if ok {
                spell_call_lightning(w, level, ch, Some(ch), None);
            }
        }
        SPELL_TYPE_SCROLL => {
            if ok {
                if v.is_some() {
                    spell_call_lightning(w, level, ch, v, None);
                } else if o.is_none() {
                    spell_call_lightning(w, level, ch, Some(ch), None);
                }
            }
        }
        SPELL_TYPE_STAFF => {
            if ok {
                staff(w, level, ch, spell_call_lightning, None);
            }
        }
        _ => screwup("call lightning"),
    }
}

pub fn cast_chill_touch(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, v: Option<CharId>, _o: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL => spell_chill_touch(w, level, ch, v, None),
        _ => screwup("chill touch"),
    }
}

pub fn cast_shocking_grasp(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, v: Option<CharId>, _o: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL => spell_shocking_grasp(w, level, ch, v, None),
        _ => screwup("shocking grasp"),
    }
}

/// The common "attack spell" pattern: spell, scroll (victim or self), wand.
fn attack_pattern(w: &mut World, level: i32, ch: CharId, t: i32, v: Option<CharId>, o: Option<ObjId>, f: SpellImpl, name: &str) {
    match t {
        SPELL_TYPE_SPELL => f(w, level, ch, v, None),
        SPELL_TYPE_SCROLL => {
            if v.is_some() {
                f(w, level, ch, v, None);
            } else if o.is_none() {
                f(w, level, ch, Some(ch), None);
            }
        }
        SPELL_TYPE_WAND => {
            if v.is_some() {
                f(w, level, ch, v, None);
            }
        }
        _ => screwup(name),
    }
}

pub fn cast_colour_spray(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, v: Option<CharId>, o: Option<ObjId>) {
    attack_pattern(w, level, ch, t, v, o, spell_colour_spray, "colour spray");
}

pub fn cast_earthquake(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, _v: Option<CharId>, _o: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL | SPELL_TYPE_SCROLL | SPELL_TYPE_STAFF => spell_earthquake(w, level, ch, None, None),
        _ => screwup("earthquake"),
    }
}

pub fn cast_energy_drain(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, v: Option<CharId>, o: Option<ObjId>) {
    match t {
        SPELL_TYPE_POTION => spell_energy_drain(w, level, ch, Some(ch), None),
        SPELL_TYPE_STAFF => staff(w, level, ch, spell_energy_drain, None),
        _ => attack_pattern(w, level, ch, t, v, o, spell_energy_drain, "energy drain"),
    }
}

pub fn cast_fireball(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, v: Option<CharId>, o: Option<ObjId>) {
    attack_pattern(w, level, ch, t, v, o, spell_fireball, "fireball");
}

pub fn cast_harm(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, v: Option<CharId>, _o: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL => spell_harm(w, level, ch, v, None),
        SPELL_TYPE_POTION => spell_harm(w, level, ch, Some(ch), None),
        SPELL_TYPE_STAFF => staff(w, level, ch, spell_harm, None),
        _ => screwup("harm"),
    }
}

pub fn cast_lightning_bolt(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, v: Option<CharId>, o: Option<ObjId>) {
    attack_pattern(w, level, ch, t, v, o, spell_lightning_bolt, "lightning bolt");
}

pub fn cast_magic_missile(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, v: Option<CharId>, o: Option<ObjId>) {
    attack_pattern(w, level, ch, t, v, o, spell_magic_missile, "magic missile");
}

/* ---------------------------- spells2.c ---------------------------- */

pub fn cast_armor(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, tar_ch: Option<CharId>, tar_obj: Option<ObjId>) {
    use crate::handler::affected_by_spell as abs;
    match t {
        SPELL_TYPE_SPELL => {
            let Some(tc) = tar_ch else { return };
            if abs(w, tc, SPELL_ARMOR) {
                send_to_char(w, "Nothing seems to happen.\n\r", ch);
                return;
            }
            if ch != tc {
                act(w, "$N is protected by your deity.", false, ch, None, Vict::Ch(tc), TO_CHAR);
            }
            spell_armor(w, level, ch, Some(tc), None);
        }
        SPELL_TYPE_POTION => {
            if abs(w, ch, SPELL_ARMOR) {
                return;
            }
            spell_armor(w, level, ch, Some(ch), None);
        }
        SPELL_TYPE_SCROLL => {
            if tar_obj.is_some() {
                return;
            }
            let tc = tar_ch.unwrap_or(ch);
            if abs(w, tc, SPELL_ARMOR) {
                return;
            }
            spell_armor(w, level, ch, Some(ch), None);
        }
        SPELL_TYPE_WAND => {
            if tar_obj.is_some() {
                return;
            }
            let Some(tc) = tar_ch else { return };
            if abs(w, tc, SPELL_ARMOR) {
                return;
            }
            spell_armor(w, level, ch, Some(ch), None);
        }
        _ => screwup("armor"),
    }
}

pub fn cast_teleport(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, tar_ch: Option<CharId>, _o: Option<ObjId>) {
    match t {
        SPELL_TYPE_SCROLL | SPELL_TYPE_POTION | SPELL_TYPE_SPELL => {
            spell_teleport(w, level, ch, Some(tar_ch.unwrap_or(ch)), None)
        }
        SPELL_TYPE_WAND => {
            if tar_ch.is_some() {
                spell_teleport(w, level, ch, tar_ch, None);
            }
        }
        SPELL_TYPE_STAFF => staff(w, level, ch, spell_teleport, None),
        _ => screwup("teleport"),
    }
}

pub fn cast_bless(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, tar_ch: Option<CharId>, tar_obj: Option<ObjId>) {
    use crate::handler::affected_by_spell as abs;
    let ch_ok = |w: &World, c: CharId| !(abs(w, c, SPELL_BLESS) || w.get_pos(c) == POSITION_FIGHTING);
    match t {
        SPELL_TYPE_SPELL => {
            if let Some(o) = tar_obj {
                if w.is_obj_stat(o, ITEM_BLESS) {
                    send_to_char(w, "Nothing seems to happen.\n\r", ch);
                    return;
                }
                spell_bless(w, level, ch, None, Some(o));
            } else if let Some(tc) = tar_ch {
                if !ch_ok(w, tc) {
                    send_to_char(w, "Nothing seems to happen.\n\r", ch);
                    return;
                }
                spell_bless(w, level, ch, Some(tc), None);
            }
        }
        SPELL_TYPE_POTION => {
            if ch_ok(w, ch) {
                spell_bless(w, level, ch, Some(ch), None);
            }
        }
        SPELL_TYPE_SCROLL | SPELL_TYPE_WAND => {
            if let Some(o) = tar_obj {
                if !w.is_obj_stat(o, ITEM_BLESS) {
                    spell_bless(w, level, ch, None, Some(o));
                }
            } else {
                let tc = if t == SPELL_TYPE_SCROLL { tar_ch.unwrap_or(ch) } else {
                    match tar_ch {
                        Some(c) => c,
                        None => return,
                    }
                };
                if ch_ok(w, tc) {
                    spell_bless(w, level, ch, Some(tc), None);
                }
            }
        }
        _ => screwup("bless"),
    }
}

pub fn cast_blindness(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, tar_ch: Option<CharId>, tar_obj: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL => {
            let Some(tc) = tar_ch else { return };
            if w.is_affected(tc, AFF_BLIND) {
                send_to_char(w, "Nothing seems to happen.\n\r", ch);
                return;
            }
            spell_blindness(w, level, ch, Some(tc), None);
        }
        SPELL_TYPE_POTION | SPELL_TYPE_SCROLL | SPELL_TYPE_WAND => {
            if t != SPELL_TYPE_POTION && tar_obj.is_some() {
                return;
            }
            if w.is_affected(ch, AFF_BLIND) {
                return;
            }
            spell_blindness(w, level, ch, Some(ch), None);
        }
        SPELL_TYPE_STAFF => staff(w, level, ch, spell_blindness, Some(|w, v| !w.is_affected(v, AFF_BLIND))),
        _ => screwup("blindness"),
    }
}

pub fn cast_clone(w: &mut World, _level: i32, ch: CharId, _a: &str, _t: i32, _v: Option<CharId>, _o: Option<ObjId>) {
    send_to_char(w, "Not *YET* implemented.", ch);
}

pub fn cast_control_weather(w: &mut World, level: i32, ch: CharId, arg: &str, t: i32, _v: Option<CharId>, _o: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL => {
            let (buffer, _) = one_argument(arg);
            if str_cmp("better", &buffer) != 0 && str_cmp("worse", &buffer) != 0 {
                send_to_char(w, "Do you want it to get better or worse?\n\r", ch);
                return;
            }
            if str_cmp("better", &buffer) == 0 {
                w.weather_info.change += dice(level / 3, 4);
            } else {
                w.weather_info.change -= dice(level / 3, 4);
            }
        }
        _ => screwup("control weather"),
    }
}

pub fn cast_create_food(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, tar_ch: Option<CharId>, tar_obj: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL => {
            act(w, "$n magically creates a mushroom.", false, ch, None, Vict::None, TO_ROOM);
            spell_create_food(w, level, ch, None, None);
        }
        SPELL_TYPE_SCROLL => {
            if tar_obj.is_some() || tar_ch.is_some() {
                return;
            }
            spell_create_food(w, level, ch, None, None);
        }
        _ => screwup("create food"),
    }
}

pub fn cast_create_water(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, _v: Option<CharId>, tar_obj: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL => {
            let Some(o) = tar_obj else { return };
            if w[o].obj_flags.type_flag != ITEM_DRINKCON {
                send_to_char(w, "It is unable to hold water.\n\r", ch);
                return;
            }
            spell_create_water(w, level, ch, None, Some(o));
        }
        _ => screwup("create water"),
    }
}

/// spell / potion / staff pattern used by the healing spells.
fn heal_pattern(w: &mut World, level: i32, ch: CharId, t: i32, tar_ch: Option<CharId>, f: SpellImpl, name: &str) {
    match t {
        SPELL_TYPE_SPELL => f(w, level, ch, tar_ch, None),
        SPELL_TYPE_POTION => f(w, level, ch, Some(ch), None),
        SPELL_TYPE_STAFF => staff(w, level, ch, f, None),
        _ => screwup(name),
    }
}

pub fn cast_cure_blind(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, v: Option<CharId>, _o: Option<ObjId>) {
    heal_pattern(w, level, ch, t, v, spell_cure_blind, "cure blind");
}

pub fn cast_cure_critic(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, v: Option<CharId>, _o: Option<ObjId>) {
    heal_pattern(w, level, ch, t, v, spell_cure_critic, "cure critic");
}

pub fn cast_cure_light(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, v: Option<CharId>, _o: Option<ObjId>) {
    heal_pattern(w, level, ch, t, v, spell_cure_light, "cure light");
}

pub fn cast_curse(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, tar_ch: Option<CharId>, tar_obj: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL => {
            if tar_obj.is_some() {
                spell_curse(w, level, ch, None, tar_obj);
            } else {
                spell_curse(w, level, ch, tar_ch, None);
            }
        }
        SPELL_TYPE_POTION => spell_curse(w, level, ch, Some(ch), None),
        SPELL_TYPE_SCROLL => {
            if tar_obj.is_some() {
                spell_curse(w, level, ch, None, tar_obj);
            } else {
                spell_curse(w, level, ch, Some(tar_ch.unwrap_or(ch)), None);
            }
        }
        SPELL_TYPE_STAFF => staff(w, level, ch, spell_curse, None),
        _ => screwup("curse"),
    }
}

/// The "detect" spells: refuse when already affected.
fn detect_pattern(w: &mut World, level: i32, ch: CharId, t: i32, tar_ch: Option<CharId>, spell: i32, f: SpellImpl, name: &str) {
    use crate::handler::affected_by_spell as abs;
    match t {
        SPELL_TYPE_SPELL => {
            let Some(tc) = tar_ch else { return };
            if abs(w, tc, spell) {
                send_to_char(w, "Nothing seems to happen.\n\r", tc);
                return;
            }
            f(w, level, ch, Some(tc), None);
        }
        SPELL_TYPE_POTION => {
            if abs(w, ch, spell) {
                return;
            }
            f(w, level, ch, Some(ch), None);
        }
        SPELL_TYPE_STAFF => {
            // the C code tested IS_AFFECTED(tar_ch, SPELL_XXX) - i.e. the
            // spell number used as a bit mask.  Kept as it was.
            for v in others(w, ch) {
                if w.ch_exists(v) && !w.is_affected(v, spell as i64) {
                    f(w, level, ch, Some(v), None);
                }
            }
        }
        _ => screwup(name),
    }
}

pub fn cast_detect_evil(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, v: Option<CharId>, _o: Option<ObjId>) {
    detect_pattern(w, level, ch, t, v, SPELL_DETECT_EVIL, spell_detect_evil, "detect evil");
}

pub fn cast_detect_invisibility(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, v: Option<CharId>, _o: Option<ObjId>) {
    detect_pattern(w, level, ch, t, v, SPELL_DETECT_INVISIBLE, spell_detect_invisibility, "detect invisibility");
}

pub fn cast_detect_magic(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, v: Option<CharId>, _o: Option<ObjId>) {
    detect_pattern(w, level, ch, t, v, SPELL_DETECT_MAGIC, spell_detect_magic, "detect magic");
}

pub fn cast_detect_poison(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, tar_ch: Option<CharId>, tar_obj: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL => spell_detect_poison(w, level, ch, tar_ch, tar_obj),
        SPELL_TYPE_POTION => spell_detect_poison(w, level, ch, Some(ch), None),
        SPELL_TYPE_SCROLL => {
            if tar_obj.is_some() {
                spell_detect_poison(w, level, ch, None, tar_obj);
                return;
            }
            spell_detect_poison(w, level, ch, Some(tar_ch.unwrap_or(ch)), None);
        }
        _ => screwup("detect poison"),
    }
}

pub fn cast_dispel_evil(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, tar_ch: Option<CharId>, tar_obj: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL => spell_dispel_evil(w, level, ch, tar_ch, None),
        SPELL_TYPE_POTION => spell_dispel_evil(w, level, ch, Some(ch), None),
        SPELL_TYPE_SCROLL => {
            if tar_obj.is_some() {
                return;
            }
            spell_dispel_evil(w, level, ch, Some(tar_ch.unwrap_or(ch)), None);
        }
        SPELL_TYPE_WAND => {
            if tar_obj.is_some() {
                return;
            }
            spell_dispel_evil(w, level, ch, tar_ch, None);
        }
        SPELL_TYPE_STAFF => staff(w, level, ch, spell_dispel_evil, None),
        _ => screwup("dispel evil"),
    }
}

pub fn cast_enchant_weapon(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, _v: Option<CharId>, tar_obj: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL | SPELL_TYPE_SCROLL => {
            if tar_obj.is_some() {
                spell_enchant_weapon(w, level, ch, None, tar_obj);
            }
        }
        _ => screwup("enchant weapon"),
    }
}

pub fn cast_heal(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, tar_ch: Option<CharId>, _o: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL => {
            let Some(tc) = tar_ch else { return };
            act(w, "$n heals $N.", false, ch, None, Vict::Ch(tc), TO_NOTVICT);
            act(w, "You heal $N.", false, ch, None, Vict::Ch(tc), TO_CHAR);
            spell_heal(w, level, ch, Some(tc), None);
        }
        SPELL_TYPE_POTION => spell_heal(w, level, ch, Some(ch), None),
        SPELL_TYPE_STAFF => staff(w, level, ch, spell_heal, None),
        _ => screwup("heal"),
    }
}

pub fn cast_invisibility(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, tar_ch: Option<CharId>, tar_obj: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL => {
            if let Some(o) = tar_obj {
                if w.is_obj_stat(o, ITEM_INVISIBLE) {
                    send_to_char(w, "Nothing new seems to happen.\n\r", ch);
                } else {
                    spell_invisibility(w, level, ch, None, Some(o));
                }
            } else if let Some(tc) = tar_ch {
                if w.is_affected(tc, AFF_INVISIBLE) {
                    send_to_char(w, "Nothing new seems to happen.\n\r", ch);
                } else {
                    spell_invisibility(w, level, ch, Some(tc), None);
                }
            }
        }
        SPELL_TYPE_POTION => {
            if !w.is_affected(ch, AFF_INVISIBLE) {
                spell_invisibility(w, level, ch, Some(ch), None);
            }
        }
        SPELL_TYPE_SCROLL | SPELL_TYPE_WAND => {
            if let Some(o) = tar_obj {
                if !w.is_obj_stat(o, ITEM_INVISIBLE) {
                    spell_invisibility(w, level, ch, None, Some(o));
                }
            } else {
                let tc = if t == SPELL_TYPE_SCROLL { Some(tar_ch.unwrap_or(ch)) } else { tar_ch };
                if let Some(tc) = tc {
                    if !w.is_affected(tc, AFF_INVISIBLE) {
                        spell_invisibility(w, level, ch, Some(tc), None);
                    }
                }
            }
        }
        SPELL_TYPE_STAFF => staff(w, level, ch, spell_invisibility, Some(|w, v| !w.is_affected(v, AFF_INVISIBLE))),
        _ => screwup("invisibility"),
    }
}

pub fn cast_locate_object(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, _v: Option<CharId>, tar_obj: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL => spell_locate_object(w, level, ch, None, tar_obj),
        _ => screwup("locate object"),
    }
}

pub fn cast_poison(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, tar_ch: Option<CharId>, tar_obj: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL => spell_poison(w, level, ch, tar_ch, tar_obj),
        SPELL_TYPE_POTION => spell_poison(w, level, ch, Some(ch), None),
        SPELL_TYPE_STAFF => staff(w, level, ch, spell_poison, None),
        _ => screwup("poison"),
    }
}

/// spell / potion / scroll (victim or self, not objects) / staff.
fn buff_pattern(w: &mut World, level: i32, ch: CharId, t: i32, tar_ch: Option<CharId>, tar_obj: Option<ObjId>, f: SpellImpl, name: &str) {
    match t {
        SPELL_TYPE_SPELL => f(w, level, ch, tar_ch, None),
        SPELL_TYPE_POTION => f(w, level, ch, Some(ch), None),
        SPELL_TYPE_SCROLL => {
            if tar_obj.is_some() {
                return;
            }
            f(w, level, ch, Some(tar_ch.unwrap_or(ch)), None);
        }
        SPELL_TYPE_STAFF => staff(w, level, ch, f, None),
        _ => screwup(name),
    }
}

pub fn cast_protection_from_evil(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, v: Option<CharId>, o: Option<ObjId>) {
    buff_pattern(w, level, ch, t, v, o, spell_protection_from_evil, "protection from evil");
}

pub fn cast_remove_curse(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, tar_ch: Option<CharId>, tar_obj: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL => spell_remove_curse(w, level, ch, tar_ch, tar_obj),
        SPELL_TYPE_POTION => spell_remove_curse(w, level, ch, Some(ch), None),
        SPELL_TYPE_SCROLL => {
            if tar_obj.is_some() {
                spell_remove_curse(w, level, ch, None, tar_obj);
                return;
            }
            spell_remove_curse(w, level, ch, Some(tar_ch.unwrap_or(ch)), None);
        }
        SPELL_TYPE_STAFF => staff(w, level, ch, spell_remove_curse, None),
        _ => screwup("remove curse"),
    }
}

pub fn cast_remove_poison(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, tar_ch: Option<CharId>, tar_obj: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL => spell_remove_poison(w, level, ch, tar_ch, tar_obj),
        SPELL_TYPE_POTION => spell_remove_poison(w, level, ch, Some(ch), None),
        SPELL_TYPE_STAFF => staff(w, level, ch, spell_remove_poison, None),
        _ => screwup("remove poison"),
    }
}

pub fn cast_sanctuary(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, v: Option<CharId>, o: Option<ObjId>) {
    buff_pattern(w, level, ch, t, v, o, spell_sanctuary, "sanctuary");
}

pub fn cast_sleep(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, tar_ch: Option<CharId>, tar_obj: Option<ObjId>) {
    match t {
        SPELL_TYPE_WAND => {
            if tar_obj.is_some() {
                return;
            }
            spell_sleep(w, level, ch, tar_ch, None);
        }
        _ => buff_pattern(w, level, ch, t, tar_ch, tar_obj, spell_sleep, "sleep"),
    }
}

pub fn cast_strength(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, v: Option<CharId>, o: Option<ObjId>) {
    buff_pattern(w, level, ch, t, v, o, spell_strength, "strength");
}

pub fn cast_ventriloquate(w: &mut World, _level: i32, ch: CharId, arg: &str, t: i32, tar_ch: Option<CharId>, tar_obj: Option<ObjId>) {
    if t != SPELL_TYPE_SPELL {
        log("Attempt to ventriloquate by non-cast-spell.");
        return;
    }
    let arg = arg.trim_start_matches(' ');
    let (buf1, buf2) = if let Some(o) = tar_obj {
        let n = fname(&w[o].name);
        (
            format!("The {} says '{}'\n\r", n, arg),
            format!("Someone makes it sound like the {} says '{}'.\n\r", n, arg),
        )
    } else if let Some(tc) = tar_ch {
        let n = w[tc].player.name.clone();
        (format!("{} says '{}'\n\r", n, arg), format!("Someone makes it sound like {} says '{}'\n\r", n, arg))
    } else {
        return;
    };
    let buf3 = format!("Someone says, '{}'\n\r", arg);
    for tmp in w.room(w[ch].in_room).people.clone() {
        if tmp != ch && Some(tmp) != tar_ch {
            if saves_spell(w, tmp, SAVING_SPELL) {
                send_to_char(w, &buf2, tmp);
            } else {
                send_to_char(w, &buf1, tmp);
            }
        } else if Some(tmp) == tar_ch {
            send_to_char(w, &buf3, tmp);
        }
    }
}

pub fn cast_word_of_recall(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, tar_ch: Option<CharId>, tar_obj: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL | SPELL_TYPE_POTION => spell_word_of_recall(w, level, ch, Some(ch), None),
        SPELL_TYPE_SCROLL => {
            if tar_obj.is_some() {
                return;
            }
            spell_word_of_recall(w, level, ch, Some(tar_ch.unwrap_or(ch)), None);
        }
        SPELL_TYPE_WAND => {
            if tar_obj.is_some() {
                return;
            }
            spell_word_of_recall(w, level, ch, tar_ch, None);
        }
        SPELL_TYPE_STAFF => staff(w, level, ch, spell_word_of_recall, None),
        _ => screwup("word of recall"),
    }
}

pub fn cast_summon(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, tar_ch: Option<CharId>, _o: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL => spell_summon(w, level, ch, tar_ch, None),
        _ => screwup("summon"),
    }
}

pub fn cast_charm_person(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, tar_ch: Option<CharId>, _o: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL => spell_charm_person(w, level, ch, tar_ch, None),
        SPELL_TYPE_SCROLL => {
            if tar_ch.is_some() {
                spell_charm_person(w, level, ch, tar_ch, None);
            }
        }
        SPELL_TYPE_STAFF => staff(w, level, ch, spell_charm_person, None),
        _ => screwup("charm person"),
    }
}

pub fn cast_sense_life(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, _v: Option<CharId>, _o: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL | SPELL_TYPE_POTION => spell_sense_life(w, level, ch, Some(ch), None),
        SPELL_TYPE_STAFF => staff(w, level, ch, spell_sense_life, None),
        _ => screwup("sense life"),
    }
}

pub fn cast_identify(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, tar_ch: Option<CharId>, tar_obj: Option<ObjId>) {
    match t {
        SPELL_TYPE_SCROLL => spell_identify(w, level, ch, tar_ch, tar_obj),
        _ => screwup("identify"),
    }
}

pub fn cast_fire_breath(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, tar_ch: Option<CharId>, _o: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL => spell_fire_breath(w, level, ch, tar_ch, None),
        _ => screwup("firebreath"),
    }
}

pub fn cast_frost_breath(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, tar_ch: Option<CharId>, _o: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL => spell_frost_breath(w, level, ch, tar_ch, None),
        _ => screwup("frostbreath"),
    }
}

pub fn cast_acid_breath(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, tar_ch: Option<CharId>, _o: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL => spell_acid_breath(w, level, ch, tar_ch, None),
        _ => screwup("acidbreath"),
    }
}

pub fn cast_gas_breath(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, _v: Option<CharId>, _o: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL => staff(w, level, ch, spell_gas_breath, None),
        _ => screwup("gasbreath"),
    }
}

pub fn cast_lightning_breath(w: &mut World, level: i32, ch: CharId, _a: &str, t: i32, tar_ch: Option<CharId>, _o: Option<ObjId>) {
    match t {
        SPELL_TYPE_SPELL => spell_lightning_breath(w, level, ch, tar_ch, None),
        _ => screwup("lightningbreath"),
    }
}
