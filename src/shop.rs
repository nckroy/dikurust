//! Shops and shopkeepers (shop.c).

use crate::act_comm::{do_say, do_tell};
use crate::act_social::do_action;
use crate::act_wizard::do_emote;
use crate::comm::*;
use crate::constants::DRINKS;
use crate::db::{read_object, real_mobile, real_object, real_room, Reader, REAL};
use crate::game::World;
use crate::handler::*;
use crate::interpreter::one_argument;
use crate::structs::*;
use crate::utils::*;

const SHOP_FILE: &str = "tinyworld.shp";
const MAX_TRADE: usize = 5;
const MAX_PROD: usize = 6;

#[derive(Clone, Debug, Default)]
pub struct Shop {
    pub producing: [i32; MAX_PROD],
    pub profit_buy: f32,
    pub profit_sell: f32,
    pub type_: [i32; MAX_TRADE],
    pub no_such_item1: String,
    pub no_such_item2: String,
    pub missing_cash1: String,
    pub missing_cash2: String,
    pub do_not_buy: String,
    pub message_buy: String,
    pub message_sell: String,
    pub temper1: i32,
    pub temper2: i32,
    pub keeper: i32,
    pub with_who: i32,
    pub in_room: i32,
    pub open1: i32,
    pub open2: i32,
    pub close1: i32,
    pub close2: i32,
}

