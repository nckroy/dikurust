//! The spells themselves (magic.c).

use crate::act_info::do_look;
use crate::act_obj2::{name_from_drinkcon, name_to_drinkcon, weight_change_object};
use crate::comm::*;
use crate::constants::*;
use crate::fight::{damage, death_cry, update_pos};
use crate::game::World;
use crate::handler::*;
use crate::limits::{gain_exp, hit_limit};
use crate::spell_parser::{add_follower, circle_follow, saves_spell, stop_follower, SPELLS};
use crate::spells::*;
use crate::structs::*;
use crate::utils::*;

fn lv(level: i32) -> usize {
    level.clamp(0, 30) as usize
}

fn af(type_: i32, duration: i32, modifier: i32, location: i32, bitvector: i64) -> Affect {
    Affect { type_, duration, modifier, location, bitvector }
}

macro_rules! need {
    ($w:expr, $v:expr) => {
        match $v {
            Some(v) if $w.ch_exists(v) => v,
            _ => return,
        }
    };
}

fn simple_attack(w: &mut World, ch: CharId, victim: Option<CharId>, dam: i32, spell: i32) {
    let victim = need!(w, victim);
    let mut dam = dam;
    if saves_spell(w, victim, SAVING_SPELL) {
        dam >>= 1;
    }
    damage(w, ch, victim, dam, spell);
}

pub fn spell_magic_missile(w: &mut World, level: i32, ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    const DAM: [i32; 31] = [0, 3, 3, 4, 4, 5, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6];
    let d = DAM[lv(level)];
    simple_attack(w, ch, victim, number(d >> 1, d << 1), SPELL_MAGIC_MISSILE);
}

pub fn spell_chill_touch(w: &mut World, level: i32, ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    const DAM: [i32; 31] =
        [0, 0, 0, 8, 7, 9, 9, 12, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13];
    let victim = need!(w, victim);
    let d = DAM[lv(level)];
    let mut dam = number(d >> 1, d << 1);
    if !saves_spell(w, victim, SAVING_SPELL) {
        let mut a = af(SPELL_CHILL_TOUCH, 6, -1, APPLY_STR, 0);
        affect_join(w, victim, &mut a, true, false);
    } else {
        dam >>= 1;
    }
    damage(w, ch, victim, dam, SPELL_CHILL_TOUCH);
}

pub fn spell_burning_hands(w: &mut World, level: i32, ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    const DAM: [i32; 31] =
        [0, 0, 0, 0, 0, 19, 17, 20, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19, 19];
    let d = DAM[lv(level)];
    simple_attack(w, ch, victim, number(d >> 1, d << 1), SPELL_BURNING_HANDS);
}

pub fn spell_shocking_grasp(w: &mut World, level: i32, ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    const DAM: [i32; 31] =
        [0, 0, 0, 0, 0, 0, 0, 41, 33, 29, 27, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25, 25];
    let d = DAM[lv(level)];
    simple_attack(w, ch, victim, number(d >> 1, d << 1), SPELL_SHOCKING_GRASP);
}

pub fn spell_lightning_bolt(w: &mut World, level: i32, ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    const DAM: [i32; 31] =
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 59, 46, 39, 35, 38, 36, 36, 36, 36, 36, 36, 36, 36, 36, 36, 36, 36, 36, 36, 36, 36, 36];
    let d = DAM[lv(level)];
    simple_attack(w, ch, victim, number(d >> 1, d << 1), SPELL_LIGHTNING_BOLT);
}

pub fn spell_colour_spray(w: &mut World, level: i32, ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    const DAM: [i32; 31] =
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 79, 60, 57, 51, 47, 44, 44, 44, 44, 44, 44, 44, 44, 44, 44, 44, 44, 44, 44, 44];
    let d = DAM[lv(level)];
    simple_attack(w, ch, victim, number(d - 20, d + 20), SPELL_COLOUR_SPRAY);
}

pub fn spell_energy_drain(w: &mut World, level: i32, ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    let victim = need!(w, victim);
    if !saves_spell(w, victim, SAVING_SPELL) {
        w[ch].specials.alignment = (w[ch].specials.alignment - 200).min(-1000);
        if w.level(victim) <= 2 {
            damage(w, ch, victim, 100, SPELL_ENERGY_DRAIN);
        } else {
            let xp = number(level >> 1, level) * 1000;
            gain_exp(w, victim, -xp);
            let dam = dice(1, 10);
            let mana = w[victim].points.mana >> 1;
            w[victim].points.mov >>= 1;
            w[victim].points.mana = mana;
            w[ch].points.mana += mana >> 1;
            w[ch].points.hit += dam;
            send_to_char(w, "Your life energy is drained!\n\r", victim);
            damage(w, ch, victim, dam, SPELL_ENERGY_DRAIN);
        }
    } else {
        damage(w, ch, victim, 0, SPELL_ENERGY_DRAIN);
    }
}

