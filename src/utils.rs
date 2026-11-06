//! Utility routines (utility.c and the helper macros of utils.h).

#![allow(dead_code)]

use std::cell::Cell;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::structs::*;

thread_local! {
    static RNG: Cell<u64> = Cell::new(0x9E37_79B9_7F4A_7C15);
}

/// Seed the random generator (the C code called `srandom(time(0))`).
pub fn seed_random(seed: u64) {
    RNG.with(|r| r.set(seed | 1));
}

/// xorshift64* - plenty for dice rolls.
pub fn random() -> u32 {
    RNG.with(|r| {
        let mut x = r.get();
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        r.set(x);
        (x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 33) as u32
    })
}

/// Random number in the interval [from;to].
pub fn number(from: i32, to: i32) -> i32 {
    if to < from {
        // The C version computed `random() % (negative)`; be defined instead.
        return from;
    }
    let span = (to as i64 - from as i64 + 1) as u64;
    (from as i64 + (random() as u64 % span) as i64) as i32
}

/// Simulates a dice roll.
pub fn dice(num: i32, size: i32) -> i32 {
    let size = size.max(1);
    let mut sum = 0;
    for _ in 0..num.max(0) {
        sum += (random() % size as u32) as i32 + 1;
    }
    sum
}

pub fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// Case-insensitive compare: 0 if equal, -1 / 1 otherwise.
pub fn str_cmp(a: &str, b: &str) -> i32 {
    let a = a.as_bytes();
    let b = b.as_bytes();
    let mut i = 0;
    loop {
        let ca = a.get(i).copied().unwrap_or(0).to_ascii_lowercase();
        let cb = b.get(i).copied().unwrap_or(0).to_ascii_lowercase();
        if ca == 0 && cb == 0 {
            return 0;
        }
        if ca != cb {
            return if ca < cb { -1 } else { 1 };
        }
        i += 1;
    }
}

pub fn strn_cmp(a: &str, b: &str, n: usize) -> i32 {
    let a = a.as_bytes();
    let b = b.as_bytes();
    for i in 0..n {
        let ca = a.get(i).copied().unwrap_or(0).to_ascii_lowercase();
        let cb = b.get(i).copied().unwrap_or(0).to_ascii_lowercase();
        if ca == 0 && cb == 0 {
            return 0;
        }
        if ca != cb {
            return if ca < cb { -1 } else { 1 };
        }
    }
    0
}

/// Writes a line to the log (stderr).
pub fn log(s: &str) {
    eprintln!("{} :: {}", timestamp(), s);
}

fn timestamp() -> String {
    // Simple UTC timestamp, no external crates.
    let t = now();
    let (days, secs) = (t.div_euclid(86400), t.rem_euclid(86400));
    let (y, m, d) = civil_from_days(days);
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, m, d, secs / 3600, (secs / 60) % 60, secs % 60)
}

pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// `asctime()`-like rendering of a unix time, used by stat/users.
pub fn ctime(t: i64) -> String {
    const WD: [&str; 7] = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"];
    const MO: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let days = t.div_euclid(86400);
    let secs = t.rem_euclid(86400);
    let (y, m, d) = civil_from_days(days);
    format!(
        "{} {} {:2} {:02}:{:02}:{:02} {}",
        WD[days.rem_euclid(7) as usize],
        MO[(m - 1) as usize],
        d,
        secs / 3600,
        (secs / 60) % 60,
        secs % 60,
        y
    )
}

pub fn sprintbit(mut vektor: i64, names: &[&str]) -> String {
    let mut result = String::new();
    let mut nr = 0usize;
    while vektor != 0 {
        if vektor & 1 != 0 {
            if nr < names.len() {
                result.push_str(names[nr]);
                result.push(' ');
            } else {
                result.push_str("UNDEFINED ");
            }
        }
        if nr < names.len() {
            nr += 1;
        }
        vektor = ((vektor as u64) >> 1) as i64;
    }
    if result.is_empty() {
        result.push_str("NOBITS");
    }
    result
}

pub fn sprinttype(t: i32, names: &[&str]) -> String {
    if t >= 0 && (t as usize) < names.len() {
        names[t as usize].to_string()
    } else {
        "UNDEFINED".to_string()
    }
}

pub fn real_time_passed(t2: i64, t1: i64) -> TimeInfo {
    let mut secs = t2 - t1;
    let hours = (secs / SECS_PER_REAL_HOUR) % 24;
    secs -= SECS_PER_REAL_HOUR * hours;
    let day = secs / SECS_PER_REAL_DAY;
    TimeInfo { hours: hours as i32, day: day as i32, month: -1, year: -1 }
}

pub fn mud_time_passed(t2: i64, t1: i64) -> TimeInfo {
    let mut secs = t2 - t1;
    let hours = (secs / SECS_PER_MUD_HOUR) % 24;
    secs -= SECS_PER_MUD_HOUR * hours;
    let day = (secs / SECS_PER_MUD_DAY) % 35;
    secs -= SECS_PER_MUD_DAY * day;
    let month = (secs / SECS_PER_MUD_MONTH) % 17;
    secs -= SECS_PER_MUD_MONTH * month;
    let year = secs / SECS_PER_MUD_YEAR;
    TimeInfo { hours: hours as i32, day: day as i32, month: month as i32, year: year as i32 }
}

pub fn age(ch: &Char) -> TimeInfo {
    let mut a = mud_time_passed(now(), ch.player.time.birth);
    a.year += 17;
    a
}

/// CAP(): upper-case the first character.
pub fn cap(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_ascii_uppercase().to_string() + c.as_str(),
        None => String::new(),
    }
}

pub fn is_set<T: std::ops::BitAnd<Output = T> + PartialEq + Default>(flag: T, bit: T) -> bool {
    (flag & bit) != T::default()
}

pub fn get_req(i: i32) -> &'static str {
    if i < 2 {
        "Awful"
    } else if i < 4 {
        "Bad"
    } else if i < 7 {
        "Poor"
    } else if i < 10 {
        "Average"
    } else if i < 14 {
        "Fair"
    } else if i < 20 {
        "Good"
    } else if i < 24 {
        "Very good"
    } else {
        "Superb"
    }
}

/// ANA()/SANA(): article for an object name.
pub fn ana(name: &str) -> &'static str {
    match name.chars().next() {
        Some(c) if "aeiouyAEIOUY".contains(c) => "An",
        _ => "A",
    }
}

pub fn sana(name: &str) -> &'static str {
    match name.chars().next() {
        Some(c) if "aeiouyAEIOUY".contains(c) => "an",
        _ => "a",
    }
}

/// atoi() semantics: leading whitespace, optional sign, digits; 0 otherwise.
pub fn atoi(s: &str) -> i32 {
    let s = s.trim_start();
    let mut chars = s.chars().peekable();
    let mut neg = false;
    if let Some(&c) = chars.peek() {
        if c == '-' || c == '+' {
            neg = c == '-';
            chars.next();
        }
    }
    let mut v: i64 = 0;
    for c in chars {
        if let Some(d) = c.to_digit(10) {
            v = (v * 10 + d as i64).min(i32::MAX as i64 + 1);
        } else {
            break;
        }
    }
    let v = if neg { -v } else { v };
    v.clamp(i32::MIN as i64, i32::MAX as i64) as i32
}
