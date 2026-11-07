//! Machine-readable views of the running game, for monitoring and for the
//! "Diku Wizard" iOS app (see `ios/`).
//!
//! Two views:
//!
//! * `GET /status` (public, no login): whether the game is up, uptime,
//!   how many people are on, wizlock, game time and weather.  It names no
//!   players and shows no addresses.
//!
//! * The wizard snapshot: a client that is logged in over `/ws` as an
//!   immortal (level 21+) sends a text frame starting with `\x01snap`. The
//!   server answers on the same socket with `{"t":"wiz","d":{...}}`: every
//!   connection and player, their rooms and states, the zones and their
//!   reset ages. Implementors (24) also get the tail of the server log.
//!   Anyone else gets `{"t":"wiz","err":"..."}`.  Normal input cannot start
//!   with a control character (the input path strips them), so this can't
//!   collide with a typed command.
//!
//! Nothing here changes the game: the app performs wizard actions by
//! sending ordinary commands, so the game's own level checks and logging
//! still apply.

use std::collections::VecDeque;
use std::fmt::Write as _;
use std::sync::{Mutex, OnceLock};

use crate::game::World;
use crate::structs::*;

/// Prefix of an out-of-band request frame on the WebSocket.
pub const OOB: char = '\u{1}';
/// Lowest level that may ask for the wizard snapshot.
pub const LEVEL_IMMORTAL: i32 = 21;
/// Lowest level that sees the server log in the snapshot.
pub const LEVEL_IMPL: i32 = 24;

const LOG_LINES: usize = 300;

static BOOT_TIME: OnceLock<i64> = OnceLock::new();
static LOG_TAIL: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());

pub fn mark_boot() {
    let _ = BOOT_TIME.set(crate::utils::now());
}

fn boot_time() -> i64 {
    *BOOT_TIME.get().unwrap_or(&0)
}

/// Keep a line of the server log for the snapshot.  Called by `utils::log`.
pub fn remember_log(line: String) {
    if let Ok(mut q) = LOG_TAIL.lock() {
        if q.len() >= LOG_LINES {
            q.pop_front();
        }
        q.push_back(line);
    }
}

/* ------------------------------------------------------------------ */
/* a tiny JSON writer                                                  */
/* ------------------------------------------------------------------ */

pub fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(o, "\\u{:04x}", c as u32);
            }
            c => o.push(c),
        }
    }
    o
}

struct Obj(String);

impl Obj {
    fn new() -> Obj {
        Obj(String::from("{"))
    }
    fn key(&mut self, k: &str) {
        if self.0.len() > 1 {
            self.0.push(',');
        }
        let _ = write!(self.0, "\"{}\":", k);
    }
    fn str(mut self, k: &str, v: &str) -> Obj {
        self.key(k);
        let _ = write!(self.0, "\"{}\"", esc(v));
        self
    }
    fn num(mut self, k: &str, v: i64) -> Obj {
        self.key(k);
        let _ = write!(self.0, "{}", v);
        self
    }
    fn bool(mut self, k: &str, v: bool) -> Obj {
        self.key(k);
        self.0.push_str(if v { "true" } else { "false" });
        self
    }
    fn raw(mut self, k: &str, v: &str) -> Obj {
        self.key(k);
        self.0.push_str(v);
        self
    }
    fn done(mut self) -> String {
        self.0.push('}');
        self.0
    }
}

fn array<I: IntoIterator<Item = String>>(items: I) -> String {
    let v: Vec<String> = items.into_iter().collect();
    format!("[{}]", v.join(","))
}

/* ------------------------------------------------------------------ */
/* names for numbers                                                   */
/* ------------------------------------------------------------------ */

fn class_name(c: i32) -> &'static str {
    match c {
        CLASS_MAGIC_USER => "Magic User",
        CLASS_CLERIC => "Cleric",
        CLASS_THIEF => "Thief",
        CLASS_WARRIOR => "Warrior",
        _ => "Unknown",
    }
}

fn position_name(p: i32) -> &'static str {
    match p {
        POSITION_DEAD => "dead",
        POSITION_MORTALLYW => "mortally wounded",
        POSITION_INCAP => "incapacitated",
        POSITION_STUNNED => "stunned",
        POSITION_SLEEPING => "sleeping",
        POSITION_RESTING => "resting",
        POSITION_SITTING => "sitting",
        POSITION_FIGHTING => "fighting",
        POSITION_STANDING => "standing",
        _ => "unknown",
    }
}

