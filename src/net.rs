//! Networking and the main game loop (the socket half of comm.c).
//!
//! The original spoke raw telnet.  This port serves a small web client
//! over HTTP and talks to it over a WebSocket (RFC 6455), implemented here
//! with nothing but the standard library.  Everything above this layer -
//! descriptors, output queues, prompts, paging - works exactly as before.
//!
//! Wire protocol (server -> browser), one JSON object per text frame:
//!   {"t":"o","d":"<text>"}      game output (with the original "\n\r")
//!   {"t":"echo","v":false}       hide input (password prompt) / show again
//!   {"t":"resume","k":"<token>"} once playing: a one-use token for resuming
//!                                (null when the player leaves the game)
//!   {"t":"resume","ok":true}     answer to a resume attempt (false: log in)
//!   {"t":"bye"}                  the game closed the link (quit etc.); the
//!                                client should not try to resume
//! Browser -> server: each text frame is one or more input lines.
//!
//! Resuming: mobile browsers (iPadOS Safari especially) drop WebSockets
//! when a tab goes into the background.  The character stays in the game
//! link-dead, as with any lost link.  A client holding a resume token opens
//! `/ws?resume` and sends the token as its first frame; on success the new
//! connection takes over the character without asking for the password,
//! closing any old connection the server still thinks is alive.

use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::{Duration, Instant};

use crate::comm::*;
use crate::game::World;
use crate::handler::*;
use crate::structs::*;
use crate::utils::{log, str_cmp};

const OPT_USEC: u64 = 250_000; // 4 pulses per second, as in the original
const MAX_HTTP_HEADER: usize = 16 * 1024;
const MAX_WS_PAYLOAD: usize = 64 * 1024;
const MAX_CONNECTIONS: usize = 256;
/// A resume token is good for this long after it was issued.  The game's
/// own idle timers (the void, then extraction) usually end things sooner.
const RESUME_TTL: Duration = Duration::from_secs(24 * 3600);

pub static INDEX_HTML: &str = include_str!("../web/index.html");

#[derive(PartialEq)]
enum Kind {
    Http,
    Ws,
}

struct Conn {
    stream: TcpStream,
    peer: SocketAddr,
    kind: Kind,
    inbuf: Vec<u8>,
    outbuf: Vec<u8>,
    desc: Option<DescId>,
    closing: bool,
    dead: bool,
    frag: Vec<u8>,
    echo_sent: bool,
    /// opened as /ws?resume: the first frame is a resume token
    awaiting_resume: bool,
    /// the resume token this connection holds while its character plays
    token: Option<String>,
}

pub struct Server {
    listener: TcpListener,
    conns: Vec<Conn>,
}

impl Server {
    pub fn bind(addr: &str) -> std::io::Result<Server> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        Ok(Server { listener, conns: Vec::new() })
    }
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.listener.local_addr().ok()
    }
}

/* ------------------------------------------------------------------ */
/* game-side descriptor handling                                       */
/* ------------------------------------------------------------------ */

fn new_descriptor(w: &mut World, host: String, greet: bool) -> DescId {
    let id = w.next_desc_id;
    w.next_desc_id += 1;
    let d = Descriptor {
        id,
        host,
        pwd: String::new(),
        connected: CON_NME,
        wait: 1,
        showstr: None,
        str_target: None,
        str_buf: String::new(),
        max_str: 0,
        prompt_mode: false,
        last_input: String::new(),
        output: Vec::new(),
        input: std::collections::VecDeque::new(),
        character: None,
        original: None,
        snoop: Snoop::default(),
        echo: true,
    };
    w.descriptors.insert(0, d);
    if greet {
        greet_descriptor(w, id);
    }
    id
}

fn greet_descriptor(w: &mut World, id: DescId) {
    write_to_q(w, id, crate::db::GREETINGS);
    write_to_q(w, id, "By what name do you wish to be known? ");
}

/// Issue a fresh resume token for `name`, replacing any older one.
fn issue_resume_token(w: &mut World, name: &str) -> String {
    let now = Instant::now();
    w.resume_tokens
        .retain(|_, (n, at)| str_cmp(n, name) != 0 && now.duration_since(*at) < RESUME_TTL);
    let token = crate::crypto::random_token();
    w.resume_tokens.insert(token.clone(), (name.to_string(), now));
    token
}

