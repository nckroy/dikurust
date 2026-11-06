//! The bulletin board (board.c).
//!
//! Messages are kept in `lib/board.messages`, now as plain text:
//! a header line "#<header>" followed by the body, terminated by a line "~".

use std::fs;

use crate::comm::*;
use crate::game::World;
use crate::handler::isname;
use crate::interpreter::one_argument;
use crate::modify::page_string;
use crate::structs::*;
use crate::utils::*;

const MAX_MSGS: usize = 50;
const SAVE_FILE: &str = "board.messages";
const MAX_MESSAGE_LENGTH: usize = 4000;

#[derive(Default)]
pub struct Board {
    pub head: Vec<String>,
    pub msgs: Vec<String>,
}

pub fn board_load(w: &mut World) {
    w.board = Board::default();
    let Ok(text) = fs::read(SAVE_FILE) else {
        log("Board : Can't open message file. Board will be empty.");
        return;
    };
    let text = String::from_utf8_lossy(&text);
    let mut cur: Option<(String, String)> = None;
    for line in text.lines() {
        if let Some(h) = line.strip_prefix('#') {
            cur = Some((h.to_string(), String::new()));
        } else if line == "~" {
            if let Some((h, b)) = cur.take() {
                if w.board.head.len() < MAX_MSGS {
                    w.board.head.push(h);
                    w.board.msgs.push(b.replace("\\r", "\r"));
                }
            }
        } else if let Some((_, b)) = cur.as_mut() {
            b.push_str(line);
            b.push('\n');
        }
    }
}

pub fn board_save(w: &World) {
    let mut s = String::new();
    for (h, m) in w.board.head.iter().zip(&w.board.msgs) {
        s.push('#');
        s.push_str(&h.replace('\n', " "));
        s.push('\n');
        // keep the body line-oriented; "~" alone would end the message
        for l in m.replace('\r', "\\r").split('\n') {
            if l == "~" {
                s.push_str(" ~");
            } else {
                s.push_str(l);
            }
            s.push('\n');
        }
        s.push_str("~\n");
    }
    if let Err(e) = fs::write(SAVE_FILE, s) {
        log(&format!("Board : Unable to open/create savefile: {}", e));
    }
}

pub fn board(w: &mut World, ch: CharId, cmd: i32, arg: &str) -> bool {
    if w[ch].desc.is_none() {
        return false;
    }
    match cmd {
        15 => board_show_board(w, ch, arg),
        149 => {
            board_write_msg(w, ch, arg);
            true
        }
        63 => board_display_msg(w, ch, arg),
        66 => board_remove_msg(w, ch, arg),
        _ => false,
    }
}

fn board_write_msg(w: &mut World, ch: CharId, arg: &str) {
    if w.board.head.len() >= MAX_MSGS {
        send_to_char(w, "The board is full already.\n\r", ch);
        return;
    }
    let arg = arg.trim_start();
    if arg.is_empty() {
        send_to_char(w, "We must have a headline!\n\r", ch);
        return;
    }
    let head = format!("{} ({})", arg, w[ch].player.name);
    w.board.head.push(head);
    w.board.msgs.push(String::new());
    let idx = w.board.head.len() - 1;
    send_to_char(w, "Write your message. Terminate with a @.\n\r\n\r", ch);
    act(w, "$n starts to write a message.", true, ch, None, Vict::None, TO_ROOM);
    if let Some(d) = w[ch].desc {
        if let Some(d) = w.desc_mut(d) {
            d.str_target = Some(StrTarget::Board(idx));
            d.str_buf.clear();
            d.max_str = MAX_MESSAGE_LENGTH;
        }
    }
}

fn msg_number(arg: &str) -> Option<i32> {
    let (number, _) = one_argument(arg);
    if number.is_empty() || !number.as_bytes()[0].is_ascii_digit() {
        return None;
    }
    let n = atoi(&number);
    if n == 0 {
        None
    } else {
        Some(n)
    }
}

fn board_remove_msg(w: &mut World, ch: CharId, arg: &str) -> bool {
    if w.level(ch) < 10 {
        send_to_char(w, "Due to misuse of the REMOVE command, only 10th level\n\r", ch);
        send_to_char(w, "and above can remove messages.\n\r", ch);
        return true;
    }
    let Some(msg) = msg_number(arg) else { return false };
    let n = w.board.head.len() as i32;
    if n == 0 {
        send_to_char(w, "The board is empty!\n\r", ch);
        return true;
    }
    if msg < 1 || msg > n {
        send_to_char(w, "That message exists only in your imagination..\n\r", ch);
        return true;
    }
    let ind = (msg - 1) as usize;
    w.board.head.remove(ind);
    w.board.msgs.remove(ind);
    // anyone still writing a later message must follow the shift
    for d in w.descriptors.iter_mut() {
        if let Some(StrTarget::Board(i)) = d.str_target {
            if i == ind {
                d.str_target = None;
            } else if i > ind {
                d.str_target = Some(StrTarget::Board(i - 1));
            }
        }
    }
    send_to_char(w, "Message removed.\n\r", ch);
    let buf = format!("$n just removed message {}.", msg);
    act(w, &buf, false, ch, None, Vict::None, TO_ROOM);
    board_save(w);
    true
}

fn board_display_msg(w: &mut World, ch: CharId, arg: &str) -> bool {
    let Some(msg) = msg_number(arg) else { return false };
    let n = w.board.head.len() as i32;
    if n == 0 {
        send_to_char(w, "The board is empty!\n\r", ch);
        return true;
    }
    if msg < 1 || msg > n {
        send_to_char(w, "That message exists only in your imagination..\n\r", ch);
        return true;
    }
    let i = (msg - 1) as usize;
    let buffer = format!("Message {} : {}\n\r\n\r{}", msg, w.board.head[i], w.board.msgs[i]);
    if let Some(d) = w[ch].desc {
        page_string(w, d, &buffer);
    }
    true
}

fn board_show_board(w: &mut World, ch: CharId, arg: &str) -> bool {
    let (tmp, _) = one_argument(arg);
    if tmp.is_empty() || !isname(&tmp, "board bulletin") {
        return false;
    }
    act(w, "$n studies the board.", true, ch, None, Vict::None, TO_ROOM);
    let mut buf = String::from("This is a bulletin board. Usage: READ/REMOVE <messg #>, WRITE <header>\n\r");
    if w.board.head.is_empty() {
        buf.push_str("The board is empty.\n\r");
    } else {
        buf.push_str(&format!("There are {} messages on the board.\n\r", w.board.head.len()));
        for (i, h) in w.board.head.iter().enumerate() {
            buf.push_str(&format!("{:<2} : {}\n\r", i + 1, h));
        }
    }
    if let Some(d) = w[ch].desc {
        page_string(w, d, &buf);
    }
    true
}
