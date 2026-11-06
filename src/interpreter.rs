//! Command interpreter and the login state machine (interpreter.c).

#![allow(dead_code)]

use crate::comm::*;
use crate::db::*;
use crate::game::World;
use crate::handler::*;
use crate::save::*;
use crate::structs::*;
use crate::utils::*;
use crate::{
    act_comm, act_info, act_move, act_obj1, act_obj2, act_offensive, act_other, act_social, act_wizard, changes,
    modify, spell_parser,
};

pub type CmdFn = fn(&mut World, CharId, &str, i32);

pub struct Cmd {
    pub name: &'static str,
    pub func: Option<CmdFn>,
    pub min_pos: i32,
    pub min_level: i32,
}

/// The command table.  Index == command number used all over the code
/// (special procedures and socials refer to commands by number).
pub static CMD_INFO: &[Cmd] = &[
    Cmd { name: "", func: None, min_pos: POSITION_DEAD, min_level: 0 }, // 0
    Cmd { name: "north", func: Some(act_move::do_move), min_pos: POSITION_STANDING, min_level: 0 }, // 1
    Cmd { name: "east", func: Some(act_move::do_move), min_pos: POSITION_STANDING, min_level: 0 }, // 2
    Cmd { name: "south", func: Some(act_move::do_move), min_pos: POSITION_STANDING, min_level: 0 }, // 3
    Cmd { name: "west", func: Some(act_move::do_move), min_pos: POSITION_STANDING, min_level: 0 }, // 4
    Cmd { name: "up", func: Some(act_move::do_move), min_pos: POSITION_STANDING, min_level: 0 }, // 5
    Cmd { name: "down", func: Some(act_move::do_move), min_pos: POSITION_STANDING, min_level: 0 }, // 6
    Cmd { name: "enter", func: Some(act_move::do_enter), min_pos: POSITION_STANDING, min_level: 0 }, // 7
    Cmd { name: "exits", func: Some(act_info::do_exits), min_pos: POSITION_RESTING, min_level: 0 }, // 8
    Cmd { name: "kiss", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 9
    Cmd { name: "get", func: Some(act_obj1::do_get), min_pos: POSITION_RESTING, min_level: 0 }, // 10
    Cmd { name: "drink", func: Some(act_obj2::do_drink), min_pos: POSITION_RESTING, min_level: 0 }, // 11
    Cmd { name: "eat", func: Some(act_obj2::do_eat), min_pos: POSITION_RESTING, min_level: 0 }, // 12
    Cmd { name: "wear", func: Some(act_obj2::do_wear), min_pos: POSITION_RESTING, min_level: 0 }, // 13
    Cmd { name: "wield", func: Some(act_obj2::do_wield), min_pos: POSITION_RESTING, min_level: 0 }, // 14
    Cmd { name: "look", func: Some(act_info::do_look), min_pos: POSITION_RESTING, min_level: 0 }, // 15
    Cmd { name: "score", func: Some(act_info::do_score), min_pos: POSITION_DEAD, min_level: 0 }, // 16
    Cmd { name: "say", func: Some(act_comm::do_say), min_pos: POSITION_RESTING, min_level: 0 }, // 17
    Cmd { name: "shout", func: Some(act_comm::do_shout), min_pos: POSITION_RESTING, min_level: 0 }, // 18
    Cmd { name: "tell", func: Some(act_comm::do_tell), min_pos: POSITION_DEAD, min_level: 0 }, // 19
    Cmd { name: "inventory", func: Some(act_info::do_inventory), min_pos: POSITION_DEAD, min_level: 0 }, // 20
    Cmd { name: "qui", func: Some(act_other::do_qui), min_pos: POSITION_DEAD, min_level: 0 }, // 21
    Cmd { name: "bounce", func: Some(act_social::do_action), min_pos: POSITION_STANDING, min_level: 0 }, // 22
    Cmd { name: "smile", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 23
    Cmd { name: "dance", func: Some(act_social::do_action), min_pos: POSITION_STANDING, min_level: 0 }, // 24
    Cmd { name: "kill", func: Some(act_offensive::do_kill), min_pos: POSITION_FIGHTING, min_level: 0 }, // 25
    Cmd { name: "cackle", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 26
    Cmd { name: "laugh", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 27
    Cmd { name: "giggle", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 28
    Cmd { name: "shake", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 29
    Cmd { name: "puke", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 30
    Cmd { name: "growl", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 31
    Cmd { name: "scream", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 32
    Cmd { name: "insult", func: Some(act_social::do_insult), min_pos: POSITION_RESTING, min_level: 0 }, // 33
    Cmd { name: "comfort", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 34
    Cmd { name: "nod", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 35
    Cmd { name: "sigh", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 36
    Cmd { name: "sulk", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 37
    Cmd { name: "help", func: Some(act_info::do_help), min_pos: POSITION_DEAD, min_level: 0 }, // 38
    Cmd { name: "who", func: Some(act_info::do_who), min_pos: POSITION_DEAD, min_level: 0 }, // 39
    Cmd { name: "emote", func: Some(act_wizard::do_emote), min_pos: POSITION_SLEEPING, min_level: 1 }, // 40
    Cmd { name: "echo", func: Some(act_wizard::do_echo), min_pos: POSITION_SLEEPING, min_level: 21 }, // 41
    Cmd { name: "stand", func: Some(act_move::do_stand), min_pos: POSITION_RESTING, min_level: 0 }, // 42
    Cmd { name: "sit", func: Some(act_move::do_sit), min_pos: POSITION_RESTING, min_level: 0 }, // 43
    Cmd { name: "rest", func: Some(act_move::do_rest), min_pos: POSITION_RESTING, min_level: 0 }, // 44
    Cmd { name: "sleep", func: Some(act_move::do_sleep), min_pos: POSITION_SLEEPING, min_level: 0 }, // 45
    Cmd { name: "wake", func: Some(act_move::do_wake), min_pos: POSITION_SLEEPING, min_level: 0 }, // 46
    Cmd { name: "force", func: Some(act_wizard::do_force), min_pos: POSITION_SLEEPING, min_level: 22 }, // 47
    Cmd { name: "transfer", func: Some(act_wizard::do_trans), min_pos: POSITION_SLEEPING, min_level: 22 }, // 48
    Cmd { name: "hug", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 49
    Cmd { name: "snuggle", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 50
    Cmd { name: "cuddle", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 51
    Cmd { name: "nuzzle", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 52
    Cmd { name: "cry", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 53
    Cmd { name: "news", func: Some(act_info::do_news), min_pos: POSITION_SLEEPING, min_level: 0 }, // 54
    Cmd { name: "equipment", func: Some(act_info::do_equipment), min_pos: POSITION_SLEEPING, min_level: 0 }, // 55
    Cmd { name: "buy", func: Some(act_other::do_not_here), min_pos: POSITION_STANDING, min_level: 0 }, // 56
    Cmd { name: "sell", func: Some(act_other::do_not_here), min_pos: POSITION_STANDING, min_level: 0 }, // 57
    Cmd { name: "value", func: Some(act_other::do_not_here), min_pos: POSITION_STANDING, min_level: 0 }, // 58
    Cmd { name: "list", func: Some(act_other::do_not_here), min_pos: POSITION_STANDING, min_level: 0 }, // 59
    Cmd { name: "drop", func: Some(act_obj1::do_drop), min_pos: POSITION_RESTING, min_level: 0 }, // 60
    Cmd { name: "goto", func: Some(act_wizard::do_goto), min_pos: POSITION_SLEEPING, min_level: 21 }, // 61
    Cmd { name: "weather", func: Some(act_info::do_weather), min_pos: POSITION_RESTING, min_level: 0 }, // 62
    Cmd { name: "read", func: Some(act_info::do_read), min_pos: POSITION_RESTING, min_level: 0 }, // 63
    Cmd { name: "pour", func: Some(act_obj2::do_pour), min_pos: POSITION_STANDING, min_level: 0 }, // 64
    Cmd { name: "grab", func: Some(act_obj2::do_grab), min_pos: POSITION_RESTING, min_level: 0 }, // 65
    Cmd { name: "remove", func: Some(act_obj2::do_remove), min_pos: POSITION_RESTING, min_level: 0 }, // 66
    Cmd { name: "put", func: Some(act_obj1::do_put), min_pos: POSITION_RESTING, min_level: 0 }, // 67
    Cmd { name: "shutdow", func: Some(act_wizard::do_shutdow), min_pos: POSITION_DEAD, min_level: 24 }, // 68
    Cmd { name: "save", func: Some(act_other::do_save), min_pos: POSITION_SLEEPING, min_level: 0 }, // 69
    Cmd { name: "hit", func: Some(act_offensive::do_hit), min_pos: POSITION_FIGHTING, min_level: 0 }, // 70
    Cmd { name: "string", func: Some(modify::do_string), min_pos: POSITION_SLEEPING, min_level: 23 }, // 71
    Cmd { name: "give", func: Some(act_obj1::do_give), min_pos: POSITION_RESTING, min_level: 0 }, // 72
    Cmd { name: "quit", func: Some(act_other::do_quit), min_pos: POSITION_DEAD, min_level: 0 }, // 73
    Cmd { name: "stat", func: Some(act_wizard::do_stat), min_pos: POSITION_DEAD, min_level: 21 }, // 74
    Cmd { name: "set", func: Some(changes::do_set), min_pos: POSITION_SLEEPING, min_level: 23 }, // 75
    Cmd { name: "time", func: Some(act_info::do_time), min_pos: POSITION_DEAD, min_level: 0 }, // 76
    Cmd { name: "load", func: Some(act_wizard::do_load), min_pos: POSITION_DEAD, min_level: 22 }, // 77
    Cmd { name: "purge", func: Some(act_wizard::do_purge), min_pos: POSITION_DEAD, min_level: 22 }, // 78
    Cmd { name: "shutdown", func: Some(act_wizard::do_shutdown), min_pos: POSITION_DEAD, min_level: 24 }, // 79
    Cmd { name: "idea", func: Some(act_other::do_idea), min_pos: POSITION_DEAD, min_level: 0 }, // 80
    Cmd { name: "typo", func: Some(act_other::do_typo), min_pos: POSITION_DEAD, min_level: 0 }, // 81
    Cmd { name: "bug", func: Some(act_other::do_bug), min_pos: POSITION_DEAD, min_level: 0 }, // 82
    Cmd { name: "whisper", func: Some(act_comm::do_whisper), min_pos: POSITION_RESTING, min_level: 0 }, // 83
    Cmd { name: "cast", func: Some(spell_parser::do_cast), min_pos: POSITION_SITTING, min_level: 1 }, // 84
    Cmd { name: "at", func: Some(act_wizard::do_at), min_pos: POSITION_DEAD, min_level: 21 }, // 85
    Cmd { name: "ask", func: Some(act_comm::do_ask), min_pos: POSITION_RESTING, min_level: 0 }, // 86
    Cmd { name: "order", func: Some(act_offensive::do_order), min_pos: POSITION_RESTING, min_level: 1 }, // 87
    Cmd { name: "sip", func: Some(act_obj2::do_sip), min_pos: POSITION_RESTING, min_level: 0 }, // 88
    Cmd { name: "taste", func: Some(act_obj2::do_taste), min_pos: POSITION_RESTING, min_level: 0 }, // 89
    Cmd { name: "snoop", func: Some(act_wizard::do_snoop), min_pos: POSITION_DEAD, min_level: 21 }, // 90
    Cmd { name: "follow", func: Some(act_move::do_follow), min_pos: POSITION_RESTING, min_level: 0 }, // 91
    Cmd { name: "rent", func: Some(act_other::do_not_here), min_pos: POSITION_STANDING, min_level: 1 }, // 92
    Cmd { name: "offer", func: Some(act_other::do_not_here), min_pos: POSITION_STANDING, min_level: 1 }, // 93
    Cmd { name: "poke", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 94
    Cmd { name: "advance", func: Some(act_wizard::do_advance), min_pos: POSITION_DEAD, min_level: 23 }, // 95
    Cmd { name: "accuse", func: Some(act_social::do_action), min_pos: POSITION_SITTING, min_level: 0 }, // 96
    Cmd { name: "grin", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 97
    Cmd { name: "bow", func: Some(act_social::do_action), min_pos: POSITION_STANDING, min_level: 0 }, // 98
    Cmd { name: "open", func: Some(act_move::do_open), min_pos: POSITION_SITTING, min_level: 0 }, // 99
    Cmd { name: "close", func: Some(act_move::do_close), min_pos: POSITION_SITTING, min_level: 0 }, // 100
    Cmd { name: "lock", func: Some(act_move::do_lock), min_pos: POSITION_SITTING, min_level: 0 }, // 101
    Cmd { name: "unlock", func: Some(act_move::do_unlock), min_pos: POSITION_SITTING, min_level: 0 }, // 102
    Cmd { name: "leave", func: Some(act_move::do_leave), min_pos: POSITION_STANDING, min_level: 0 }, // 103
    Cmd { name: "applaud", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 104
    Cmd { name: "blush", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 105
    Cmd { name: "burp", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 106
    Cmd { name: "chuckle", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 107
    Cmd { name: "clap", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 108
    Cmd { name: "cough", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 109
    Cmd { name: "curtsey", func: Some(act_social::do_action), min_pos: POSITION_STANDING, min_level: 0 }, // 110
    Cmd { name: "fart", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 111
    Cmd { name: "flip", func: Some(act_social::do_action), min_pos: POSITION_STANDING, min_level: 0 }, // 112
    Cmd { name: "fondle", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 113
    Cmd { name: "frown", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 114
    Cmd { name: "gasp", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 115
    Cmd { name: "glare", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 116
    Cmd { name: "groan", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 117
    Cmd { name: "grope", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 118
    Cmd { name: "hiccup", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 119
    Cmd { name: "lick", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 120
    Cmd { name: "love", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 121
    Cmd { name: "moan", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 122
    Cmd { name: "nibble", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 123
    Cmd { name: "pout", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 124
    Cmd { name: "purr", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 125
    Cmd { name: "ruffle", func: Some(act_social::do_action), min_pos: POSITION_STANDING, min_level: 0 }, // 126
    Cmd { name: "shiver", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 127
    Cmd { name: "shrug", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 128
    Cmd { name: "sing", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 129
    Cmd { name: "slap", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 130
    Cmd { name: "smirk", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 131
    Cmd { name: "snap", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 132
    Cmd { name: "sneeze", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 133
    Cmd { name: "snicker", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 134
    Cmd { name: "sniff", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 135
    Cmd { name: "snore", func: Some(act_social::do_action), min_pos: POSITION_SLEEPING, min_level: 0 }, // 136
    Cmd { name: "spit", func: Some(act_social::do_action), min_pos: POSITION_STANDING, min_level: 0 }, // 137
    Cmd { name: "squeeze", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 138
    Cmd { name: "stare", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 139
    Cmd { name: "strut", func: Some(act_social::do_action), min_pos: POSITION_STANDING, min_level: 0 }, // 140
    Cmd { name: "thank", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 141
    Cmd { name: "twiddle", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 142
    Cmd { name: "wave", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 143
    Cmd { name: "whistle", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 144
    Cmd { name: "wiggle", func: Some(act_social::do_action), min_pos: POSITION_STANDING, min_level: 0 }, // 145
    Cmd { name: "wink", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 146
    Cmd { name: "yawn", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 147
    Cmd { name: "snowball", func: Some(act_social::do_action), min_pos: POSITION_STANDING, min_level: 22 }, // 148
    Cmd { name: "write", func: Some(act_comm::do_write), min_pos: POSITION_STANDING, min_level: 1 }, // 149
    Cmd { name: "hold", func: Some(act_obj2::do_grab), min_pos: POSITION_RESTING, min_level: 1 }, // 150
    Cmd { name: "flee", func: Some(act_offensive::do_flee), min_pos: POSITION_FIGHTING, min_level: 1 }, // 151
    Cmd { name: "sneak", func: Some(act_other::do_sneak), min_pos: POSITION_STANDING, min_level: 1 }, // 152
    Cmd { name: "hide", func: Some(act_other::do_hide), min_pos: POSITION_RESTING, min_level: 1 }, // 153
    Cmd { name: "backstab", func: Some(act_offensive::do_backstab), min_pos: POSITION_STANDING, min_level: 1 }, // 154
    Cmd { name: "pick", func: Some(act_move::do_pick), min_pos: POSITION_STANDING, min_level: 1 }, // 155
    Cmd { name: "steal", func: Some(act_other::do_steal), min_pos: POSITION_STANDING, min_level: 1 }, // 156
    Cmd { name: "bash", func: Some(act_offensive::do_bash), min_pos: POSITION_FIGHTING, min_level: 1 }, // 157
    Cmd { name: "rescue", func: Some(act_offensive::do_rescue), min_pos: POSITION_FIGHTING, min_level: 1 }, // 158
    Cmd { name: "kick", func: Some(act_offensive::do_kick), min_pos: POSITION_FIGHTING, min_level: 1 }, // 159
    Cmd { name: "french", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 160
    Cmd { name: "comb", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 161
    Cmd { name: "massage", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 162
    Cmd { name: "tickle", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 163
    Cmd { name: "practice", func: Some(act_other::do_practice), min_pos: POSITION_RESTING, min_level: 1 }, // 164
    Cmd { name: "pat", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 165
    Cmd { name: "examine", func: Some(act_info::do_examine), min_pos: POSITION_SITTING, min_level: 0 }, // 166
    Cmd { name: "take", func: Some(act_obj1::do_get), min_pos: POSITION_RESTING, min_level: 0 }, // 167
    Cmd { name: "info", func: Some(act_info::do_info), min_pos: POSITION_SLEEPING, min_level: 0 }, // 168
    Cmd { name: "'", func: Some(act_comm::do_say), min_pos: POSITION_RESTING, min_level: 0 }, // 169
    Cmd { name: "practise", func: Some(act_other::do_practice), min_pos: POSITION_RESTING, min_level: 1 }, // 170
    Cmd { name: "curse", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 171
    Cmd { name: "use", func: Some(act_other::do_use), min_pos: POSITION_SITTING, min_level: 1 }, // 172
    Cmd { name: "where", func: Some(act_info::do_where), min_pos: POSITION_DEAD, min_level: 1 }, // 173
    Cmd { name: "levels", func: Some(act_info::do_levels), min_pos: POSITION_DEAD, min_level: 0 }, // 174
    Cmd { name: "reroll", func: Some(act_wizard::do_reroll), min_pos: POSITION_DEAD, min_level: 24 }, // 175
    Cmd { name: "pray", func: Some(act_social::do_action), min_pos: POSITION_SITTING, min_level: 0 }, // 176
    Cmd { name: ",", func: Some(act_wizard::do_emote), min_pos: POSITION_SLEEPING, min_level: 1 }, // 177
    Cmd { name: "beg", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 178
    Cmd { name: "bleed", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 179
    Cmd { name: "cringe", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 180
    Cmd { name: "daydream", func: Some(act_social::do_action), min_pos: POSITION_SLEEPING, min_level: 0 }, // 181
    Cmd { name: "fume", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 182
    Cmd { name: "grovel", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 183
    Cmd { name: "hop", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 184
    Cmd { name: "nudge", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 185
    Cmd { name: "peer", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 186
    Cmd { name: "point", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 187
    Cmd { name: "ponder", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 188
    Cmd { name: "punch", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 189
    Cmd { name: "snarl", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 190
    Cmd { name: "spank", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 191
    Cmd { name: "steam", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 192
    Cmd { name: "tackle", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 193
    Cmd { name: "taunt", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 194
    Cmd { name: "think", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 195
    Cmd { name: "whine", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 196
    Cmd { name: "worship", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 197
    Cmd { name: "yodel", func: Some(act_social::do_action), min_pos: POSITION_RESTING, min_level: 0 }, // 198
    Cmd { name: "brief", func: Some(act_other::do_brief), min_pos: POSITION_DEAD, min_level: 0 }, // 199
    Cmd { name: "wiz", func: Some(changes::do_wiz), min_pos: POSITION_DEAD, min_level: 21 }, // 200
    Cmd { name: "consider", func: Some(act_info::do_consider), min_pos: POSITION_RESTING, min_level: 0 }, // 201
    Cmd { name: "group", func: Some(act_other::do_group), min_pos: POSITION_RESTING, min_level: 1 }, // 202
    Cmd { name: "restore", func: Some(act_wizard::do_restore), min_pos: POSITION_DEAD, min_level: 22 }, // 203
    Cmd { name: "return", func: Some(act_wizard::do_return), min_pos: POSITION_DEAD, min_level: 0 }, // 204
    Cmd { name: "switch", func: Some(act_wizard::do_switch), min_pos: POSITION_DEAD, min_level: 23 }, // 205
    Cmd { name: "quaff", func: Some(act_other::do_quaff), min_pos: POSITION_RESTING, min_level: 0 }, // 206
    Cmd { name: "recite", func: Some(act_other::do_recite), min_pos: POSITION_RESTING, min_level: 0 }, // 207
    Cmd { name: "users", func: Some(act_info::do_users), min_pos: POSITION_DEAD, min_level: 21 }, // 208
    Cmd { name: "pose", func: Some(act_social::do_pose), min_pos: POSITION_STANDING, min_level: 0 }, // 209
    Cmd { name: "noshout", func: Some(act_wizard::do_noshout), min_pos: POSITION_SLEEPING, min_level: 22 }, // 210
    Cmd { name: "wizhelp", func: Some(act_info::do_wizhelp), min_pos: POSITION_SLEEPING, min_level: 21 }, // 211
    Cmd { name: "credits", func: Some(act_info::do_credits), min_pos: POSITION_DEAD, min_level: 0 }, // 212
    Cmd { name: "compact", func: Some(act_other::do_compact), min_pos: POSITION_DEAD, min_level: 0 }, // 213
    Cmd { name: "wizlock", func: Some(changes::do_wizlock), min_pos: POSITION_DEAD, min_level: 24 }, // 214
    Cmd { name: "notell", func: Some(changes::do_notell), min_pos: POSITION_DEAD, min_level: 22 }, // 215
    Cmd { name: "noemote", func: Some(changes::do_noemote), min_pos: POSITION_DEAD, min_level: 22 }, // 216
    Cmd { name: "freeze", func: Some(changes::do_freeze), min_pos: POSITION_DEAD, min_level: 23 }, // 217
    Cmd { name: "gol", func: Some(changes::do_log), min_pos: POSITION_DEAD, min_level: 24 }, // 218
    Cmd { name: "wizlist", func: Some(act_info::do_wizlist), min_pos: POSITION_DEAD, min_level: 0 }, // 219
    Cmd { name: ";", func: Some(changes::do_wiz), min_pos: POSITION_DEAD, min_level: 21 }, // 220
];

const FILL: [&str; 7] = ["in", "from", "with", "the", "on", "at", "to"];

/// search_block(): index of `arg` in `list` (exact or prefix match), or -1.
pub fn search_block(arg: &str, list: &[&str], exact: bool) -> i32 {
    let a = arg.to_ascii_lowercase();
    if exact {
        return list.iter().position(|&l| l == a).map(|p| p as i32).unwrap_or(-1);
    }
    // strncmp(arg, list[i], max(strlen(arg), 1)) == 0
    let n = a.len().max(1);
    let ab = a.as_bytes();
    for (i, l) in list.iter().enumerate() {
        let lb = l.as_bytes();
        let mut eq = true;
        for k in 0..n {
            let ca = ab.get(k).copied().unwrap_or(0);
            let cb = lb.get(k).copied().unwrap_or(0);
            if ca != cb {
                eq = false;
                break;
            }
            if ca == 0 {
                break;
            }
        }
        if eq {
            return i as i32;
        }
    }
    -1
}

/// Find a command number by (abbreviated) name; 0 for an empty word, -1 if none.
pub fn find_command(word: &str) -> i32 {
    if word.is_empty() {
        return 0;
    }
    for (i, c) in CMD_INFO.iter().enumerate().skip(1) {
        if c.name.starts_with(word) {
            return i as i32;
        }
    }
    -1
}

pub fn fill_word(arg: &str) -> bool {
    search_block(arg, &FILL, true) >= 0
}

pub fn is_number(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// Split off the first word, skipping fill words.  Returns (word, rest).
pub fn one_argument(argument: &str) -> (String, &str) {
    let mut rest = argument;
    loop {
        rest = rest.trim_start();
        let end = rest.find(|c: char| (c as u32) <= b' ' as u32).unwrap_or(rest.len());
        let word = rest[..end].to_ascii_lowercase();
        rest = &rest[end..];
        if !fill_word(&word) {
            return (word, rest);
        }
    }
}

/// argument_interpreter(): two arguments, skipping fill words.
pub fn argument_interpreter(argument: &str) -> (String, String) {
    let (a, rest) = one_argument(argument);
    let (b, _) = one_argument(rest);
    (a, b)
}

/// Is `arg1` an abbreviation of `arg2`?
pub fn is_abbrev(arg1: &str, arg2: &str) -> bool {
    if arg1.is_empty() {
        return false;
    }
    let a = arg1.as_bytes();
    let b = arg2.as_bytes();
    if a.len() > b.len() {
        return false;
    }
    a.iter().zip(b).all(|(x, y)| x.to_ascii_lowercase() == y.to_ascii_lowercase())
}

/// First word plus the rest of the string.
pub fn half_chop(s: &str) -> (String, String) {
    let s = s.trim_start();
    let end = s.find(|c: char| c.is_whitespace()).unwrap_or(s.len());
    (s[..end].to_string(), s[end..].trim_start().to_string())
}

pub fn command_interpreter(w: &mut World, ch: CharId, argument: &str) {
    w[ch].specials.affected_by &= !AFF_HIDE;

    let arg = argument.trim_start_matches(' ');
    let end = arg.find(|c: char| (c as u32) <= b' ' as u32).unwrap_or(arg.len());
    let word = arg[..end].to_ascii_lowercase();
    let rest = &arg[end..];

    let cmd = find_command(&word);
    if cmd == 0 {
        return;
    }
    if cmd > 0 && w.level(ch) < CMD_INFO[cmd as usize].min_level {
        send_to_char(w, "Arglebargle, glop-glyf!?!\n\r", ch);
        return;
    }
    if cmd > 0 {
        if let Some(f) = CMD_INFO[cmd as usize].func {
            if w.get_pos(ch) < CMD_INFO[cmd as usize].min_pos {
                let msg = match w.get_pos(ch) {
                    POSITION_DEAD => "Lie still; you are DEAD!!! :-( \n\r",
                    POSITION_INCAP | POSITION_MORTALLYW => "You are in a pretty bad shape, unable to do anything!\n\r",
                    POSITION_STUNNED => "All you can do right now, is think about the stars!\n\r",
                    POSITION_SLEEPING => "In your dreams, or what?\n\r",
                    POSITION_RESTING => "Nah... You feel too relaxed to do that..\n\r",
                    POSITION_SITTING => "Maybe you should get on your feet first?\n\r",
                    POSITION_FIGHTING => "No way! You are fighting for your life!\n\r",
                    _ => "",
                };
                send_to_char(w, msg, ch);
            } else {
                if w[ch].specials.act & PLR_FREEZE != 0 && !w.is_npc(ch) {
                    send_to_char(w, "You're totally frozen!\n\r", ch);
                    return;
                }
                if !w.no_specials && special(w, ch, cmd, rest) {
                    return;
                }
                f(w, ch, rest, cmd);
            }
            return;
        }
        send_to_char(w, "Sorry, but that command has yet to be implemented...\n\r", ch);
    } else {
        send_to_char(w, "Arglebargle, glop-glyf!?!\n\r", ch);
    }
}

/// Give rooms, carried/worn objects, mobs and objects present a chance
/// to intercept a command.
pub fn special(w: &mut World, ch: CharId, cmd: i32, arg: &str) -> bool {
    let room = w[ch].in_room;
    if room < 0 {
        return false;
    }
    if let Some(f) = w.room(room).funct {
        if f(w, ch, cmd, arg) {
            return true;
        }
    }
    if !w.ch_exists(ch) {
        return true;
    }
    for j in 0..MAX_WEAR {
        if let Some(o) = w[ch].equipment[j] {
            let nr = w[o].item_number;
            if nr >= 0 {
                if let Some(f) = w.obj_index[nr as usize].func {
                    if f(w, ch, cmd, arg) {
                        return true;
                    }
                }
            }
        }
        if !w.ch_exists(ch) {
            return true;
        }
    }
    for o in w[ch].carrying.clone() {
        if !w.obj_exists(o) {
            continue;
        }
        let nr = w[o].item_number;
        if nr >= 0 {
            if let Some(f) = w.obj_index[nr as usize].func {
                if f(w, ch, cmd, arg) {
                    return true;
                }
            }
        }
        if !w.ch_exists(ch) {
            return true;
        }
    }
    for k in w.room(room).people.clone() {
        if w.ch_exists(k) && w.is_mob(k) {
            if let Some(f) = w.mob_index[w[k].nr as usize].func {
                if f(w, ch, cmd, arg) {
                    return true;
                }
            }
        }
        if !w.ch_exists(ch) {
            return true;
        }
    }
    for o in w.room(room).contents.clone() {
        if !w.obj_exists(o) {
            continue;
        }
        let nr = w[o].item_number;
        if nr >= 0 {
            if let Some(f) = w.obj_index[nr as usize].func {
                if f(w, ch, cmd, arg) {
                    return true;
                }
            }
        }
        if !w.ch_exists(ch) {
            return true;
        }
    }
    false
}

/* ------------------------------------------------------------------ */
/* the login sequence                                                  */
/* ------------------------------------------------------------------ */

fn send_q(w: &mut World, d: DescId, s: &str) {
    write_to_q(w, d, s);
}

fn set_echo(w: &mut World, d: DescId, on: bool) {
    if let Some(d) = w.desc_mut(d) {
        d.echo = on;
    }
}

fn set_state(w: &mut World, d: DescId, st: i32) {
    if let Some(d) = w.desc_mut(d) {
        d.connected = st;
    }
}

fn state(w: &World, d: DescId) -> i32 {
    w.desc(d).map(|d| d.connected).unwrap_or(-1)
}

fn parse_name(arg: &str) -> Option<String> {
    let a = arg.trim_start();
    if a.is_empty() || a.len() > 15 || !a.bytes().all(|b| b.is_ascii_alphabetic()) {
        return None;
    }
    Some(a.to_string())
}

const MAX_PWD_LENGTH: usize = 64;

/// Deal with newcomers and other non-playing sockets.
pub fn nanny(w: &mut World, d: DescId, arg: &str) {
    let arg = arg.trim_start();
    let Some(ch) = w.desc(d).and_then(|x| x.character) else {
        // CON_NME without a character yet
        let c = Char { desc: Some(d), ..Default::default() };
        let id = w.chars.insert(c);
        if let Some(dd) = w.desc_mut(d) {
            dd.character = Some(id);
        }
        return nanny(w, d, arg);
    };

    match state(w, d) {
        CON_NME => {
            if arg.is_empty() {
                crate::net::close_socket(w, d);
                return;
            }
            let Some(tmp_name) = parse_name(arg) else {
                send_q(w, d, "Illegal name, please try another.");
                send_q(w, d, "Name: ");
                return;
            };
            // already playing?
            let mut playing = false;
            for k in &w.descriptors {
                if let Some(kc) = k.character {
                    if kc != ch {
                        let who = k.original.unwrap_or(kc);
                        if w.ch_exists(who) && str_cmp(&w[who].player.name, &tmp_name) == 0 {
                            playing = true;
                        }
                    }
                }
            }
            if playing {
                send_q(w, d, "Already playing, cannot connect\n\r");
                send_q(w, d, "Name: ");
                return;
            }
            if let Some(st) = load_char(w, &tmp_name) {
                store_to_char(w, &st, ch);
                if let Some(dd) = w.desc_mut(d) {
                    dd.pwd = st.pwd.clone();
                }
                send_q(w, d, "Password: ");
                set_echo(w, d, false);
                set_state(w, d, CON_PWDNRM);
            } else {
                let name = cap(&tmp_name.to_ascii_lowercase());
                w[ch].player.name = name.clone();
                send_q(w, d, &format!("Did I get that right, {} (Y/N)? ", name));
                set_state(w, d, CON_NMECNF);
            }
        }
        CON_NMECNF => match arg.chars().next() {
            Some('y') | Some('Y') => {
                send_q(w, d, "New character.\n\r");
                let buf = format!("Give me a password for {}: ", w[ch].player.name);
                send_q(w, d, &buf);
                set_echo(w, d, false);
                set_state(w, d, CON_PWDGET);
            }
            Some('n') | Some('N') => {
                send_q(w, d, "Ok, what IS it, then? ");
                w[ch].player.name.clear();
                set_state(w, d, CON_NME);
            }
            _ => send_q(w, d, "Please type Yes or No? "),
        },
        CON_PWDNRM => {
            if arg.is_empty() {
                crate::net::close_socket(w, d);
                return;
            }
            let stored = w.desc(d).map(|x| x.pwd.clone()).unwrap_or_default();
            if !crate::crypto::verify_password(arg.trim_end(), &stored) {
                send_q(w, d, "Wrong password.\n\r");
                send_q(w, d, "Password: ");
                return;
            }
            set_echo(w, d, true);
            let name = w[ch].player.name.clone();
            let found = w
                .character_list
                .iter()
                .copied()
                .find(|&t| str_cmp(&name, &w[t].player.name) == 0 && w[t].desc.is_none() && !w.is_npc(t));
            if let Some(tmp_ch) = found {
                send_q(w, d, "Reconnecting.\n\r");
                w.chars.remove(ch);
                w[tmp_ch].desc = Some(d);
                w[tmp_ch].specials.timer = 0;
                if let Some(dd) = w.desc_mut(d) {
                    dd.character = Some(tmp_ch);
                    dd.connected = CON_PLYNG;
                }
                act(w, "$n has reconnected.", true, tmp_ch, None, Vict::None, TO_ROOM);
                let host = w.desc(d).map(|x| x.host.clone()).unwrap_or_default();
                log(&format!("{}[{}] has reconnected.", name, host));
                return;
            }
            let host = w.desc(d).map(|x| x.host.clone()).unwrap_or_default();
            log(&format!("{}[{}] has connected.", name, host));
            let motd = w.motd.clone();
            send_q(w, d, &motd);
            send_q(w, d, "\n\r\n*** PRESS RETURN: ");
            set_state(w, d, CON_RMOTD);
        }
        CON_PWDGET => {
            let pw = arg.trim_end();
            if pw.is_empty() || pw.len() > MAX_PWD_LENGTH {
                send_q(w, d, "Illegal password.\n\r");
                send_q(w, d, "Password: ");
                return;
            }
            if let Some(dd) = w.desc_mut(d) {
                dd.pwd = crate::crypto::hash_password(pw);
            }
            send_q(w, d, "Please retype password: ");
            set_state(w, d, CON_PWDCNF);
        }
        CON_PWDCNF => {
            let stored = w.desc(d).map(|x| x.pwd.clone()).unwrap_or_default();
            if !crate::crypto::verify_password(arg.trim_end(), &stored) {
                send_q(w, d, "Passwords don't match.\n\r");
                send_q(w, d, "Retype password: ");
                set_state(w, d, CON_PWDGET);
                return;
            }
            set_echo(w, d, true);
            send_q(w, d, "What is your sex (M/F) ? ");
            set_state(w, d, CON_QSEX);
        }
        CON_QSEX => {
            match arg.chars().next() {
                Some('m') | Some('M') => w[ch].player.sex = SEX_MALE,
                Some('f') | Some('F') => w[ch].player.sex = SEX_FEMALE,
                _ => {
                    send_q(w, d, "That's not a sex..\n\r");
                    send_q(w, d, "What IS your sex? :");
                    return;
                }
            }
            send_q(w, d, "\n\rSelect a class:\n\rCleric\n\rThief\n\rWarrior\n\rMagic-user");
            send_q(w, d, "\n\rClass :");
            set_state(w, d, CON_QCLASS);
        }
        CON_QCLASS => {
            let class = match arg.chars().next() {
                Some('m') | Some('M') => CLASS_MAGIC_USER,
                Some('c') | Some('C') => CLASS_CLERIC,
                Some('w') | Some('W') => CLASS_WARRIOR,
                Some('t') | Some('T') => CLASS_THIEF,
                _ => {
                    send_q(w, d, "\n\rThat's not a class.\n\rClass:");
                    return;
                }
            };
            w[ch].player.class = class;
            init_char(w, ch);
            let name = w[ch].player.name.clone();
            create_entry(w, &name);
            save_char(w, ch, NOWHERE);
            let motd = w.motd.clone();
            send_q(w, d, &motd);
            send_q(w, d, "\n\r\n*** PRESS RETURN: ");
            set_state(w, d, CON_RMOTD);
            let host = w.desc(d).map(|x| x.host.clone()).unwrap_or_default();
            log(&format!("{} [{}] new player.", name, host));
        }
        CON_RMOTD => {
            send_q(w, d, MENU);
            set_state(w, d, CON_SLCT);
        }
        CON_SLCT => match arg.chars().next() {
            Some('0') => crate::net::close_socket(w, d),
            Some('1') => {
                reset_char(w, ch);
                if w[ch].in_room != NOWHERE {
                    log("Loading chars equipment and transferring to room.");
                    crate::reception::load_char_objs(w, ch);
                    save_char(w, ch, NOWHERE);
                }
                send_to_char(w, WELC_MESSG, ch);
                w.character_list.insert(0, ch);
                let start = real_room(w, 3001);
                let target = if w[ch].in_room == NOWHERE {
                    start
                } else {
                    let r = real_room(w, w[ch].in_room);
                    if r > -1 {
                        r
                    } else {
                        start
                    }
                };
                w[ch].in_room = NOWHERE;
                char_to_room(w, ch, target);
                act(w, "$n has entered the game.", true, ch, None, Vict::None, TO_ROOM);
                set_state(w, d, CON_PLYNG);
                if w.level(ch) == 0 {
                    act_wizard::do_start(w, ch);
                }
                act_info::do_look(w, ch, "", 15);
                if let Some(dd) = w.desc_mut(d) {
                    dd.prompt_mode = true;
                }
            }
            Some('2') => {
                send_q(w, d, "Enter a text you'd like others to see when they look at you.\n\r");
                send_q(w, d, "Terminate with a '@'.\n\r");
                let old = std::mem::take(&mut w[ch].player.description);
                if !old.is_empty() {
                    send_q(w, d, "Old description :\n\r");
                    send_q(w, d, &old);
                }
                if let Some(dd) = w.desc_mut(d) {
                    dd.str_target = Some(StrTarget::CharDescription(ch));
                    dd.str_buf.clear();
                    dd.max_str = 240;
                    dd.connected = CON_EXDSCR;
                }
            }
            Some('3') => {
                send_q(w, d, STORY);
                set_state(w, d, CON_RMOTD);
            }
            Some('4') => {
                send_q(w, d, "Enter a new password: ");
                set_echo(w, d, false);
                set_state(w, d, CON_PWDNEW);
            }
            _ => {
                send_q(w, d, "Wrong option.\n\r");
                send_q(w, d, MENU);
            }
        },
        CON_PWDNEW => {
            let pw = arg.trim_end();
            if pw.is_empty() || pw.len() > MAX_PWD_LENGTH {
                send_q(w, d, "Illegal password.\n\r");
                send_q(w, d, "Password: ");
                return;
            }
            if let Some(dd) = w.desc_mut(d) {
                dd.pwd = crate::crypto::hash_password(pw);
            }
            send_q(w, d, "Please retype password: ");
            set_state(w, d, CON_PWDNCNF);
        }
        CON_PWDNCNF => {
            let stored = w.desc(d).map(|x| x.pwd.clone()).unwrap_or_default();
            if !crate::crypto::verify_password(arg.trim_end(), &stored) {
                send_q(w, d, "Passwords don't match.\n\r");
                send_q(w, d, "Retype password: ");
                set_state(w, d, CON_PWDNEW);
                return;
            }
            set_echo(w, d, true);
            send_q(w, d, "\n\rDone. You must enter the game to make the change final\n\r");
            send_q(w, d, MENU);
            set_state(w, d, CON_SLCT);
        }
        _ => {
            log("Nanny: illegal state of con'ness");
        }
    }
}
