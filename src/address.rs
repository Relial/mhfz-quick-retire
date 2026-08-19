use bunny_plugin::{GameMode, MhfoInfo};

#[derive(Clone, Copy, Debug)]
pub struct Addresses {
    iframe_flags: usize,
}

impl Addresses {
    pub fn new(mhfo_info: MhfoInfo) -> Self {
        let dll = mhfo_info.address;
        match mhfo_info.game_mode {
            GameMode::LowGrade => Self {
                iframe_flags: dll + 0x5740db0,
            },
            GameMode::HighGrade => Self {
                iframe_flags: dll + 0xe378970,
            },
        }
    }

    pub fn iframe_flags(&self) -> *mut u8 {
        self.iframe_flags as *mut u8
    }
}