pub fn spell_fireball(w: &mut World, level: i32, ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    const DAM: [i32; 31] =
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 143, 105, 88, 77, 71, 71, 71, 71, 71, 71, 71, 71, 71, 71, 71, 71];
    let d = DAM[lv(level)];
    simple_attack(w, ch, victim, number(d - 20, d + 20), SPELL_FIREBALL);
}

pub fn spell_earthquake(w: &mut World, level: i32, ch: CharId, _v: Option<CharId>, _o: Option<ObjId>) {
    let dam = dice(1, 8) + level;
    send_to_char(w, "The earth trembles beneath your feet!\n\r", ch);
    act(w, "$n makes the earth trembles and shivers\n\rYou fall, and hit yourself!", false, ch, None, Vict::None, TO_ROOM);
    for t in w.character_list.clone() {
        if !w.ch_exists(t) || !w.ch_exists(ch) || w[t].in_room < 0 {
            continue;
        }
        if w[ch].in_room == w[t].in_room && ch != t {
            damage(w, ch, t, dam, SPELL_EARTHQUAKE);
        } else if w.room(w[ch].in_room).zone == w.room(w[t].in_room).zone {
            send_to_char(w, "The earth trembles and shivers\n\r", t);
        }
    }
}

pub fn spell_dispel_evil(w: &mut World, level: i32, ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    let mut victim = need!(w, victim);
    if w.is_evil(ch) {
        victim = ch;
    } else if w.is_good(victim) {
        act(w, "God protects $N.", false, ch, None, Vict::Ch(victim), TO_CHAR);
        return;
    }
    let dam = if w.level(victim) < level || victim == ch {
        100
    } else {
        let mut d = dice(level, 4);
        if saves_spell(w, victim, SAVING_SPELL) {
            d >>= 1;
        }
        d
    };
    damage(w, ch, victim, dam, SPELL_DISPEL_EVIL);
}

pub fn spell_call_lightning(w: &mut World, level: i32, ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    let victim = need!(w, victim);
    let mut dam = dice(level.min(15), 8);
    if w.outside(ch) && w.weather_info.sky >= SKY_RAINING {
        if saves_spell(w, victim, SAVING_SPELL) {
            dam >>= 1;
        }
        damage(w, ch, victim, dam, SPELL_CALL_LIGHTNING);
    }
}

pub fn spell_harm(w: &mut World, _level: i32, ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    let victim = need!(w, victim);
    let mut dam = w[victim].points.hit - dice(1, 4);
    if dam < 0 {
        dam = 0;
    } else if saves_spell(w, victim, SAVING_SPELL) {
        dam = (dam / 2).min(50);
    }
    damage(w, ch, victim, dam, SPELL_HARM);
}

pub fn spell_armor(w: &mut World, _level: i32, _ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    let victim = need!(w, victim);
    if !affected_by_spell(w, victim, SPELL_ARMOR) {
        affect_to_char(w, victim, &af(SPELL_ARMOR, 24, -20, APPLY_AC, 0));
        send_to_char(w, "You feel someone protecting you.\n\r", victim);
    }
}

pub fn spell_teleport(w: &mut World, _level: i32, ch: CharId, _v: Option<CharId>, _o: Option<ObjId>) {
    let top = w.world.len() as i32 - 1;
    let mut to_room;
    loop {
        to_room = number(0, top);
        if w.room(to_room).room_flags & PRIVATE == 0 {
            break;
        }
    }
    act(w, "$n slowly fade out of existence.", false, ch, None, Vict::None, TO_ROOM);
    char_from_room(w, ch);
    char_to_room(w, ch, to_room);
    act(w, "$n slowly fade in to existence.", false, ch, None, Vict::None, TO_ROOM);
    do_look(w, ch, "", 0);
    if w.room(to_room).room_flags & DEATH != 0 && w.level(ch) < 21 {
        death_cry(w, ch);
        extract_char(w, ch);
    }
}

