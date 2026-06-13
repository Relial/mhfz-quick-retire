#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub enum GameMode {
    LowGrade,
    HighGrade,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MainDllInfo {
    pub game_mode: GameMode,
    pub address: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Addresses {
    pub main_dll_info: MainDllInfo,
    pub quest_update: usize,
    quest_info: usize,
    encryption1: usize,
    encryption2: usize,
    encryption3: u16,
}

impl Addresses {
    pub fn new(main_dll_info: MainDllInfo) -> Self {
        let dll = main_dll_info.address;
        match main_dll_info.game_mode {
            GameMode::LowGrade => Self {
                main_dll_info,
                quest_update: dll + 0x880380,
                quest_info: dll + 0x5bc85bc,
                encryption1: dll + 0x1a52b5c,
                encryption2: dll + 0x617ce86,
                encryption3: 0xb7a0,
            },
            GameMode::HighGrade => Self {
                main_dll_info,
                quest_update: dll + 0x89be30,
                quest_info: dll + 0xe8001ec,
                encryption1: dll + 0x1a422c4,
                encryption2: dll + 0xedb7626,
                encryption3: 0x5ec0,
            },
        }
    }

    pub fn quest(&self) -> Option<Quest> {
        Quest::from_addr(self.quest_info)
    }
}

pub struct Quest(*mut u8);

impl Quest {
    fn from_addr(addr: usize) -> Option<Self> {
        unsafe {
            let ptr = (addr as *const *mut u8).read();
            if ptr.is_null() {
                return None;
            }
            Some(Self(ptr))
        }
    }

    pub fn complete(&self) -> bool {
        let state = unsafe { self.0.read() };
        if state != 1 {
            return false;
        }
        // This is so we don't mess with the timer before erupe's quest completion timestamp logs it.
        let substate = unsafe { self.0.wrapping_byte_add(1).read() };
        substate > 2
    }

    pub fn set_remaining_time(&self, time: u32) {
        unsafe {
            let ptr = self.0.wrapping_byte_add(0x10) as *mut u32;
            ptr.write(time);
        }
    }

    pub fn remaining_carts(&self) -> u32 {
        unsafe {
            let ptr = self.0.wrapping_byte_add(0x1C) as *const u32;
            ptr.read()
        }
    }

    pub fn max_carts(&self) -> Option<u32> {
        unsafe {
            let ptr = (self.0.wrapping_byte_add(0x80) as *const *const u32).read();
            if ptr.is_null() {
                return None;
            }
            Some(ptr.wrapping_byte_add(0x14).read())
        }
    }

    pub fn carted_count(&self) -> u32 {
        if let Some(max_carts) = self.max_carts() {
            max_carts.saturating_sub(self.remaining_carts())
        } else {
            0
        }
    }
}

pub struct Player(*mut u8);

impl Player {
    pub fn from_addr(addr: u32) -> Self {
        Self(addr as *mut u8)
    }

    fn struct_idx(&self) -> u16 {
        unsafe { (self.0.wrapping_byte_add(0xC) as *const u16).read() }
    }

    pub fn current_health(&self, addresses: &Addresses) -> u16 {
        unsafe {
            let health = (self.0.wrapping_byte_add(0x624) as *const u16).read();
            let idx = self.struct_idx();
            let key = (addresses.encryption1 as *const u16).read();
            let p = (addresses.encryption2 - idx as usize * 2) as *const u16;
            !(p.read()) ^ health.rotate_right(3) ^ key ^ addresses.encryption3
        }
    }
}