fn state_name(con: i32) -> &'static str {
    match con {
        CON_PLYNG => "playing",
        CON_NME | CON_NMECNF => "name",
        CON_PWDNRM | CON_PWDGET | CON_PWDCNF | CON_PWDNEW | CON_PWDNCNF => "password",
        CON_RMOTD => "motd",
        CON_SLCT => "menu",
        _ => "creating",
    }
}

fn sky_name(s: i32) -> &'static str {
    match s {
        SKY_CLOUDLESS => "cloudless",
        SKY_CLOUDY => "cloudy",
        SKY_RAINING => "raining",
        SKY_LIGHTNING => "lightning",
        _ => "unknown",
    }
}

fn sun_name(s: i32) -> &'static str {
    match s {
        SUN_DARK => "dark",
        SUN_RISE => "rising",
        SUN_LIGHT => "light",
        SUN_SET => "setting",
        _ => "unknown",
    }
}

/* ------------------------------------------------------------------ */
/* views                                                               */
/* ------------------------------------------------------------------ */

fn is_player(w: &World, c: CharId) -> bool {
    w.ch_exists(c) && !w.is_npc(c)
}

fn time_and_weather(w: &World) -> String {
    let t = &w.time_info;
    let month = crate::constants::MONTH_NAME.get(t.month as usize).copied().unwrap_or("?");
    let time = Obj::new()
        .num("hour", t.hours as i64)
        .num("day", t.day as i64 + 1)
        .num("month", t.month as i64)
        .str("month_name", month)
        .num("year", t.year as i64)
        .done();
    let weather = Obj::new()
        .str("sky", sky_name(w.weather_info.sky))
        .str("sun", sun_name(w.weather_info.sunlight))
        .num("pressure", w.weather_info.pressure as i64)
        .done();
    Obj::new().raw("time", &time).raw("weather", &weather).done()
}

/// The fields shared by `/status` and the snapshot's "server" object.
fn server_obj(w: &World) -> Obj {
    let now = crate::utils::now();
    let playing = w
        .descriptors
        .iter()
        .filter(|d| d.connected == CON_PLYNG)
        .count();
    let linkdead = w
        .character_list
        .iter()
        .filter(|&&c| is_player(w, c) && w[c].desc.is_none())
        .count();
    let tw = time_and_weather(w);
    Obj::new()
        .str("name", "DikuMUD")
        .str("version", env!("CARGO_PKG_VERSION"))
        .bool("up", true)
        .num("now", now)
        .num("boot", boot_time())
        .num("uptime", now - boot_time())
        .num("playing", playing as i64)
        .num("connections", w.descriptors.len() as i64)
        .num("linkdead", linkdead as i64)
        .bool("wizlock", w.wizlock)
        .raw("world", &tw)
}

/// Body of `GET /status`.
pub fn status_json(w: &World) -> String {
    server_obj(w).done()
}

fn zone_of_room(w: &World, vnum: i32) -> i64 {
    w.zone_table.iter().position(|z| vnum <= z.top).map(|i| i as i64).unwrap_or(-1)
}