pub fn spell_bless(w: &mut World, _level: i32, ch: CharId, victim: Option<CharId>, obj: Option<ObjId>) {
    if let Some(obj) = obj {
        if 5 * w.level(ch) > w[obj].obj_flags.weight && w.get_pos(ch) != POSITION_FIGHTING && !w.is_obj_stat(obj, ITEM_EVIL)
        {
            w[obj].obj_flags.extra_flags |= ITEM_BLESS;
            act(w, "$p briefly glows.", false, ch, Some(obj), Vict::None, TO_CHAR);
        }
    } else {
        let victim = need!(w, victim);
        if w.get_pos(victim) != POSITION_FIGHTING && !affected_by_spell(w, victim, SPELL_BLESS) {
            send_to_char(w, "You feel righteous.\n\r", victim);
            affect_to_char(w, victim, &af(SPELL_BLESS, 6, 1, APPLY_HITROLL, 0));
            affect_to_char(w, victim, &af(SPELL_BLESS, 6, -1, APPLY_SAVING_SPELL, 0));
        }
    }
}

pub fn spell_blindness(w: &mut World, _level: i32, _ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    let victim = need!(w, victim);
    if saves_spell(w, victim, SAVING_SPELL) || affected_by_spell(w, victim, SPELL_BLINDNESS) {
        return;
    }
    act(w, "$n seems to be blinded!", true, victim, None, Vict::None, TO_ROOM);
    send_to_char(w, "You have been blinded!\n\r", victim);
    affect_to_char(w, victim, &af(SPELL_BLINDNESS, 1, -4, APPLY_HITROLL, AFF_BLIND));
    affect_to_char(w, victim, &af(SPELL_BLINDNESS, 1, 40, APPLY_AC, AFF_BLIND));
}

#[allow(dead_code)]
pub fn spell_clone(w: &mut World, _level: i32, ch: CharId, _v: Option<CharId>, _o: Option<ObjId>) {
    send_to_char(w, "Clone is not ready yet.", ch);
}

#[allow(dead_code)]
pub fn spell_control_weather(_w: &mut World, _level: i32, _ch: CharId, _v: Option<CharId>, _o: Option<ObjId>) {}

pub fn spell_create_food(w: &mut World, level: i32, ch: CharId, _v: Option<CharId>, _o: Option<ObjId>) {
    let mut o = Obj {
        item_number: -1,
        in_room: NOWHERE,
        name: "mushroom".into(),
        short_description: "A Magic Mushroom".into(),
        description: "A really delicious looking magic mushroom lies here.".into(),
        ..Default::default()
    };
    o.obj_flags.type_flag = ITEM_FOOD;
    o.obj_flags.wear_flags = ITEM_TAKE | ITEM_HOLD;
    o.obj_flags.value[0] = 5 + level;
    o.obj_flags.weight = 1;
    o.obj_flags.cost = 10;
    o.obj_flags.cost_per_day = 1;
    let id = w.objs.insert(o);
    w.object_list.insert(0, id);
    let r = w[ch].in_room;
    obj_to_room(w, id, r);
    act(w, "$p suddenly appears.", true, ch, Some(id), Vict::None, TO_ROOM);
    act(w, "$p suddenly appears.", true, ch, Some(id), Vict::None, TO_CHAR);
}

pub fn spell_create_water(w: &mut World, level: i32, ch: CharId, _v: Option<CharId>, obj: Option<ObjId>) {
    let Some(obj) = obj else { return };
    if w[obj].obj_flags.type_flag != ITEM_DRINKCON {
        return;
    }
    if w[obj].obj_flags.value[2] != LIQ_WATER && w[obj].obj_flags.value[1] != 0 {
        name_from_drinkcon(w, obj);
        w[obj].obj_flags.value[2] = LIQ_SLIME;
        name_to_drinkcon(w, obj, LIQ_SLIME);
    } else {
        let mut water = 2 * level * if w.weather_info.sky >= SKY_RAINING { 2 } else { 1 };
        water = (w[obj].obj_flags.value[0] - w[obj].obj_flags.value[1]).min(water);
        if water > 0 {
            w[obj].obj_flags.value[2] = LIQ_WATER;
            w[obj].obj_flags.value[1] += water;
            weight_change_object(w, obj, water);
            name_from_drinkcon(w, obj);
            name_to_drinkcon(w, obj, LIQ_WATER);
            act(w, "$p is filled.", false, ch, Some(obj), Vict::None, TO_CHAR);
        }
    }
}

pub fn spell_cure_blind(w: &mut World, _level: i32, _ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    let victim = need!(w, victim);
    if affected_by_spell(w, victim, SPELL_BLINDNESS) {
        affect_from_char(w, victim, SPELL_BLINDNESS);
        send_to_char(w, "Your vision returns!\n\r", victim);
    }
}

