//! Attaching special procedures to mobiles, objects and rooms (spec_assign.c).

use crate::db::{real_mobile, real_object, real_room};
use crate::game::{SpecFn, World};
use crate::spec_procs::*;
use crate::utils::log;

fn mob(w: &mut World, vnum: i32, f: SpecFn, name: &'static str) {
    let r = real_mobile(w, vnum);
    if r >= 0 {
        w.mob_index[r as usize].func = Some(f);
        w.mob_index[r as usize].spec = name;
    } else {
        log(&format!("assign_mobiles: mobile {} does not exist", vnum));
    }
}

pub fn assign_mobiles(w: &mut World) {
    mob(w, 1, puff, "puff");
    mob(w, 3060, cityguard, "cityguard");
    mob(w, 3067, cityguard, "cityguard");
    mob(w, 3061, janitor, "janitor");
    mob(w, 3062, fido, "fido");
    mob(w, 3066, fido, "fido");
    mob(w, 3005, crate::reception::receptionist, "receptionist");
    mob(w, 5007, crate::reception::receptionist, "receptionist");
    mob(w, 3020, guild, "guild");
    mob(w, 3021, guild, "guild");
    mob(w, 3022, guild, "guild");
    mob(w, 3023, guild, "guild");
    mob(w, 3024, guild_guard, "guild_guard");
    mob(w, 3025, guild_guard, "guild_guard");
    mob(w, 3026, guild_guard, "guild_guard");
    mob(w, 3027, guild_guard, "guild_guard");
    mob(w, 3143, mayor, "mayor");
    // MORIA
    mob(w, 4000, snake, "snake");
    mob(w, 4001, snake, "snake");
    mob(w, 4053, snake, "snake");
    mob(w, 4103, thief, "thief");
    mob(w, 4100, magic_user, "magic_user");
    mob(w, 4102, snake, "snake");
    mob(w, 4150, bat_blue, "bat_blue");
    mob(w, 4151, bat_red, "bat_red");
    mob(w, 4152, bat_green, "bat_green");
    mob(w, 4153, bat_white, "bat_white");
    mob(w, 4154, bat_black, "bat_black");
    mob(w, 4155, thief, "thief");
    mob(w, 4157, magic_user, "magic_user");
    mob(w, 4158, snake, "snake");
    // SEWERS
    mob(w, 7006, snake, "snake");
    mob(w, 7040, bat_red, "bat_red");
    // FOREST
    mob(w, 6113, snake, "snake");
    mob(w, 6114, snake, "snake");
    mob(w, 6910, magic_user, "magic_user");

    crate::shop::boot_the_shops(w);
    crate::shop::assign_the_shopkeepers(w);
}

pub fn assign_objects(w: &mut World) {
    let r = real_object(w, 3099);
    if r >= 0 {
        w.obj_index[r as usize].func = Some(crate::board::board);
    }
}

pub fn assign_rooms(w: &mut World) {
    let rooms: [(i32, SpecFn); 5] = [
        (5039, worm_ritual),
        (3030, dump),
        (3054, pray_for_items),
        (3031, pet_shops),
        (8001, crate::mar_fiz_maz::mar_gate),
    ];
    for (vnum, f) in rooms {
        let r = real_room(w, vnum);
        if r >= 0 {
            w.world[r as usize].funct = Some(f);
        }
    }
}
