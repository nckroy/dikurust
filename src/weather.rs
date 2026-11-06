//! Time and weather (weather.c).

use crate::comm::send_to_outdoor;
use crate::game::World;
use crate::structs::*;
use crate::utils::dice;

pub fn weather_and_time(w: &mut World, mode: i32) {
    another_hour(w, mode);
    if mode != 0 {
        weather_change(w);
    }
}

pub fn another_hour(w: &mut World, mode: i32) {
    w.time_info.hours += 1;
    if mode != 0 {
        match w.time_info.hours {
            5 => {
                w.weather_info.sunlight = SUN_RISE;
                send_to_outdoor(w, "The sun rises in the east.\n\r");
            }
            6 => {
                w.weather_info.sunlight = SUN_LIGHT;
                send_to_outdoor(w, "The day has begun.\n\r");
            }
            21 => {
                w.weather_info.sunlight = SUN_SET;
                send_to_outdoor(w, "The sun slowly disappears in the west.\n\r");
            }
            22 => {
                w.weather_info.sunlight = SUN_DARK;
                send_to_outdoor(w, "The night has begun.\n\r");
            }
            _ => {}
        }
    }
    let t = &mut w.time_info;
    if t.hours > 23 {
        t.hours -= 24;
        t.day += 1;
        if t.day > 34 {
            t.day = 0;
            t.month += 1;
            if t.month > 16 {
                t.month = 0;
                t.year += 1;
            }
        }
    }
}

pub fn weather_change(w: &mut World) {
    let wi = &mut w.weather_info;
    let diff = if w.time_info.month >= 9 && w.time_info.month <= 16 {
        if wi.pressure > 985 { -2 } else { 2 }
    } else if wi.pressure > 1015 {
        -2
    } else {
        2
    };
    wi.change += dice(1, 4) * diff + dice(2, 6) - dice(2, 6);
    wi.change = wi.change.clamp(-12, 12);
    wi.pressure += wi.change;
    wi.pressure = wi.pressure.clamp(960, 1040);

    let p = wi.pressure;
    let change = match wi.sky {
        SKY_CLOUDLESS => {
            if p < 990 || (p < 1010 && dice(1, 4) == 1) {
                1
            } else {
                0
            }
        }
        SKY_CLOUDY => {
            if p < 970 {
                2
            } else if p < 990 {
                if dice(1, 4) == 1 { 2 } else { 0 }
            } else if p > 1030 && dice(1, 4) == 1 {
                3
            } else {
                0
            }
        }
        SKY_RAINING => {
            if p < 970 {
                if dice(1, 4) == 1 { 4 } else { 0 }
            } else if p > 1030 {
                5
            } else if p > 1010 && dice(1, 4) == 1 {
                5
            } else {
                0
            }
        }
        SKY_LIGHTNING => {
            if p > 1010 || (p > 990 && dice(1, 4) == 1) {
                6
            } else {
                0
            }
        }
        _ => {
            wi.sky = SKY_CLOUDLESS;
            0
        }
    };
    let (msg, sky) = match change {
        1 => ("The sky is getting cloudy.\n\r", SKY_CLOUDY),
        2 => ("It starts to rain.\n\r", SKY_RAINING),
        3 => ("The clouds disappear.\n\r", SKY_CLOUDLESS),
        4 => ("Lightning starts to show in the sky.\n\r", SKY_LIGHTNING),
        5 => ("The rain stopped.\n\r", SKY_CLOUDY),
        6 => ("The lightning has stopped.\n\r", SKY_RAINING),
        _ => return,
    };
    send_to_outdoor(w, msg);
    w.weather_info.sky = sky;
}