/// Body of the `{"t":"wiz"}` reply for an immortal of level `level`.
pub fn snapshot_json(w: &World, level: i32) -> String {
    let mut seen = Vec::new();
    let mut players = Vec::new();

    // every connection, including people still at the login prompts
    for d in &w.descriptors {
        let who = d.original.or(d.character).filter(|&c| w.ch_exists(c));
        let mut o = Obj::new()
            .num("desc", d.id as i64)
            .str("host", &d.host)
            .str("state", state_name(d.connected))
            .bool("linkdead", false);
        if let Some(c) = who {
            seen.push(c);
            o = player_fields(w, o, c);
            if let (Some(orig), Some(body)) = (d.original, d.character) {
                if orig != body && w.ch_exists(body) {
                    o = o.str("switched_into", &w[body].player.short_descr);
                }
            }
            o = o.bool("snooping", d.snoop.snooping.is_some());
        }
        players.push(o.done());
    }
    // players still in the world with no connection
    for &c in &w.character_list {
        if is_player(w, c) && w[c].desc.is_none() && !seen.contains(&c) {
            let o = Obj::new().num("desc", -1).str("host", "").str("state", "linkdead").bool("linkdead", true);
            players.push(player_fields(w, o, c).done());
        }
    }

    let mut zone_players = vec![0i64; w.zone_table.len()];
    let mut zone_mobs = vec![0i64; w.zone_table.len()];
    for &c in &w.character_list {
        if !w.ch_exists(c) || w[c].in_room < 0 {
            continue;
        }
        let z = zone_of_room(w, w.room(w[c].in_room).number);
        if z >= 0 {
            if w.is_npc(c) {
                zone_mobs[z as usize] += 1;
            } else {
                zone_players[z as usize] += 1;
            }
        }
    }
    let mut bottom = 0;
    let zones = array(w.zone_table.iter().enumerate().map(|(i, z)| {
        let o = Obj::new()
            .num("n", i as i64)
            .str("name", &z.name)
            .num("bottom", bottom as i64)
            .num("top", z.top as i64)
            .num("age", z.age as i64)
            .num("lifespan", z.lifespan as i64)
            .num("reset_mode", z.reset_mode as i64)
            .num("players", zone_players[i])
            .num("mobs", zone_mobs[i])
            .done();
        bottom = z.top + 1;
        o
    }));

    let mobs = w.character_list.iter().filter(|&&c| w.ch_exists(c) && w.is_npc(c)).count();
    let counts = Obj::new()
        .num("rooms", w.world.len() as i64)
        .num("mobs", mobs as i64)
        .num("objects", w.object_list.len() as i64)
        .num("mob_protos", w.mob_index.len() as i64)
        .num("obj_protos", w.obj_index.len() as i64)
        .num("player_files", w.player_table.len() as i64)
        .num("in_combat", w.combat_list.len() as i64)
        .done();

    let mut o = Obj::new()
        .raw("server", &server_obj(w).done())
        .raw("counts", &counts)
        .raw("players", &format!("[{}]", players.join(",")))
        .raw("zones", &zones)
        .num("viewer_level", level as i64);
    if level >= LEVEL_IMPL {
        let lines: Vec<String> = LOG_TAIL
            .lock()
            .map(|q| q.iter().map(|l| format!("\"{}\"", esc(l))).collect())
            .unwrap_or_default();
        o = o.raw("log", &format!("[{}]", lines.join(",")));
    }
    o.done()
}

fn player_fields(w: &World, o: Obj, c: CharId) -> Obj {
    let ch = &w[c];
    let (vnum, room_name, zone) = if ch.in_room >= 0 {
        let r = w.room(ch.in_room);
        (r.number as i64, r.name.as_str(), zone_of_room(w, r.number))
    } else {
        (-1, "", -1)
    };
    let fl = ch.specials.act;
    let npc = w.is_npc(c);
    o.str("name", &ch.player.name)
        .str("title", &ch.player.title)
        .num("level", ch.player.level as i64)
        .str("class", class_name(ch.player.class))
        .num("room", vnum)
        .str("room_name", room_name)
        .num("zone", zone)
        .num("idle", ch.specials.timer as i64)
        .num("hit", ch.points.hit as i64)
        .num("max_hit", crate::limits::hit_limit(w, c) as i64)
        .num("mana", ch.points.mana as i64)
        .num("max_mana", crate::limits::mana_limit(w, c) as i64)
        .num("move", ch.points.mov as i64)
        .num("max_move", crate::limits::move_limit(w, c) as i64)
        .num("gold", ch.points.gold as i64)
        .num("exp", ch.points.exp as i64)
        .num("alignment", ch.specials.alignment as i64)
        .str("position", position_name(ch.specials.position))
        .bool("fighting", ch.specials.fighting.is_some())
        .bool("invisible", ch.specials.affected_by & AFF_INVISIBLE != 0)
        .bool("frozen", !npc && fl & PLR_FREEZE != 0)
        .bool("noshout", !npc && fl & PLR_NOSHOUT != 0)
        .bool("notell", !npc && fl & PLR_NOTELL != 0)
        .bool("noemote", !npc && fl & PLR_NOEMOTE != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_controls_and_quotes() {
        assert_eq!(esc("a\"b\\c\n\r\u{1b}"), "a\\\"b\\\\c\\n\\r\\u001b");
    }

    #[test]
    fn builds_objects() {
        let s = Obj::new().str("a", "x").num("b", -2).bool("c", true).raw("d", "[]").done();
        assert_eq!(s, r#"{"a":"x","b":-2,"c":true,"d":[]}"#);
    }

    #[test]
    fn log_tail_is_bounded() {
        for i in 0..LOG_LINES + 10 {
            remember_log(format!("line {}", i));
        }
        let q = LOG_TAIL.lock().unwrap();
        assert_eq!(q.len(), LOG_LINES);
        assert_eq!(q.back().unwrap(), &format!("line {}", LOG_LINES + 9));
    }
}
