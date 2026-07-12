use bunny_plugin::{GameMode, MhfoInfo};
use mhfz_structs::{
    player::{Player, PlayerInfo},
    quest::QuestInfo,
};

#[derive(Clone, Copy, Debug)]
pub struct Addresses {
    player_structs: usize,
    player_info: usize,
    quest_info: usize,
    pub encryption1: usize,
    pub encryption2: usize,
    pub encryption3: u16,
    iframe_flags: usize,
}

impl Addresses {
    pub fn new(mhfo_info: MhfoInfo) -> Self {
        let dll = mhfo_info.address;
        match mhfo_info.game_mode {
            GameMode::LowGrade => Self {
                player_structs: dll + 0x5033b90,
                player_info: dll + 0x5bc830c,
                quest_info: dll + 0x5bc85bc,
                encryption1: dll + 0x1a52b5c,
                encryption2: dll + 0x617ce86,
                encryption3: 0xb7a0,
                iframe_flags: dll + 0x5740db0,
            },
            GameMode::HighGrade => Self {
                player_structs: dll + 0xDC6B750,
                player_info: dll + 0xE7FFF3C,
                quest_info: dll + 0xe8001ec,
                encryption1: dll + 0x1a422c4,
                encryption2: dll + 0xedb7626,
                encryption3: 0x5ec0,
                iframe_flags: dll + 0xe378970,
            },
        }
    }

    pub fn player_info(&self) -> Option<PlayerInfo> {
        let ptr = unsafe { (self.player_info as *const *mut u8).read() };
        PlayerInfo::new(ptr)
    }

    pub fn own_player(&self) -> Option<Player> {
        let info = self.player_info()?;
        Some(Player::from_idx(
            self.player_structs as *mut u8,
            info.player_struct_idx() as usize,
        ))
    }

    pub fn quest(&self) -> Option<QuestInfo> {
        let ptr = unsafe { (self.quest_info as *const *mut u8).read() };
        QuestInfo::new(ptr)
    }

    pub fn iframe_flags(&self) -> *mut u8 {
        self.iframe_flags as *mut u8
    }
}