/// Attach a fresh descriptor to the player a resume token names.  The token
/// is used up either way.  Returns false if it is unknown, expired, or the
/// character is no longer in the game; the caller then greets as usual.
fn try_resume(w: &mut World, did: DescId, token: &str) -> bool {
    let Some((name, at)) = w.resume_tokens.remove(token.trim()) else { return false };
    if at.elapsed() >= RESUME_TTL {
        return false;
    }
    let Some(ch) = w
        .character_list
        .iter()
        .copied()
        .find(|&t| !w.is_npc(t) && str_cmp(&name, &w[t].player.name) == 0)
    else {
        return false;
    };
    // save_char() writes the descriptor's password hash into the player
    // file, so the new descriptor needs it as if the player had typed it.
    let Some(pwd) = crate::save::load_char(w, &name).map(|st| st.pwd).filter(|p| !p.is_empty()) else {
        return false;
    };
    // The old connection may still look alive to us (a suspended tab
    // rarely says goodbye).  Drop it so the character is link-dead.
    let old: Vec<DescId> = w
        .descriptors
        .iter()
        .filter(|d| d.id != did && (d.character == Some(ch) || d.original == Some(ch)))
        .map(|d| d.id)
        .collect();
    for od in old {
        close_socket(w, od);
    }
    w[ch].desc = Some(did);
    w[ch].specials.timer = 0;
    let host = match w.desc_mut(did) {
        Some(d) => {
            d.character = Some(ch);
            d.connected = CON_PLYNG;
            d.pwd = pwd;
            d.host.clone()
        }
        None => return false,
    };
    write_to_q(w, did, "Reconnecting.\n\r");
    act(w, "$n has reconnected.", true, ch, None, Vict::None, TO_ROOM);
    log(&format!("{}[{}] has resumed.", w[ch].player.name, host));
    true
}

/// close_socket(): the game drops a connection.
pub fn close_socket(w: &mut World, did: DescId) {
    let Some(pos) = w.descriptors.iter().position(|d| d.id == did) else { return };
    let (snooping, snoop_by, character, connected, original) = {
        let d = &w.descriptors[pos];
        (d.snoop.snooping, d.snoop.snoop_by, d.character, d.connected, d.original)
    };
    if let Some(s) = snooping {
        if w.ch_exists(s) {
            if let Some(sd) = w[s].desc {
                if let Some(sd) = w.desc_mut(sd) {
                    sd.snoop.snoop_by = None;
                }
            }
        }
    }
    if let Some(s) = snoop_by {
        if w.ch_exists(s) {
            send_to_char(w, "Your victim is no longer among us.\n\r", s);
            if let Some(sd) = w[s].desc {
                if let Some(sd) = w.desc_mut(sd) {
                    sd.snoop.snooping = None;
                }
            }
        }
    }
    match character {
        Some(ch) if w.ch_exists(ch) => {
            if connected == CON_PLYNG {
                // a switched god: the body we are in keeps playing linkless,
                // the original body is the one to save.
                crate::save::save_char(w, ch, NOWHERE);
                act(w, "$n has lost $s link.", true, ch, None, Vict::None, TO_ROOM);
                log(&format!("Closing link to: {}.", w[ch].player.name));
                w[ch].desc = None;
                if let Some(o) = original {
                    if w.ch_exists(o) {
                        w[o].desc = None;
                    }
                }
            } else {
                log(&format!("Losing player: {}.", w[ch].player.name));
                if !w.character_list.contains(&ch) {
                    w.chars.remove(ch);
                } else {
                    w[ch].desc = None;
                }
            }
        }
        _ => log("Losing descriptor without char."),
    }
    let d = w.descriptors.remove(pos);
    w.closed.push((did, d.output));
}

/// Split one line of raw input the way process_input() did.
fn queue_input_line(w: &mut World, did: DescId, raw: &str) {
    let mut tmp = String::new();
    let mut too_long = false;
    for c in raw.chars() {
        if tmp.len() >= MAX_INPUT_LENGTH - 2 {
            too_long = true;
            break;
        }
        if c == '\u{8}' {
            if tmp.pop() == Some('$') {
                tmp.pop();
            }
        } else if c.is_ascii() && !c.is_ascii_control() {
            tmp.push(c);
            if c == '$' {
                tmp.push('$');
            }
        }
    }
    let Some(d) = w.desc_mut(did) else { return };
    if tmp.starts_with('!') {
        tmp = d.last_input.clone();
    } else {
        d.last_input = tmp.clone();
    }
    d.input.push_back(tmp.clone());
    let snoop_by = d.snoop.snoop_by;
    if too_long {
        d.output.push(format!("Line too long. Truncated to:\n\r{}\n\r", tmp));
    }
    if let Some(s) = snoop_by {
        if w.ch_exists(s) {
            if let Some(sd) = w[s].desc {
                write_to_q(w, sd, "% ");
                write_to_q(w, sd, &tmp);
                write_to_q(w, sd, "\n\r");
            }
        }
    }
}