pub fn spell_cure_critic(w: &mut World, _level: i32, _ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    let victim = need!(w, victim);
    let hp = dice(3, 8) + 3;
    let lim = hit_limit(w, victim);
    if hp + w[victim].points.hit > lim {
        w[victim].points.hit = lim;
    } else {
        w[victim].points.hit += hp;
    }
    send_to_char(w, "You feel better!\n\r", victim);
    update_pos(w, victim);
}

pub fn spell_cure_light(w: &mut World, _level: i32, _ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    let victim = need!(w, victim);
    let hp = dice(1, 8);
    let lim = hit_limit(w, victim);
    if hp + w[victim].points.hit > lim {
        w[victim].points.hit = lim;
    } else {
        w[victim].points.hit += hp;
    }
    update_pos(w, victim);
    send_to_char(w, "You feel better!\n\r", victim);
}

pub fn spell_curse(w: &mut World, _level: i32, ch: CharId, victim: Option<CharId>, obj: Option<ObjId>) {
    if let Some(obj) = obj {
        w[obj].obj_flags.extra_flags |= ITEM_EVIL | ITEM_NODROP;
        if w[obj].obj_flags.type_flag == ITEM_WEAPON {
            w[obj].obj_flags.value[2] -= 1;
        }
        act(w, "$p glows red.", false, ch, Some(obj), Vict::None, TO_CHAR);
    } else {
        let victim = need!(w, victim);
        if saves_spell(w, victim, SAVING_SPELL) || affected_by_spell(w, victim, SPELL_CURSE) {
            return;
        }
        affect_to_char(w, victim, &af(SPELL_CURSE, 24 * 7, -1, APPLY_HITROLL, AFF_CURSE));
        affect_to_char(w, victim, &af(SPELL_CURSE, 24 * 7, 1, APPLY_SAVING_PARA, AFF_CURSE));
        act(w, "$n briefly reveal a red aura!", false, victim, None, Vict::None, TO_ROOM);
        act(w, "You feel very uncomfortable.", false, victim, None, Vict::None, TO_CHAR);
    }
}

fn detect(w: &mut World, level: i32, victim: Option<CharId>, spell: i32, bit: i64) {
    let victim = need!(w, victim);
    if affected_by_spell(w, victim, spell) {
        return;
    }
    affect_to_char(w, victim, &af(spell, level * 5, 0, APPLY_NONE, bit));
    send_to_char(w, "Your eyes tingle.\n\r", victim);
}

pub fn spell_detect_evil(w: &mut World, level: i32, _ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    detect(w, level, victim, SPELL_DETECT_EVIL, AFF_DETECT_EVIL);
}

pub fn spell_detect_invisibility(w: &mut World, level: i32, _ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    detect(w, level, victim, SPELL_DETECT_INVISIBLE, AFF_DETECT_INVISIBLE);
}

pub fn spell_detect_magic(w: &mut World, level: i32, _ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    detect(w, level, victim, SPELL_DETECT_MAGIC, AFF_DETECT_MAGIC);
}

pub fn spell_detect_poison(w: &mut World, _level: i32, ch: CharId, victim: Option<CharId>, obj: Option<ObjId>) {
    if let Some(victim) = victim {
        if victim == ch {
            if w.is_affected(victim, AFF_POISON) {
                send_to_char(w, "You can sense poison in your blood.\n\r", ch);
            } else {
                send_to_char(w, "You feel healthy.\n\r", ch);
            }
        } else if w.is_affected(victim, AFF_POISON) {
            act(w, "You sense that $E is poisoned.", false, ch, None, Vict::Ch(victim), TO_CHAR);
        } else {
            act(w, "You sense that $E is healthy.", false, ch, None, Vict::Ch(victim), TO_CHAR);
        }
    } else if let Some(obj) = obj {
        let t = w[obj].obj_flags.type_flag;
        if t == ITEM_DRINKCON || t == ITEM_FOOD {
            if w[obj].obj_flags.value[3] != 0 {
                act(w, "Poisonous fumes are revealed.", false, ch, None, Vict::None, TO_CHAR);
            } else {
                send_to_char(w, "It looks very delicious.\n\r", ch);
            }
        }
    }
}

