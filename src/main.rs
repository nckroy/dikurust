//! DikuMUD I (Gamma 0.0), ported from C to Rust.
//!
//! Copyright (C) 1990, 1991 Sebastian Hammer, Michael Seifert,
//! Hans Henrik Staerfeldt, Tom Madsen and Katja Nyboe.
//! See doc/license.doc - the DikuMud license applies to this port too.
//!
//! Usage: dmserver [-s] [-d <dir>] [-b <bind address>] [port]

mod act_comm;
mod act_info;
mod act_move;
mod act_obj1;
mod act_obj2;
mod act_offensive;
mod act_other;
mod act_social;
mod act_wizard;
mod admin;
mod board;
mod changes;
mod comm;
mod constants;
mod crypto;
mod db;
mod fight;
mod game;
mod handler;
mod interpreter;
mod limits;
mod magic;
mod mar_fiz_maz;
mod mobact;
mod modify;
mod net;
mod reception;
mod save;
mod shop;
mod spec_assign;
mod spec_procs;
mod spell_parser;
mod spells;
mod structs;
mod utils;
mod weather;

use utils::log;

const DFLT_PORT: u16 = 4000;
const DFLT_DIR: &str = "lib";

fn usage(prog: &str) -> ! {
    eprintln!("Usage: {} [-s] [-d pathname] [-b bind-address] [ port # ]", prog);
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut port = DFLT_PORT;
    let mut dir = DFLT_DIR.to_string();
    let mut bind = "127.0.0.1".to_string();
    let mut no_specials = false;

    let mut pos = 1;
    while pos < args.len() && args[pos].starts_with('-') {
        let a = &args[pos];
        match a.chars().nth(1) {
            Some('l') => log("Lawful mode is not supported by this port; ignoring -l."),
            Some('d') => {
                if a.len() > 2 {
                    dir = a[2..].to_string();
                } else if pos + 1 < args.len() {
                    pos += 1;
                    dir = args[pos].clone();
                } else {
                    log("Directory arg expected after option -d.");
                    std::process::exit(1);
                }
            }
            Some('b') => {
                if a.len() > 2 {
                    bind = a[2..].to_string();
                } else if pos + 1 < args.len() {
                    pos += 1;
                    bind = args[pos].clone();
                } else {
                    log("Address arg expected after option -b.");
                    std::process::exit(1);
                }
            }
            Some('s') => {
                no_specials = true;
                log("Suppressing assignment of special routines.");
            }
            Some('h') => usage(&args[0]),
            _ => log(&format!("Unknown option {} in argument string.", a)),
        }
        pos += 1;
    }
    if pos < args.len() {
        match args[pos].parse::<u16>() {
            Ok(p) if p > 1024 => port = p,
            Ok(_) => {
                println!("Illegal port #");
                std::process::exit(1);
            }
            Err(_) => usage(&args[0]),
        }
    }

    admin::mark_boot();
    log(&format!("Running game on port {}.", port));
    if let Err(e) = std::env::set_current_dir(&dir) {
        log(&format!("chdir {}: {}", dir, e));
        std::process::exit(1);
    }
    log(&format!("Using {} as data directory.", dir));

    utils::seed_random(utils::now() as u64 ^ (std::process::id() as u64) << 32);

    log("Opening mother connection.");
    let addr = format!("{}:{}", bind, port);
    let mut srv = match net::Server::bind(&addr) {
        Ok(s) => s,
        Err(e) => {
            log(&format!("bind {}: {}", addr, e));
            std::process::exit(1);
        }
    };

    let mut w = game::World::new();
    w.no_specials = no_specials;
    db::boot_db(&mut w);

    if let Some(a) = srv.local_addr() {
        log(&format!("Entering game loop. Play at http://{}/", a));
    }
    net::game_loop(&mut w, &mut srv);

    if w.reboot {
        log("Rebooting.");
        std::process::exit(52);
    }
    log("Normal termination of game.");
}