/* ------------------------------------------------------------------ */
/* HTTP + WebSocket                                                    */
/* ------------------------------------------------------------------ */

fn json_escape(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o
}

fn ws_frame(opcode: u8, payload: &[u8]) -> Vec<u8> {
    let mut f = vec![0x80 | opcode];
    let n = payload.len();
    if n < 126 {
        f.push(n as u8);
    } else if n < 65536 {
        f.push(126);
        f.extend_from_slice(&(n as u16).to_be_bytes());
    } else {
        f.push(127);
        f.extend_from_slice(&(n as u64).to_be_bytes());
    }
    f.extend_from_slice(payload);
    f
}

fn ws_send_text(c: &mut Conn, s: &str) {
    let f = ws_frame(1, s.as_bytes());
    c.outbuf.extend_from_slice(&f);
}

fn http_response(c: &mut Conn, status: &str, ctype: &str, body: &str) {
    let r = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n{}",
        status,
        ctype,
        body.len(),
        body
    );
    c.outbuf.extend_from_slice(r.as_bytes());
    c.closing = true;
}

fn handle_http(w: &mut World, c: &mut Conn) {
    let Some(end) = c.inbuf.windows(4).position(|x| x == b"\r\n\r\n") else {
        if c.inbuf.len() > MAX_HTTP_HEADER {
            http_response(c, "431 Request Header Fields Too Large", "text/plain", "too large");
        }
        return;
    };
    let head = String::from_utf8_lossy(&c.inbuf[..end]).to_string();
    c.inbuf.drain(..end + 4);
    let mut lines = head.split("\r\n");
    let request = lines.next().unwrap_or("");
    let mut parts = request.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("/");
    let mut key = None;
    let mut upgrade = false;
    for l in lines {
        if let Some((k, v)) = l.split_once(':') {
            let k = k.trim().to_ascii_lowercase();
            let v = v.trim();
            if k == "sec-websocket-key" {
                key = Some(v.to_string());
            } else if k == "upgrade" && v.eq_ignore_ascii_case("websocket") {
                upgrade = true;
            }
        }
    }
    if method != "GET" {
        http_response(c, "405 Method Not Allowed", "text/plain", "method not allowed");
        return;
    }
    let (path, query) = path.split_once('?').unwrap_or((path, ""));
    let resume = query.split('&').any(|q| q == "resume");
    match (path, upgrade, key) {
        ("/ws", true, Some(key)) => {
            if w.wizlock {
                http_response(c, "503 Service Unavailable", "text/plain", "The game is wizlocked...");
                return;
            }
            let accept = crate::crypto::base64(&crate::crypto::sha1(
                format!("{}258EAFA5-E914-47DA-95CA-C5AB0DC85B11", key).as_bytes(),
            ));
            let r = format!(
                "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n",
                accept
            );
            c.outbuf.extend_from_slice(r.as_bytes());
            c.kind = Kind::Ws;
            let did = new_descriptor(w, c.peer.ip().to_string(), !resume);
            c.desc = Some(did);
            c.awaiting_resume = resume;
            log(&format!("New connection from {}.", c.peer));
        }
        ("/status", false, _) => {
            let body = crate::admin::status_json(w);
            http_response(c, "200 OK", "application/json", &body)
        }
        ("/" | "/index.html", _, _) => http_response(c, "200 OK", "text/html; charset=utf-8", INDEX_HTML),
        _ => http_response(c, "404 Not Found", "text/plain", "not found"),
    }
}

/// An out-of-band request from a client (see `admin.rs`).  Only immortals
/// who are logged in and playing get an answer.
fn handle_oob(w: &World, c: &mut Conn, req: &str) {
    let level = c
        .desc
        .and_then(|did| w.desc(did))
        .filter(|d| d.connected == CON_PLYNG)
        .and_then(|d| d.original.or(d.character))
        .filter(|&ch| w.ch_exists(ch) && !w.is_npc(ch))
        .map(|ch| w.level(ch));
    let reply = match (req.trim(), level) {
        ("snap", Some(l)) if l >= crate::admin::LEVEL_IMMORTAL => {
            format!("{{\"t\":\"wiz\",\"d\":{}}}", crate::admin::snapshot_json(w, l))
        }
        ("snap", _) => "{\"t\":\"wiz\",\"err\":\"Only immortals who are in the game can ask for this.\"}".to_string(),
        _ => "{\"t\":\"wiz\",\"err\":\"Unknown request.\"}".to_string(),
    };
    ws_send_text(c, &reply);
}