pub fn spell_enchant_weapon(w: &mut World, level: i32, ch: CharId, _v: Option<CharId>, obj: Option<ObjId>) {
    let Some(obj) = obj else { return };
    if w[obj].obj_flags.type_flag == ITEM_WEAPON && !w.is_obj_stat(obj, ITEM_MAGIC) {
        if w[obj].affected.iter().any(|a| a.location != APPLY_NONE) {
            return;
        }
        w[obj].obj_flags.extra_flags |= ITEM_MAGIC;
        w[obj].affected[0] = ObjAffect { location: APPLY_HITROLL, modifier: 1 + (level >= 18) as i32 };
        w[obj].affected[1] = ObjAffect { location: APPLY_DAMROLL, modifier: 1 + (level >= 20) as i32 };
        if w.is_good(ch) {
            w[obj].obj_flags.extra_flags |= ITEM_ANTI_EVIL;
            act(w, "$p glows blue.", false, ch, Some(obj), Vict::None, TO_CHAR);
        } else if w.is_evil(ch) {
            w[obj].obj_flags.extra_flags |= ITEM_ANTI_GOOD;
            act(w, "$p glows red.", false, ch, Some(obj), Vict::None, TO_CHAR);
        } else {
            act(w, "$p glows yellow.", false, ch, Some(obj), Vict::None, TO_CHAR);
        }
    }
}

pub fn spell_heal(w: &mut World, level: i32, ch: CharId, victim: Option<CharId>, obj: Option<ObjId>) {
    let victim = need!(w, victim);
    spell_cure_blind(w, level, ch, Some(victim), obj);
    w[victim].points.hit += 100;
    let lim = hit_limit(w, victim);
    if w[victim].points.hit >= lim {
        w[victim].points.hit = lim - dice(1, 4);
    }
    update_pos(w, victim);
    send_to_char(w, "A warm feeling fills your body.\n\r", victim);
}

pub fn spell_invisibility(w: &mut World, _level: i32, ch: CharId, victim: Option<CharId>, obj: Option<ObjId>) {
    if let Some(obj) = obj {
        if !w.is_obj_stat(obj, ITEM_INVISIBLE) {
            act(w, "$p turns invisible.", false, ch, Some(obj), Vict::None, TO_CHAR);
            act(w, "$p turns invisible.", true, ch, Some(obj), Vict::None, TO_ROOM);
            w[obj].obj_flags.extra_flags |= ITEM_INVISIBLE;
        }
    } else {
        let victim = need!(w, victim);
        if !affected_by_spell(w, victim, SPELL_INVISIBLE) {
            act(w, "$n slowly fade out of existence.", true, victim, None, Vict::None, TO_ROOM);
            send_to_char(w, "You vanish.\n\r", victim);
            affect_to_char(w, victim, &af(SPELL_INVISIBLE, 24, -40, APPLY_AC, AFF_INVISIBLE));
        }
    }
}

pub fn spell_locate_object(w: &mut World, level: i32, ch: CharId, _v: Option<CharId>, obj: Option<ObjId>) {
    let Some(obj) = obj else { return };
    let name = fname(&w[obj].name);
    let mut j = level >> 1;
    for i in w.object_list.clone() {
        if j <= 0 {
            break;
        }
        if !isname(&name, &w[i].name) {
            continue;
        }
        let buf = if let Some(c) = w[i].carried_by {
            format!("{} carried by {}.\n\r", w[i].short_description, w.pers(c, ch))
        } else if let Some(io) = w[i].in_obj {
            format!("{} in {}.\n\r", w[i].short_description, w[io].short_description)
        } else {
            let where_ = if w[i].in_room == NOWHERE {
                "use, but uncertain.".to_string()
            } else {
                w.room(w[i].in_room).name.clone()
            };
            j -= 1;
            format!("{} in {}.\n\r", w[i].short_description, where_)
        };
        send_to_char(w, &buf, ch);
    }
    if j == 0 {
        send_to_char(w, "You are very confused.\n\r", ch);
    }
    if j == level >> 1 {
        send_to_char(w, "No such object.\n\r", ch);
    }
}

pub fn spell_poison(w: &mut World, level: i32, _ch: CharId, victim: Option<CharId>, obj: Option<ObjId>) {
    if let Some(victim) = victim {
        if !w.ch_exists(victim) {
            return;
        }
        if !saves_spell(w, victim, SAVING_PARA) {
            let mut a = af(SPELL_POISON, level * 2, -2, APPLY_STR, AFF_POISON);
            affect_join(w, victim, &mut a, false, false);
            send_to_char(w, "You feel very sick.\n\r", victim);
        }
    } else if let Some(obj) = obj {
        let t = w[obj].obj_flags.type_flag;
        if t == ITEM_DRINKCON || t == ITEM_FOOD {
            w[obj].obj_flags.value[3] = 1;
        }
    }
}

