use crate::mem::Mem;

/// the main DMG-CPU processing unit.
/// owns registers and handles fetch/decode/execute
pub struct Cpu {
    pub regs: Registers,
    pub mcycle: u8,   // M-cycle count
    pub halted: bool, // halt state
    pub ime: bool,    // IME control flag
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            regs: Registers::default(),
            mcycle: 0,
            halted: false,
            ime: true,
        }
    }

    /// step the cpu
    pub fn step<M: Mem>(&mut self, mem: &mut M) -> u8 {
        // reset cycle before step
        self.mcycle = 0;

        if !self.halted {
            // fetch & execute
            let opcode = self.fetch(mem);
            self.exec(opcode, mem);
        } else {
            // consume cycle
            self.mcycle += 1

            // TODO: wakeup check here
        }

        self.mcycle
    }

    /// fetch the byte at pc
    fn fetch<M: Mem>(&mut self, mem: &mut M) -> u8 {
        // get the byte at pc
        let byte = self.read_cycle(mem, self.regs.pc);
        // increment pc
        self.regs.pc = self.regs.pc.wrapping_add(1);

        byte
    }

    /// fetch a 16 bit little endian value at pc
    fn fetch16<M: Mem>(&mut self, mem: &mut M) -> u16 {
        // get low first then high
        let low = self.fetch(mem);
        let high = self.fetch(mem);

        // combine as little endian
        (high as u16) << 8 | low as u16
    }

    /// decode & execute
    ///
    /// opcode ref: https://izik1.github.io/gbops/
    fn exec<M: Mem>(&mut self, opcode: u8, mem: &mut M) {
        match opcode {
            // misc instructions
            // NOP
            0x00 => {
                return;
            }
            // HALT
            0x76 => {
                self.halted = true;
            }
            // EI
            0xFB => {
                self.ime = true;
            }
            // DI
            0xF3 => {
                self.ime = false;
            }

            // load instructions
            // LD r8 r8
            0x40..=0x7F if opcode != 0x76 => {
                // get reg bits
                let dest_bits = (opcode >> 3) & 0x07;
                let src_bits = opcode & 0x07;
                // write src val to dst
                let val = self.read_reg_bits(mem, src_bits);
                self.write_reg_bits(mem, dest_bits, val);
            }
            // LD r8 u8
            0x00..=0x3F if (opcode & 0x07) == 0x06 => {
                // get reg bits
                let dest_bits = (opcode >> 3) & 0x07;
                // fetch operand
                let val = self.fetch(mem);
                // write val to dst
                self.write_reg_bits(mem, dest_bits, val);
            }
            // LD r16 u16
            0x00..=0x3F if (opcode & 0x0F) == 0x01 => {
                // get reg16 bits
                let dest_bits = (opcode >> 4) & 0x03;
                // fetch operand
                let val = self.fetch16(mem);
                // write val to dst
                self.write_reg16_bits(dest_bits, val);
            }

            // jump instructions
            // JP u16
            0xC3 => {
                // set pc to operand
                self.regs.pc = self.fetch16(mem);
            }
            // JP hl
            0xE9 => {
                // set pc to hl
                self.regs.pc = self.regs.hl();
            }
            // JP cond u16
            0xC2 | 0xCA | 0xD2 | 0xDA => {
                // get condition bits
                let cond_bits = (opcode >> 3) & 0x03;

                if self.check_jmp_condition(cond_bits) {
                    // set PC to operand
                    self.regs.pc = self.fetch16(mem);
                    self.ensure_mcycles(4);
                } else {
                    // increment PC over operand
                    self.regs.pc = self.regs.pc.wrapping_add(2);
                    self.ensure_mcycles(3);
                }
            }
            // JR i8
            0x18 => {
                // get operand as signed
                let offset = i8::from_ne_bytes([self.fetch(mem)]);
                // apply offset to PC
                self.regs.pc = self.regs.pc.wrapping_add_signed(offset as i16);
                self.ensure_mcycles(3);
            }
            // JR cond i8
            0x20 | 0x28 | 0x30 | 0x38 => {
                // get condition bits
                let cond_bits = (opcode >> 3) & 0x03;

                if self.check_jmp_condition(cond_bits) {
                    // get operand as signed
                    let offset = i8::from_ne_bytes([self.fetch(mem)]);
                    // apply offset to PC
                    self.regs.pc = self.regs.pc.wrapping_add_signed(offset as i16);
                    self.ensure_mcycles(3);
                } else {
                    // increment PC over operand
                    self.regs.pc = self.regs.pc.wrapping_add(1);
                    self.ensure_mcycles(2);
                }
            }

            // subroutine instructions
            // CALL u16
            0xCD => {
                // get operand
                let addr = self.fetch16(mem);
                // push return addr (PC) to stack
                self.stack_push16(mem, self.regs.pc);
                // set PC to operand
                self.regs.pc = addr;
                self.ensure_mcycles(6);
            }
            // CALL cond u16
            0xC4 | 0xCC | 0xD4 | 0xDC => {
                // get condition bits
                let cond_bits = (opcode >> 3) & 0x03;

                if self.check_jmp_condition(cond_bits) {
                    // get operand
                    let addr = self.fetch16(mem);
                    // push return addr (PC) to stack
                    self.stack_push16(mem, self.regs.pc);
                    // set PC to operand
                    self.regs.pc = addr;
                    self.ensure_mcycles(6);
                } else {
                    // increment PC over operand
                    self.regs.pc = self.regs.pc.wrapping_add(1);
                    self.ensure_mcycles(3);
                }
            }
            // RET
            0xC9 => {
                // get operand
                let addr = self.stack_pop16(mem);
                // set PC to operand
                self.regs.pc = addr;
                self.ensure_mcycles(4);
            }
            // RET cond
            0xC0 | 0xC8 | 0xD0 | 0xD8 => {
                // get condition bits
                let cond_bits = (opcode >> 3) & 0x03;

                if self.check_jmp_condition(cond_bits) {
                    // get operand
                    let addr = self.stack_pop16(mem);
                    // set PC to operand
                    self.regs.pc = addr;
                    self.ensure_mcycles(4);
                } else {
                    self.ensure_mcycles(2);
                }
            }
            // RETI
            0xD9 => {
                // enable interrupts
                self.ime = true;
                // get operand
                let addr = self.stack_pop16(mem);
                // set PC to operand
                self.regs.pc = addr;
                self.ensure_mcycles(4);
            }
            // RST vec
            0xC0..=0xFF if (opcode & 0x07) == 0x07 => {
                // get target bits
                let target_bits = (opcode >> 3) & 0x07;
                // get target address
                let target_addr = match target_bits & 0x07 {
                    0 => 0x0000,
                    1 => 0x0008,
                    2 => 0x0010,
                    3 => 0x0018,
                    4 => 0x0020,
                    5 => 0x0028,
                    6 => 0x0030,
                    7 => 0x0038,
                    _ => unreachable!(),
                };
                // set PC to operand
                self.regs.pc = target_addr;
                self.ensure_mcycles(4);
            }

            // 8-bit arithmetic instructions
            // ADD a r8
            0x80..=0x87 => {
                // get reg val
                let reg_bits = opcode & 0x07;
                let val = self.read_reg_bits(mem, reg_bits);
                // perform addition
                let (res, zero, hcarry, carry) = self.add_with_flags(self.regs.a, val, 0);
                // set result + flags
                self.regs.a = res;
                self.regs.set_flags(zero, false, hcarry, carry);
            }
            // ADD a u8
            0xC6 => {
                // get operand
                let val = self.fetch(mem);
                // perform addition
                let (res, zero, hcarry, carry) = self.add_with_flags(self.regs.a, val, 0);
                // set result + flags
                self.regs.a = res;
                self.regs.set_flags(zero, false, hcarry, carry);
            }
            // ADC a r8
            0x88..=0x8F => {
                // get reg val
                let reg_bits = opcode & 0x07;
                let val = self.read_reg_bits(mem, reg_bits);
                // perform addition
                let (res, zero, hcarry, carry) =
                    self.add_with_flags(self.regs.a, val, self.regs.f_carry() as u8);
                // set (result + previous carry) + flags
                self.regs.a = res;
                self.regs.set_flags(zero, false, hcarry, carry);
            }
            // ADC a u8
            0xCE => {
                // get operand
                let val = self.fetch(mem);
                // perform addition
                let (res, zero, hcarry, carry) =
                    self.add_with_flags(self.regs.a, val, self.regs.f_carry() as u8);
                // set result + flags
                self.regs.a = res;
                self.regs.set_flags(zero, false, hcarry, carry);
            }
            // SUB a r8
            0x90..=0x97 => {
                // get reg val
                let reg_bits = opcode & 0x07;
                let val = self.read_reg_bits(mem, reg_bits);
                // perform subtraction
                let (res, zero, hcarry, carry) = self.sub_with_flags(self.regs.a, val, 0);
                // set result + flags
                self.regs.a = res;
                self.regs.set_flags(zero, true, hcarry, carry);
            }
            // SUB a u8
            0xD6 => {
                // get operand
                let val = self.fetch(mem);
                // perform subtraction
                let (res, zero, hcarry, carry) = self.sub_with_flags(self.regs.a, val, 0);
                // set result + flags
                self.regs.a = res;
                self.regs.set_flags(zero, true, hcarry, carry);
            }
            // SBC a r8
            0x98..=0x9f => {
                // get reg val
                let reg_bits = opcode & 0x07;
                let val = self.read_reg_bits(mem, reg_bits);
                // perform subtraction
                let (res, zero, hcarry, carry) =
                    self.sub_with_flags(self.regs.a, val, self.regs.f_carry() as u8);
                // set result + flags
                self.regs.a = res;
                self.regs.set_flags(zero, true, hcarry, carry);
            }
            // SBC a u8
            0xDE => {
                // get operand
                let val = self.fetch(mem);
                // perform subtraction
                let (res, zero, hcarry, carry) =
                    self.sub_with_flags(self.regs.a, val, self.regs.f_carry() as u8);
                // set result + flags
                self.regs.a = res;
                self.regs.set_flags(zero, true, hcarry, carry);
            }
            // INC r8
            0x00..=0x3F if (opcode & 0x07) == 0x03 => {
                // get reg val
                let reg_bits = (opcode >> 3) & 0x03;
                let val = self.read_reg_bits(mem, reg_bits);
                // perform increment
                let (res, zero, hcarry, _) = self.add_with_flags(val, 1, 0);
                // set result + flags
                self.write_reg_bits(mem, reg_bits, res);
                self.regs
                    .set_flags(zero, false, hcarry, self.regs.f_carry());
            }
            // DEC r8
            0x00..=0x3F if (opcode & 0x07) == 0x0A => {
                // get reg val
                let reg_bits = (opcode >> 3) & 0x03;
                let val = self.read_reg_bits(mem, reg_bits);
                // perform increment
                let (res, zero, hcarry, _) = self.sub_with_flags(val, 1, 0);
                // set result + flags
                self.write_reg_bits(mem, reg_bits, res);
                self.regs.set_flags(zero, true, hcarry, self.regs.f_carry());
            }
            // CP a r8
            0xB8..=0xBF => {
                // get reg val
                let reg_bits = opcode & 0x07;
                let val = self.read_reg_bits(mem, reg_bits);
                // perform subtraction
                let (_, zero, hcarry, carry) = self.sub_with_flags(self.regs.a, val, 0);
                // set result + flags
                self.regs.set_flags(zero, true, hcarry, carry);
            }
            // CP a u8
            0xFE => {
                // get operand
                let val = self.fetch(mem);
                // perform subtraction
                let (_, zero, hcarry, carry) = self.sub_with_flags(self.regs.a, val, 0);
                // set result + flags
                self.regs.set_flags(zero, true, hcarry, carry);
            }

            // logical instructions
            // AND a r8
            0xA0..=0xA7 => {}
            // AND a u8
            0xE6 => {}

            _ => panic!("unknown opcode"),
        }
    }

    // mem helpers

    /// mem read with M-cycle increment
    fn read_cycle<M: Mem>(&mut self, mem: &mut M, addr: u16) -> u8 {
        // increment mcycle
        self.mcycle += 1;

        mem.read(addr)
    }

    /// mem write with M-cycle increment
    fn write_cycle<M: Mem>(&mut self, mem: &mut M, addr: u16, val: u8) {
        // increment mcycle
        self.mcycle += 1;

        mem.write(addr, val);
    }

    // register helpers

    /// register read helper for instructions
    pub fn read_reg_bits<M: Mem>(&mut self, mem: &mut M, bits: u8) -> u8 {
        match bits & 0x07 {
            0 => self.regs.b,
            1 => self.regs.c,
            2 => self.regs.d,
            3 => self.regs.e,
            4 => self.regs.h,
            5 => self.regs.l,
            6 => self.read_cycle(mem, self.regs.hl()),
            7 => self.regs.a,
            _ => unreachable!(),
        }
    }

    /// register write helper for instructions
    pub fn write_reg_bits<M: Mem>(&mut self, mem: &mut M, bits: u8, val: u8) {
        match bits & 0x07 {
            0 => self.regs.b = val,
            1 => self.regs.c = val,
            2 => self.regs.d = val,
            3 => self.regs.e = val,
            4 => self.regs.h = val,
            5 => self.regs.l = val,
            6 => {
                self.write_cycle(mem, self.regs.hl(), val);
            }
            7 => self.regs.a = val,
            _ => unreachable!(),
        }
    }

    /// 16-bit register read helper for instructions
    pub fn read_reg16_bits(&mut self, bits: u8) -> u16 {
        match bits & 0x03 {
            0 => self.regs.bc(),
            1 => self.regs.de(),
            2 => self.regs.hl(),
            3 => self.regs.sp,
            _ => unreachable!(),
        }
    }

    /// 16-bit register write helper for instructions
    pub fn write_reg16_bits(&mut self, bits: u8, val: u16) {
        match bits & 0x03 {
            0 => self.regs.set_bc(val),
            1 => self.regs.set_de(val),
            2 => self.regs.set_hl(val),
            3 => self.regs.sp = val,
            _ => unreachable!(),
        }
    }

    /// check JP/JR instruction condition
    pub fn check_jmp_condition(&self, bits: u8) -> bool {
        match bits & 0x03 {
            0 => !self.regs.f_zero(),
            1 => self.regs.f_zero(),
            2 => !self.regs.f_carry(),
            3 => self.regs.f_carry(),
            _ => unreachable!(),
        }
    }

    // stack helpers

    fn stack_push16<M: Mem>(&mut self, mem: &mut M, val: u16) {
        // get high and low bytes
        let high = (val >> 8) as u8;
        let low = val as u8;

        // push addr to stack (high first then low)
        self.regs.sp = self.regs.sp.wrapping_sub(1);
        self.write_cycle(mem, self.regs.sp, high);
        self.regs.sp = self.regs.sp.wrapping_sub(1);
        self.write_cycle(mem, self.regs.sp, low);
    }

    fn stack_pop16<M: Mem>(&mut self, mem: &mut M) -> u16 {
        // pop from stack (low first then high)
        let low = mem.read(self.regs.sp);
        self.regs.sp = self.regs.sp.wrapping_add(1);
        let high = mem.read(self.regs.sp);
        self.regs.sp = self.regs.sp.wrapping_add(1);

        // merge high and low
        (high as u16) << 8 | low as u16
    }

    // arithmetic helpers

    /// add the two values and return the result along with flags
    /// as (reults, zero, hcarry, carry)
    fn add_with_flags(&self, a: u8, b: u8, c: u8) -> (u8, bool, bool, bool) {
        // widen vals
        let a_16 = a as u16;
        let b_16 = b as u16;
        let c_16 = c as u16;

        // calculate res
        let res_16 = a_16 + b_16 + c_16;
        let res_8 = res_16 as u8;

        // calculate flags
        let zero = res_8 == 0;
        let hcarry = ((a & 0x0F) + (b & 0x0F) + c) > 0x0F; // check if lower nibbles overflow
        let carry = res_16 > 0xFF;

        (res_8, zero, hcarry, carry)
    }

    /// subs the two values and return the result along with flags
    /// as (reults, zero, hcarry, carry)
    fn sub_with_flags(&self, a: u8, b: u8, c: u8) -> (u8, bool, bool, bool) {
        // widen vals to signed
        let a_16s = a as i16;
        let b_16s = b as i16;
        let c_16s = c as i16;

        // calculate res
        let res_16 = a_16s - b_16s - c_16s;
        let res_8 = res_16 as u8;

        // calculate flags
        let zero = res_8 == 0;
        let hcarry = (a_16s & 0x0F) - (b_16s & 0x0F) - c_16s < 0; // check if lower nibbles overflow
        let carry = res_16 < 0;

        (res_8, zero, hcarry, carry)
    }

    // misc helpers

    /// pass mcycles (used for matching missed instr cycles)
    fn pass_mcycles(&mut self, count: u8) {
        self.mcycle += count;
    }

    /// set mcycles to a specific value
    fn ensure_mcycles(&mut self, count: u8) {
        self.mcycle = count;
    }
}

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
    pub fn log_state<M: Mem>(&self, mem: M) -> String {
        format!(
            "A:{} F:{} B:{} C:{} D:{} E:{} H:{} L:{} SP:{} PC:{} PCMEM:{},{},{},{}",
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