/// Parse complete WebSocket frames in `inbuf`.  Returns text messages.
fn parse_ws(c: &mut Conn) -> Vec<String> {
    let mut msgs = Vec::new();
    loop {
        let b = &c.inbuf;
        if b.len() < 2 {
            break;
        }
        let fin = b[0] & 0x80 != 0;
        let opcode = b[0] & 0x0f;
        let masked = b[1] & 0x80 != 0;
        let mut len = (b[1] & 0x7f) as usize;
        let mut pos = 2;
        if len == 126 {
            if b.len() < 4 {
                break;
            }
            len = u16::from_be_bytes([b[2], b[3]]) as usize;
            pos = 4;
        } else if len == 127 {
            if b.len() < 10 {
                break;
            }
            let l = u64::from_be_bytes([b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9]]);
            if l > MAX_WS_PAYLOAD as u64 {
                c.dead = true;
                break;
            }
            len = l as usize;
            pos = 10;
        }
        if len > MAX_WS_PAYLOAD || !masked {
            c.dead = true;
            break;
        }
        if b.len() < pos + 4 + len {
            break;
        }
        let mask = [b[pos], b[pos + 1], b[pos + 2], b[pos + 3]];
        pos += 4;
        let payload: Vec<u8> = b[pos..pos + len].iter().enumerate().map(|(i, x)| x ^ mask[i % 4]).collect();
        c.inbuf.drain(..pos + len);
        match opcode {
            0 | 1 | 2 => {
                c.frag.extend_from_slice(&payload);
                if c.frag.len() > MAX_WS_PAYLOAD {
                    c.dead = true;
                    break;
                }
                if fin {
                    let data = std::mem::take(&mut c.frag);
                    msgs.push(String::from_utf8_lossy(&data).to_string());
                }
            }
            8 => {
                let f = ws_frame(8, &[]);
                c.outbuf.extend_from_slice(&f);
                c.closing = true;
                break;
            }
            9 => {
                let f = ws_frame(10, &payload);
                c.outbuf.extend_from_slice(&f);
            }
            _ => {}
        }
    }
    msgs
}