pub fn spell_protection_from_evil(w: &mut World, _level: i32, _ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    let victim = need!(w, victim);
    if !affected_by_spell(w, victim, SPELL_PROTECT_FROM_EVIL) {
        affect_to_char(w, victim, &af(SPELL_PROTECT_FROM_EVIL, 24, 0, APPLY_NONE, AFF_PROTECT_EVIL));
        send_to_char(w, "You have a righteous feeling!\n\r", victim);
    }
}

pub fn spell_remove_curse(w: &mut World, _level: i32, ch: CharId, victim: Option<CharId>, obj: Option<ObjId>) {
    if let Some(obj) = obj {
        if w.is_obj_stat(obj, ITEM_EVIL) || w.is_obj_stat(obj, ITEM_NODROP) {
            act(w, "$p briefly glows blue.", true, ch, Some(obj), Vict::None, TO_CHAR);
            w[obj].obj_flags.extra_flags &= !(ITEM_EVIL | ITEM_NODROP);
        }
    } else {
        let victim = need!(w, victim);
        if affected_by_spell(w, victim, SPELL_CURSE) {
            act(w, "$n briefly glows red, then blue.", false, victim, None, Vict::None, TO_ROOM);
            act(w, "You feel better.", false, victim, None, Vict::None, TO_CHAR);
            affect_from_char(w, victim, SPELL_CURSE);
        }
    }
}

pub fn spell_remove_poison(w: &mut World, _level: i32, ch: CharId, victim: Option<CharId>, obj: Option<ObjId>) {
    if let Some(victim) = victim {
        if !w.ch_exists(victim) {
            return;
        }
        if affected_by_spell(w, victim, SPELL_POISON) {
            affect_from_char(w, victim, SPELL_POISON);
            act(w, "A warm feeling runs through your body.", false, victim, None, Vict::None, TO_CHAR);
            act(w, "$N looks better.", false, ch, None, Vict::Ch(victim), TO_ROOM);
        }
    } else if let Some(obj) = obj {
        let t = w[obj].obj_flags.type_flag;
        if t == ITEM_DRINKCON || t == ITEM_FOOD {
            w[obj].obj_flags.value[3] = 0;
            act(w, "The $p steams briefly.", false, ch, Some(obj), Vict::None, TO_CHAR);
        }
    }
}

pub fn spell_sanctuary(w: &mut World, level: i32, _ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    let victim = need!(w, victim);
    if !affected_by_spell(w, victim, SPELL_SANCTUARY) {
        act(w, "$n is surrounded by a white aura.", true, victim, None, Vict::None, TO_ROOM);
        act(w, "You start glowing.", true, victim, None, Vict::None, TO_CHAR);
        let dur = if level < 21 { 3 } else { level };
        affect_to_char(w, victim, &af(SPELL_SANCTUARY, dur, 0, APPLY_NONE, AFF_SANCTUARY));
    }
}

pub fn spell_sleep(w: &mut World, level: i32, _ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    let victim = need!(w, victim);
    if !saves_spell(w, victim, SAVING_SPELL) {
        let mut a = af(SPELL_SLEEP, 4 + level, 0, APPLY_NONE, AFF_SLEEP);
        affect_join(w, victim, &mut a, false, false);
        if w.get_pos(victim) > POSITION_SLEEPING {
            act(w, "You feel very sleepy ..... zzzzzz", false, victim, None, Vict::None, TO_CHAR);
            act(w, "$n go to sleep.", true, victim, None, Vict::None, TO_ROOM);
            w[victim].specials.position = POSITION_SLEEPING;
        }
    }
}

pub fn spell_strength(w: &mut World, level: i32, _ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    let victim = need!(w, victim);
    act(w, "You feel stronger.", false, victim, None, Vict::None, TO_CHAR);
    let mut a = af(SPELL_STRENGTH, level, 1 + (level > 18) as i32, APPLY_STR, 0);
    affect_join(w, victim, &mut a, true, false);
}

#[allow(dead_code)]
pub fn spell_ventriloquate(_w: &mut World, _level: i32, _ch: CharId, _v: Option<CharId>, _o: Option<ObjId>) {}

pub fn spell_word_of_recall(w: &mut World, _level: i32, _ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    let victim = need!(w, victim);
    if w.is_npc(victim) {
        return;
    }
    let location = crate::db::real_room(w, 3001);
    if location < 0 {
        send_to_char(w, "You are completely lost.\n\r", victim);
        return;
    }
    act(w, "$n disappears.", true, victim, None, Vict::None, TO_ROOM);
    char_from_room(w, victim);
    char_to_room(w, victim, location);
    act(w, "$n appears in the middle of the room.", true, victim, None, Vict::None, TO_ROOM);
    do_look(w, victim, "", 15);
}

