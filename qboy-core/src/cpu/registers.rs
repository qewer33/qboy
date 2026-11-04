use crate::mem::Mem;

/// DMG-CPU registers.
///
/// ref: https://gbdev.io/pandocs/CPU_Registers_and_Flags.htmladd
#[derive(Default)]
pub struct Registers {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

impl Registers {
    /// init registers as they should be after boot rom
    ///
    /// ref: https://github.com/robert/gameboy-doctor#2-make-2-tweaks-to-your-emulator
    pub fn after_boot() -> Self {
        Self {
            a: 0x01,
            f: 0xB0,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            sp: 0xFFFE,
            pc: 0x0100,
        }
    }

    /// log registers in gameboy-doctor format
    ///
    /// ref: https://github.com/robert/gameboy-doctor#3-log-the-state-of-your-cpu
    pub fn log_state<M: Mem>(&self, mem: &M) -> String {
        format!(
            "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:02X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}",
            self.a,
            self.f,
            self.b,
            self.c,
            self.d,
            self.e,
            self.h,
            self.l,
            self.sp,
            self.pc,
            mem.read(self.pc),
            mem.read(self.pc + 1),
            mem.read(self.pc + 2),
            mem.read(self.pc + 3)
        )
    }

    // pair helpers
    pub fn af(&self) -> u16 {
        (self.a as u16) << 8 | self.f as u16
    }

    pub fn set_af(&mut self, val: u16) {
        self.a = (val >> 8) as u8;
        self.f = val as u8; // as u8 keeps the low byte
    }

    pub fn bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    pub fn set_bc(&mut self, val: u16) {
        self.b = (val >> 8) as u8;
        self.c = val as u8; // as u8 keeps the low byte
    }

    pub fn de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }

    pub fn set_de(&mut self, val: u16) {
        self.d = (val >> 8) as u8;
        self.e = val as u8;
    }

    pub fn hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    pub fn set_hl(&mut self, val: u16) {
        self.h = (val >> 8) as u8;
        self.l = val as u8;
    }

    // flag helpers
    pub fn f_zero(&self) -> bool {
        (self.f & 0x80) != 0
    }

    pub fn f_sub(&self) -> bool {
        (self.f & 0x40) != 0
    }

    pub fn f_hcarry(&self) -> bool {
        (self.f & 0x20) != 0
    }

    pub fn f_carry(&self) -> bool {
        (self.f & 0x10) != 0
    }

    pub fn set_flags(&mut self, zero: bool, sub: bool, hcarry: bool, carry: bool) {
        self.set_f_zero(zero);
        self.set_f_sub(sub);
        self.set_f_hcarry(hcarry);
        self.set_f_carry(carry);
    }

    pub fn set_f_zero(&mut self, val: bool) {
        if val {
            self.f |= 0x80;
        } else {
            self.f &= !0x80;
        }
    }

    pub fn set_f_sub(&mut self, val: bool) {
        if val {
            self.f |= 0x40;
        } else {
            self.f &= !0x40;
        }
    }

    pub fn set_f_hcarry(&mut self, val: bool) {
        if val {
            self.f |= 0x20;
        } else {
            self.f &= !0x20;
        }
    }

    pub fn set_f_carry(&mut self, val: bool) {
        if val {
            self.f |= 0x10;
        } else {
            self.f &= !0x10;
        }
    }
}