fn read_conn(c: &mut Conn) {
    let mut buf = [0u8; 4096];
    loop {
        match c.stream.read(&mut buf) {
            Ok(0) => {
                c.dead = true;
                return;
            }
            Ok(n) => {
                c.inbuf.extend_from_slice(&buf[..n]);
                if c.inbuf.len() > MAX_HTTP_HEADER + MAX_WS_PAYLOAD * 2 {
                    c.dead = true;
                    return;
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => return,
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(_) => {
                c.dead = true;
                return;
            }
        }
    }
}

fn write_conn(c: &mut Conn) {
    while !c.outbuf.is_empty() {
        match c.stream.write(&c.outbuf) {
            Ok(0) => {
                c.dead = true;
                return;
            }
            Ok(n) => {
                c.outbuf.drain(..n);
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => return,
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(_) => {
                c.dead = true;
                return;
            }
        }
    }
    if c.closing {
        let _ = c.stream.shutdown(std::net::Shutdown::Both);
        c.dead = true;
    }
}

/* ------------------------------------------------------------------ */
/* the game loop                                                       */
/* ------------------------------------------------------------------ */

fn guarded<F: FnOnce(&mut World)>(w: &mut World, what: &str, f: F) {
    if catch_unwind(AssertUnwindSafe(|| f(w))).is_err() {
        log(&format!("SYSERR: recovered from a panic in {}", what));
    }
}

fn process_commands(w: &mut World) {
    let ids: Vec<DescId> = w.descriptors.iter().map(|d| d.id).collect();
    for did in ids {
        let Some(d) = w.desc_mut(did) else { continue };
        d.wait -= 1;
        if d.wait > 0 || d.input.is_empty() {
            continue;
        }
        let comm = d.input.pop_front().unwrap_or_default();
        let ch = d.character;
        let connected = d.connected;

        if let Some(ch) = ch {
            if w.ch_exists(ch) && connected == CON_PLYNG && w[ch].specials.was_in_room != NOWHERE {
                if w[ch].in_room != NOWHERE {
                    char_from_room(w, ch);
                }
                let r = w[ch].specials.was_in_room;
                char_to_room(w, ch, r);
                w[ch].specials.was_in_room = NOWHERE;
                act(w, "$n has returned.", true, ch, None, Vict::None, TO_ROOM);
                affect_total(w, ch);
            }
        }
        let Some(d) = w.desc_mut(did) else { continue };
        d.wait = 1;
        d.prompt_mode = true;
        let has_str = d.str_target.is_some();
        let has_show = d.showstr.is_some();
        let connected = d.connected;
        if let Some(ch) = ch {
            if w.ch_exists(ch) {
                w[ch].specials.timer = 0;
            }
        }
        guarded(w, "command", |w| {
            if has_str {
                crate::modify::string_add(w, did, &comm);
            } else if connected == CON_PLYNG {
                if has_show {
                    crate::modify::show_string(w, did, &comm);
                } else if let Some(ch) = ch {
                    if w.ch_exists(ch) {
                        crate::interpreter::command_interpreter(w, ch, &comm);
                    }
                }
            } else {
                crate::interpreter::nanny(w, did, &comm);
            }
        });
    }
}

/// Move queued game output into the connections' websocket buffers.
fn process_output(w: &mut World, srv: &mut Server) {
    // connections the game closed itself
    for (did, out) in std::mem::take(&mut w.closed) {
        if let Some(c) = srv.conns.iter_mut().find(|c| c.desc == Some(did)) {
            let text: String = out.concat();
            if !text.is_empty() {
                ws_send_text(c, &format!("{{\"t\":\"o\",\"d\":\"{}\"}}", json_escape(&text)));
            }
            ws_send_text(c, "{\"t\":\"bye\"}");
            let f = ws_frame(8, &[]);
            c.outbuf.extend_from_slice(&f);
            c.closing = true;
            c.desc = None;
        }
    }

    let ids: Vec<DescId> = w.descriptors.iter().map(|d| d.id).collect();
    for did in ids {
        let mut text = String::new();
        let (snoop_by, compact, echo, playing_as) = {
            let Some(d) = w.desc(did) else { continue };
            let playing_as = match d.character {
                Some(c) if d.connected == CON_PLYNG && w.ch_exists(c) => {
                    let who = d.original.filter(|&o| w.ch_exists(o)).unwrap_or(c);
                    Some(w[who].player.name.clone())
                }
                _ => None,
            };
            let compact = match d.character {
                Some(c) if w.ch_exists(c) => !w.is_npc(c) && w[c].specials.act & PLR_COMPACT != 0,
                _ => false,
            };
            (d.snoop.snoop_by, compact, d.echo, playing_as)
        };
        let d = w.desc_mut(did).unwrap();
        if !d.output.is_empty() {
            if !d.prompt_mode && d.connected == CON_PLYNG {
                text.push_str("\n\r");
            }
            let out = std::mem::take(&mut d.output);
            for piece in &out {
                text.push_str(piece);
            }
            if d.connected == CON_PLYNG && !compact {
                text.push_str("\n\r");
            }
            d.prompt_mode = true;
            if let Some(s) = snoop_by {
                if w.ch_exists(s) {
                    if let Some(sd) = w[s].desc {
                        for piece in &out {
                            write_to_q(w, sd, "% ");
                            write_to_q(w, sd, piece);
                        }
                    }
                }
            }
        }
        // prompts
        let d = w.desc_mut(did).unwrap();
        if d.prompt_mode {
            if d.str_target.is_some() {
                text.push_str("] ");
            } else if d.connected == CON_PLYNG {
                if d.showstr.is_some() {
                    text.push_str("*** Press return ***");
                } else {
                    text.push_str("> ");
                }
            }
            d.prompt_mode = false;
        }
        if let Some(c) = srv.conns.iter_mut().find(|c| c.desc == Some(did)) {
            if !text.is_empty() {
                ws_send_text(c, &format!("{{\"t\":\"o\",\"d\":\"{}\"}}", json_escape(&text)));
            }
            if c.echo_sent != echo {
                ws_send_text(c, &format!("{{\"t\":\"echo\",\"v\":{}}}", echo));
                c.echo_sent = echo;
            }
            match (playing_as, &c.token) {
                (Some(name), None) => {
                    let token = issue_resume_token(w, &name);
                    ws_send_text(c, &format!("{{\"t\":\"resume\",\"k\":\"{}\"}}", token));
                    c.token = Some(token);
                }
                (None, Some(token)) => {
                    // left the game (quit to the menu): nothing to resume
                    w.resume_tokens.remove(token);
                    ws_send_text(c, "{\"t\":\"resume\",\"k\":null}");
                    c.token = None;
                }
                _ => {}
            }
        }
    }
}

fn heartbeat(w: &mut World, pulse: i64) {
    if pulse % PULSE_ZONE == 0 {
        guarded(w, "zone_update", crate::db::zone_update);
    }
    if pulse % PULSE_MOBILE == 0 {
        guarded(w, "mobile_activity", crate::mobact::mobile_activity);
    }
    if pulse % PULSE_VIOLENCE == 0 {
        guarded(w, "perform_violence", crate::fight::perform_violence);
    }
    if pulse % (SECS_PER_MUD_HOUR * 4) == 0 {
        guarded(w, "weather", |w| crate::weather::weather_and_time(w, 1));
        guarded(w, "affect_update", crate::spell_parser::affect_update);
        guarded(w, "point_update", crate::limits::point_update);
    }
    if pulse >= 2400 {
        guarded(w, "check_reboot", crate::modify::check_reboot);
    }
}

pub fn game_loop(w: &mut World, srv: &mut Server) {
    let mut pulse: i64 = 0;
    let opt = Duration::from_micros(OPT_USEC);
    let mut next = Instant::now() + opt;

    while !w.shutdown {
        // new connections
        loop {
            match srv.listener.accept() {
                Ok((stream, peer)) => {
                    if srv.conns.len() >= MAX_CONNECTIONS {
                        let mut s = stream;
                        let _ = s.write_all(b"HTTP/1.1 503 Service Unavailable\r\nConnection: close\r\n\r\nSorry.. The game is full...\n");
                        continue;
                    }
                    let _ = stream.set_nonblocking(true);
                    let _ = stream.set_nodelay(true);
                    srv.conns.push(Conn {
                        stream,
                        peer,
                        kind: Kind::Http,
                        inbuf: Vec::new(),
                        outbuf: Vec::new(),
                        desc: None,
                        closing: false,
                        dead: false,
                        frag: Vec::new(),
                        echo_sent: true,
                        awaiting_resume: false,
                        token: None,
                    });
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) => {
                    log(&format!("Accept: {}", e));
                    break;
                }
            }
        }

        // input
        for c in srv.conns.iter_mut() {
            if c.dead || c.closing {
                continue;
            }
            read_conn(c);
            if c.kind == Kind::Http {
                handle_http(w, c);
            }
            if c.kind == Kind::Ws {
                for msg in parse_ws(c) {
                    if c.awaiting_resume {
                        c.awaiting_resume = false;
                        if let Some(did) = c.desc {
                            let ok = try_resume(w, did, &msg);
                            ws_send_text(c, &format!("{{\"t\":\"resume\",\"ok\":{}}}", ok));
                            if !ok {
                                greet_descriptor(w, did);
                            }
                        }
                        continue;
                    }
                    if msg.starts_with(crate::admin::OOB) {
                        handle_oob(w, c, &msg[1..]);
                        continue;
                    }
                    if let Some(did) = c.desc {
                        for line in msg.split('\n') {
                            let line = line.trim_end_matches('\r');
                            queue_input_line(w, did, line);
                        }
                    }
                }
            }
        }

        // drop dead connections from the game
        for c in srv.conns.iter_mut() {
            if c.dead {
                if let Some(did) = c.desc.take() {
                    close_socket(w, did);
                    w.closed.retain(|(id, _)| *id != did);
                }
            }
        }

        process_commands(w);
        process_output(w, srv);

        for c in srv.conns.iter_mut() {
            if !c.dead {
                write_conn(c);
            }
        }
        srv.conns.retain(|c| !c.dead);

        // heartbeat
        pulse += 1;
        heartbeat(w, pulse);
        if pulse >= 2400 {
            pulse = 0;
        }

        // sleep until the next pulse
        let nowi = Instant::now();
        if next > nowi {
            std::thread::sleep(next - nowi);
            next += opt;
        } else {
            next = nowi + opt;
        }
    }

    log("Closing all sockets.");
    let ids: Vec<DescId> = w.descriptors.iter().map(|d| d.id).collect();
    for did in ids {
        close_socket(w, did);
    }
    process_output(w, srv);
    for c in srv.conns.iter_mut() {
        let _ = c.stream.set_nonblocking(false);
        let _ = c.stream.set_write_timeout(Some(Duration::from_secs(2)));
        let _ = c.stream.write_all(&c.outbuf);
    }
}