pub fn spell_summon(w: &mut World, level: i32, ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    let victim = need!(w, victim);
    if w.level(victim) > 20.min(level + 3) {
        send_to_char(w, "You failed.\n\r", ch);
        return;
    }
    if (w.is_npc(victim) && saves_spell(w, victim, SAVING_SPELL)) || w.room(w[victim].in_room).room_flags & PRIVATE != 0 {
        send_to_char(w, "You failed.\n\r", ch);
        return;
    }
    act(w, "$n disappears suddenly.", true, victim, None, Vict::None, TO_ROOM);
    let target = w[ch].in_room;
    char_from_room(w, victim);
    char_to_room(w, victim, target);
    act(w, "$n arrives suddenly.", true, victim, None, Vict::None, TO_ROOM);
    act(w, "$n has summoned you!", false, ch, None, Vict::Ch(victim), TO_VICT);
    do_look(w, victim, "", 15);
}

pub fn spell_charm_person(w: &mut World, level: i32, ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    let victim = need!(w, victim);
    if victim == ch {
        send_to_char(w, "You like yourself even better!\n\r", ch);
        return;
    }
    if !w.is_affected(victim, AFF_CHARM) && !w.is_affected(ch, AFF_CHARM) && level >= w.level(victim) {
        if circle_follow(w, victim, ch) {
            send_to_char(w, "Sorry, following in circles can not be allowed.\n\r", ch);
            return;
        }
        if saves_spell(w, victim, SAVING_PARA) {
            return;
        }
        if w[victim].master.is_some() {
            stop_follower(w, victim);
        }
        add_follower(w, victim, ch);
        let int = w[victim].tmpabilities.intel;
        let dur = if int != 0 { 24 * 18 / int } else { 24 * 18 };
        affect_to_char(w, victim, &af(SPELL_CHARM_PERSON, dur, 0, 0, AFF_CHARM));
        act(w, "Isn't $n just such a nice fellow?", false, ch, None, Vict::Ch(victim), TO_VICT);
    }
}

pub fn spell_sense_life(w: &mut World, level: i32, ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    let victim = need!(w, victim);
    if !affected_by_spell(w, victim, SPELL_SENSE_LIFE) {
        send_to_char(w, "Your feel your awareness improve.\n\r", ch);
        affect_to_char(w, victim, &af(SPELL_SENSE_LIFE, 5 * level, 0, APPLY_NONE, AFF_SENSE_LIFE));
    }
}

pub fn spell_identify(w: &mut World, _level: i32, ch: CharId, victim: Option<CharId>, obj: Option<ObjId>) {
    if let Some(obj) = obj {
        send_to_char(w, "You feel informed:\n\r", ch);
        let o = w[obj].clone();
        let buf = format!("Object '{}', Item type: {}\n\r", o.name, sprinttype(o.obj_flags.type_flag, ITEM_TYPES));
        send_to_char(w, &buf, ch);
        if o.obj_flags.bitvector != 0 {
            send_to_char(w, "Item will give you following abilities:  ", ch);
            let b = sprintbit(o.obj_flags.bitvector, AFFECTED_BITS) + "\n\r";
            send_to_char(w, &b, ch);
        }
        send_to_char(w, "Item is: ", ch);
        let b = sprintbit(o.obj_flags.extra_flags as i64, EXTRA_BITS) + "\n\r";
        send_to_char(w, &b, ch);
        let b = format!("Weight: {}, Value: {}\n\r", o.obj_flags.weight, o.obj_flags.cost);
        send_to_char(w, &b, ch);
        let v = o.obj_flags.value;
        match o.obj_flags.type_flag {
            ITEM_SCROLL | ITEM_POTION => {
                send_to_char(w, &format!("Level {} spells of:\n\r", v[0]), ch);
                for &s in &v[1..4] {
                    if s >= 1 {
                        let b = sprinttype(s - 1, SPELLS) + "\n\r";
                        send_to_char(w, &b, ch);
                    }
                }
            }
            ITEM_WAND | ITEM_STAFF => {
                send_to_char(w, &format!("Has {} chages, with {} charges left.\n\r", v[1], v[2]), ch);
                send_to_char(w, &format!("Level {} spell of:\n\r", v[0]), ch);
                if v[3] >= 1 {
                    let b = sprinttype(v[3] - 1, SPELLS) + "\n\r";
                    send_to_char(w, &b, ch);
                }
            }
            ITEM_WEAPON => send_to_char(w, &format!("Damage Dice is '{}D{}'\n\r", v[1], v[2]), ch),
            ITEM_ARMOR => send_to_char(w, &format!("AC-apply is {}\n\r", v[0]), ch),
            _ => {}
        }
        let mut found = false;
        for a in o.affected.iter() {
            if a.location != APPLY_NONE && a.modifier != 0 {
                if !found {
                    send_to_char(w, "Can affect you as :\n\r", ch);
                    found = true;
                }
                let b = format!("    Affects : {} By {}\n\r", sprinttype(a.location, APPLY_TYPES), a.modifier);
                send_to_char(w, &b, ch);
            }
        }
    } else if let Some(victim) = victim {
        if !w.is_npc(victim) {
            let a = age(&w[victim]);
            let b = format!("{} Years,  {} Months,  {} Days,  {} Hours old.\n\r", a.year, a.month, a.day, a.hours);
            send_to_char(w, &b, ch);
            let b = format!("Height {}cm  Weight {}pounds \n\r", w[victim].player.height, w[victim].player.weight);
            send_to_char(w, &b, ch);
        } else {
            send_to_char(w, "You learn nothing new.\n\r", ch);
        }
    }
}