/// The shop messages are printf formats with a `%s` (player name) and
/// sometimes a `%d` (price).
fn fmt_msg(f: &str, name: &str, amount: i32) -> String {
    let mut out = String::new();
    let mut it = f.chars().peekable();
    while let Some(c) = it.next() {
        if c == '%' {
            match it.next() {
                Some('s') => out.push_str(name),
                Some('d') => out.push_str(&amount.to_string()),
                Some('%') => out.push('%'),
                Some(o) => {
                    out.push('%');
                    out.push(o);
                }
                None => out.push('%'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn price(cost: i32, factor: f32) -> i32 {
    (cost as f32 * factor) as i32
}

fn is_ok(w: &mut World, keeper: CharId, ch: CharId, shop_nr: usize) -> bool {
    let s = &w.shops[shop_nr];
    let h = w.time_info.hours;
    if s.open1 > h {
        do_say(w, keeper, "Come back later!", 17);
        return false;
    } else if s.close1 < h {
        if s.open2 > h {
            do_say(w, keeper, "Sorry, we have closed, but come back later.", 17);
            return false;
        } else if s.close2 < h {
            do_say(w, keeper, "Sorry, come back tomorrow.", 17);
            return false;
        }
    }
    if !w.can_see(keeper, ch) {
        do_say(w, keeper, "I don't trade with someone I can't see!", 17);
        return false;
    }
    true
}

fn trade_with(w: &World, item: ObjId, shop_nr: usize) -> bool {
    if w[item].obj_flags.cost < 1 {
        return false;
    }
    w.shops[shop_nr].type_.contains(&w[item].obj_flags.type_flag)
}

fn shop_producing(w: &World, item: ObjId, shop_nr: usize) -> bool {
    let n = w[item].item_number;
    n >= 0 && w.shops[shop_nr].producing.contains(&n)
}

fn tell(w: &mut World, keeper: CharId, msg: &str) {
    do_tell(w, keeper, msg, 19);
}

fn shopping_buy(w: &mut World, arg: &str, ch: CharId, keeper: CharId, shop_nr: usize) {
    if !is_ok(w, keeper, ch, shop_nr) {
        return;
    }
    let name = w[ch].player.name.clone();
    let (argm, _) = one_argument(arg);
    if argm.is_empty() {
        tell(w, keeper, &format!("{} what do you want to buy??", name));
        return;
    }
    let Some(temp1) = get_obj_in_list_vis(w, ch, &argm, &w[keeper].carrying) else {
        let m = fmt_msg(&w.shops[shop_nr].no_such_item1, &name, 0);
        tell(w, keeper, &m);
        return;
    };
    if w[temp1].obj_flags.cost <= 0 {
        let m = fmt_msg(&w.shops[shop_nr].no_such_item1, &name, 0);
        tell(w, keeper, &m);
        extract_obj(w, temp1);
        return;
    }
    let cost = price(w[temp1].obj_flags.cost, w.shops[shop_nr].profit_buy);
    if w[ch].points.gold < cost && w.level(ch) < 22 {
        let m = fmt_msg(&w.shops[shop_nr].missing_cash2, &name, 0);
        tell(w, keeper, &m);
        match w.shops[shop_nr].temper1 {
            0 => do_action(w, keeper, &name, 30),
            1 => do_emote(w, keeper, "smokes on his joint", 36),
            _ => {}
        }
        return;
    }
    if w[ch].specials.carry_items + 1 > w.can_carry_n(ch) {
        let b = format!("{} : You can't carry that many items.\n\r", fname(&w[temp1].name));
        send_to_char(w, &b, ch);
        return;
    }
    if w[ch].specials.carry_weight + w[temp1].obj_flags.weight > w.can_carry_w(ch) {
        let b = format!("{} : You can't carry that much weight.\n\r", fname(&w[temp1].name));
        send_to_char(w, &b, ch);
        return;
    }
    act(w, "$n buys $p.", false, ch, Some(temp1), Vict::None, TO_ROOM);
    let m = fmt_msg(&w.shops[shop_nr].message_buy, &name, cost);
    tell(w, keeper, &m);
    let b = format!("You now have {}.\n\r", w[temp1].short_description);
    send_to_char(w, &b, ch);
    if w.level(ch) < 22 {
        w[ch].points.gold -= cost;
    }
    w[keeper].points.gold += cost;
    let item = if shop_producing(w, temp1, shop_nr) {
        let nr = w[temp1].item_number;
        match read_object(w, nr, REAL) {
            Some(o) => o,
            None => return,
        }
    } else {
        obj_from_char(w, temp1);
        temp1
    };
    obj_to_char(w, item, ch);
}

fn shopping_sell(w: &mut World, arg: &str, ch: CharId, keeper: CharId, shop_nr: usize) {
    if !is_ok(w, keeper, ch, shop_nr) {
        return;
    }
    let name = w[ch].player.name.clone();
    let (argm, _) = one_argument(arg);
    if argm.is_empty() {
        tell(w, keeper, &format!("{} What do you want to sell??", name));
        return;
    }
    let Some(temp1) = get_obj_in_list_vis(w, ch, &argm, &w[ch].carrying) else {
        let m = fmt_msg(&w.shops[shop_nr].no_such_item2, &name, 0);
        tell(w, keeper, &m);
        return;
    };
    if !trade_with(w, temp1, shop_nr) || w[temp1].obj_flags.cost < 1 {
        let m = fmt_msg(&w.shops[shop_nr].do_not_buy, &name, 0);
        tell(w, keeper, &m);
        return;
    }
    let cost = price(w[temp1].obj_flags.cost, w.shops[shop_nr].profit_sell);
    if w[keeper].points.gold < cost {
        let m = fmt_msg(&w.shops[shop_nr].missing_cash1, &name, 0);
        tell(w, keeper, &m);
        return;
    }
    act(w, "$n sells $p.", false, ch, Some(temp1), Vict::None, TO_ROOM);
    let m = fmt_msg(&w.shops[shop_nr].message_sell, &name, cost);
    tell(w, keeper, &m);
    let b = format!("The shopkeeper now has {}.\n\r", w[temp1].short_description);
    send_to_char(w, &b, ch);
    w[ch].points.gold += cost;
    w[keeper].points.gold -= cost;
    if get_obj_in_list(w, &argm, &w[keeper].carrying).is_some() || w[temp1].obj_flags.type_flag == ITEM_TRASH {
        extract_obj(w, temp1);
    } else {
        obj_from_char(w, temp1);
        obj_to_char(w, temp1, keeper);
    }
}

fn shopping_value(w: &mut World, arg: &str, ch: CharId, keeper: CharId, shop_nr: usize) {
    if !is_ok(w, keeper, ch, shop_nr) {
        return;
    }
    let name = w[ch].player.name.clone();
    let (argm, _) = one_argument(arg);
    if argm.is_empty() {
        tell(w, keeper, &format!("{} What do you want me to valuate??", name));
        return;
    }
    let Some(temp1) = get_obj_in_list_vis(w, ch, &argm, &w[ch].carrying) else {
        let m = fmt_msg(&w.shops[shop_nr].no_such_item2, &name, 0);
        tell(w, keeper, &m);
        return;
    };
    if !trade_with(w, temp1, shop_nr) {
        let m = fmt_msg(&w.shops[shop_nr].do_not_buy, &name, 0);
        tell(w, keeper, &m);
        return;
    }
    let cost = price(w[temp1].obj_flags.cost, w.shops[shop_nr].profit_sell);
    tell(w, keeper, &format!("{} I'll give you {} gold coins for that!", name, cost));
}

fn shopping_list(w: &mut World, _arg: &str, ch: CharId, keeper: CharId, shop_nr: usize) {
    if !is_ok(w, keeper, ch, shop_nr) {
        return;
    }
    let mut buf = String::from("You can buy:\n\r");
    let mut found = false;
    for t in w[keeper].carrying.clone() {
        if w.can_see_obj(ch, t) && w[t].obj_flags.cost > 0 {
            found = true;
            let o = &w[t];
            let cost = price(o.obj_flags.cost, w.shops[shop_nr].profit_buy);
            let what = if o.obj_flags.type_flag != ITEM_DRINKCON || o.obj_flags.value[1] == 0 {
                o.short_description.clone()
            } else {
                format!("{} of {}", o.short_description, DRINKS[o.obj_flags.value[2].clamp(0, 15) as usize])
            };
            buf.push_str(&cap(&format!("{} for {} gold coins.\n\r", what, cost)));
        }
    }
    if !found {
        buf.push_str("Nothing!\n\r");
    }
    send_to_char(w, &buf, ch);
}

pub fn shop_keeper(w: &mut World, ch: CharId, cmd: i32, arg: &str) -> bool {
    if !(56..=59).contains(&cmd) {
        return false;
    }
    let keeper = w
        .room(w[ch].in_room)
        .people
        .iter()
        .copied()
        .find(|&t| w.is_mob(t) && w.mob_index[w[t].nr as usize].spec == "shop_keeper");
    let Some(keeper) = keeper else { return false };
    let Some(shop_nr) = w.shops.iter().position(|s| s.keeper == w[keeper].nr) else { return false };
    if w[ch].in_room != real_room(w, w.shops[shop_nr].in_room) {
        return false;
    }
    match cmd {
        56 => shopping_buy(w, arg, ch, keeper, shop_nr),
        57 => shopping_sell(w, arg, ch, keeper, shop_nr),
        58 => shopping_value(w, arg, ch, keeper, shop_nr),
        59 => shopping_list(w, arg, ch, keeper, shop_nr),
        _ => return false,
    }
    true
}

pub fn boot_the_shops(w: &mut World) {
    let mut f = match Reader::open(SHOP_FILE) {
        Ok(f) => f,
        Err(e) => {
            log(&format!("Error in boot shop: {}", e));
            std::process::exit(1);
        }
    };
    loop {
        let buf = f.string();
        if buf.starts_with('$') || f.eof() {
            break;
        }
        if !buf.starts_with('#') {
            continue;
        }
        let mut s = Shop::default();
        for p in s.producing.iter_mut() {
            let t = f.int();
            *p = if t >= 0 { real_object(w, t) } else { t };
        }
        s.profit_buy = f.word().parse().unwrap_or(1.0);
        s.profit_sell = f.word().parse().unwrap_or(1.0);
        for t in s.type_.iter_mut() {
            *t = f.int();
        }
        f.line();
        s.no_such_item1 = f.string();
        s.no_such_item2 = f.string();
        s.do_not_buy = f.string();
        s.missing_cash1 = f.string();
        s.missing_cash2 = f.string();
        s.message_buy = f.string();
        s.message_sell = f.string();
        s.temper1 = f.int();
        s.temper2 = f.int();
        let k = f.int();
        s.keeper = real_mobile(w, k);
        s.with_who = f.int();
        s.in_room = f.int();
        s.open1 = f.int();
        s.close1 = f.int();
        s.open2 = f.int();
        s.close2 = f.int();
        f.line();
        w.shops.push(s);
    }
}

pub fn assign_the_shopkeepers(w: &mut World) {
    for i in 0..w.shops.len() {
        let k = w.shops[i].keeper;
        if k >= 0 {
            w.mob_index[k as usize].func = Some(shop_keeper);
            w.mob_index[k as usize].spec = "shop_keeper";
        }
    }
}