fn breath_damage(w: &mut World, ch: CharId, victim: CharId, spell: i32) {
    let hpch = w[ch].points.hit.max(10);
    let mut dam = number(0, hpch >> 2);
    if saves_spell(w, victim, SAVING_BREATH) {
        dam >>= 1;
    }
    damage(w, ch, victim, dam, spell);
}

fn breath_destroys(w: &mut World, ch: CharId, victim: CharId, types: &[i32], msg: &str) {
    if !w.ch_exists(victim) || !w.ch_exists(ch) {
        return;
    }
    if number(0, 30) < w.level(ch) && !saves_spell(w, victim, SAVING_BREATH) {
        let found = w[victim]
            .carrying
            .clone()
            .into_iter()
            .find(|&b| types.contains(&w[b].obj_flags.type_flag) && number(0, 2) == 0);
        if let Some(b) = found {
            act(w, msg, false, victim, Some(b), Vict::None, TO_CHAR);
            extract_obj(w, b);
        }
    }
}

pub fn spell_fire_breath(w: &mut World, _level: i32, ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    let victim = need!(w, victim);
    breath_damage(w, ch, victim, SPELL_FIRE_BREATH);
    breath_destroys(w, ch, victim, &[ITEM_SCROLL, ITEM_WAND, ITEM_STAFF, ITEM_NOTE], "$o burns");
}

pub fn spell_frost_breath(w: &mut World, _level: i32, ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    let victim = need!(w, victim);
    breath_damage(w, ch, victim, SPELL_FROST_BREATH);
    breath_destroys(w, ch, victim, &[ITEM_DRINKCON, ITEM_FOOD, ITEM_POTION], "$o breaks.");
}

pub fn spell_acid_breath(w: &mut World, _level: i32, ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    let victim = need!(w, victim);
    breath_damage(w, ch, victim, SPELL_ACID_BREATH);
    if !w.ch_exists(victim) || !w.ch_exists(ch) {
        return;
    }
    if number(0, 30) < w.level(ch) && !saves_spell(w, victim, SAVING_BREATH) {
        let mut hit = None;
        for d in 0..MAX_WEAR {
            if let Some(o) = w[victim].equipment[d] {
                // (the C code tested != ITEM_ARMOR here; kept as it was)
                if w[o].obj_flags.type_flag != ITEM_ARMOR && w[o].obj_flags.value[0] > 0 && number(0, 2) == 0 {
                    hit = Some((d, o));
                    break;
                }
            }
        }
        if let Some((d, o)) = hit {
            act(w, "$o is damaged.", false, victim, Some(o), Vict::None, TO_CHAR);
            let a = apply_ac(w, victim, d);
            w[victim].points.armor -= a;
            w[o].obj_flags.value[0] -= number(1, 7);
            let a = apply_ac(w, victim, d);
            w[victim].points.armor += a;
            w[o].obj_flags.cost = 0;
        }
    }
}

pub fn spell_gas_breath(w: &mut World, _level: i32, ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    let victim = need!(w, victim);
    breath_damage(w, ch, victim, SPELL_GAS_BREATH);
}

pub fn spell_lightning_breath(w: &mut World, _level: i32, ch: CharId, victim: Option<CharId>, _o: Option<ObjId>) {
    let victim = need!(w, victim);
    breath_damage(w, ch, victim, SPELL_LIGHTNING_BREATH);
}
