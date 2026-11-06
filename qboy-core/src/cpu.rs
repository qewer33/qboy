use crate::{cpu::registers::Registers, mem::Mem};

pub mod registers;

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
            // LD a (bc)
            0x0A => {
                // get value at bc
                let val = self.read_cycle(mem, self.regs.bc());
                // set a to val
                self.regs.a = val;
            }
            // LD a (de)
            0x1A => {
                // get value at bc
                let val = self.read_cycle(mem, self.regs.de());
                // set a to val
                self.regs.a = val;
            }
            // LD a (hl+)
            0x2A => {
                // get value at hl
                let val = self.read_cycle(mem, self.regs.hl());
                // increment hl
                self.regs.set_hl(self.regs.hl().wrapping_add(1));
                // set a to val
                self.regs.a = val;
            }
            // LD a (hl-)
            0x3A => {
                // get value at hl
                let val = self.read_cycle(mem, self.regs.hl());
                // decrement hl
                self.regs.set_hl(self.regs.hl().wrapping_sub(1));
                // set a to val
                self.regs.a = val;
            }
            // LD (bc) a
            0x02 => {
                // write value of a to bc addr
                self.write_cycle(mem, self.regs.bc(), self.regs.a);
            }
            // LD (de) a
            0x12 => {
                // write value of a to de addr
                self.write_cycle(mem, self.regs.de(), self.regs.a);
            }
            // LD (hl+) a
            0x22 => {
                // write value of a to hl addr
                self.write_cycle(mem, self.regs.hl(), self.regs.a);
                // increment hl
                self.regs.set_hl(self.regs.hl().wrapping_add(1));
            }
            // LD (hl-) a
            0x32 => {
                // write value of a to hl addr
                self.write_cycle(mem, self.regs.hl(), self.regs.a);
                // decrement hl
                self.regs.set_hl(self.regs.hl().wrapping_sub(1));
            }
            // LD (u16) a
            0xEA => {
                // get operand
                let addr = self.fetch16(mem);
                // write value of a to addr
                self.write_cycle(mem, addr, self.regs.a);
                self.ensure_mcycles(4);
            }
            // LD a (u16)
            0xFA => {
                // get operand
                let addr = self.fetch16(mem);
                // get value
                let val = self.read_cycle(mem, addr);
                // write value to a
                self.regs.a = val;
                self.ensure_mcycles(4);
            }
            // LD (FF00+u8) a
            0xE0 => {
                // get operand
                let addr_off = self.fetch(mem);
                // write value of a to addr
                self.write_cycle(mem, 0xFF00 + (addr_off as u16), self.regs.a);
                self.ensure_mcycles(3);
            }
            // LD (FF00+u8)
            0xF0 => {
                // get operand
                let addr_off = self.fetch(mem);
                // get value
                let val = self.read_cycle(mem, 0xFF00 + (addr_off as u16));
                // write value to a
                self.regs.a = val;
                self.ensure_mcycles(3);
            }

            // jumps & subroutine instructions
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
            0x00..=0x3F if (opcode & 0x07) == 0x04 => {
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
            0x00..=0x3F if (opcode & 0x07) == 0x05 => {
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
                // set flags
                self.regs.set_flags(zero, true, hcarry, carry);
            }

            // 16-bit arithmetic instructions
            // ADD hl r16
            0x09 | 0x19 | 0x29 | 0x39 => {
                // get reg vall
                let reg_bits = (opcode >> 4) & 0x03;
                let val = self.read_reg16_bits(reg_bits);
                // perform addition
                let (res, _, hcarry, carry) = self.add16_with_flags(self.regs.hl(), val, 0);
                // set result + flags
                self.regs.set_hl(res);
                self.regs
                    .set_flags(self.regs.f_zero(), false, hcarry, carry);
            }
            // INC r16
            0x03 | 0x13 | 0x23 | 0x33 => {
                // get reg vall
                let reg_bits = (opcode >> 4) & 0x03;
                let val = self.read_reg16_bits(reg_bits);
                // set result
                self.write_reg16_bits(reg_bits, val.wrapping_add(1));
            }
            // DEC r16
            0x0B | 0x1B | 0x2B | 0x3B => {
                // get reg vall
                let reg_bits = (opcode >> 4) & 0x03;
                let val = self.read_reg16_bits(reg_bits);
                // set result
                self.write_reg16_bits(reg_bits, val.wrapping_sub(1));
            }

            // logical instructions
            // AND a r8
            0xA0..=0xA7 => {}
            // AND a u8
            0xE6 => {}

            // stack instructions
            // PUSH r16
            0xC5 | 0xD5 | 0xE5 => {
                // get reg val
                let reg_bits = (opcode >> 4) & 0x03;
                let val = self.read_reg16_bits(reg_bits);
                // push val to stack
                self.stack_push16(mem, val);
                self.ensure_mcycles(4);
            }
            // PUSH af
            0xF5 => {
                // push val to stack
                self.stack_push16(mem, self.regs.af());
                self.ensure_mcycles(4);
            }
            // POP r16
            0xC1 | 0xD1 | 0xE1 => {
                // pop val from stack
                let val = self.stack_pop16(mem);
                // write val to reg
                let reg_bits = (opcode >> 4) & 0x03;
                self.write_reg16_bits(reg_bits, val);
                self.ensure_mcycles(3);
            }
            // POP af
            0xF1 => {
                // pop val from stack
                let val = self.stack_pop16(mem);
                // set af to val
                self.regs.set_af(val);
                self.ensure_mcycles(3);
            }

            _ => panic!("unknown opcode: 0x{opcode:X}"),
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

    /// add the two 16-bit values and return the result along with flags
    /// as (reults, zero, hcarry, carry)
    fn add16_with_flags(&self, a: u16, b: u16, c: u16) -> (u16, bool, bool, bool) {
        // widen vals
        let a_32 = a as u32;
        let b_32 = b as u32;
        let c_32 = c as u32;

        // calculate res
        let res_32 = a_32 + b_32 + c_32;
        let res_16 = res_32 as u16;

        // calculate flags
        let zero = res_16 == 0;
        let hcarry = ((a & 0x00FF) + (b & 0x00FF) + c) > 0x00FF; // check if lower bytes overflow
        let carry = res_32 > 0xFFFF;

        (res_16, zero, hcarry, carry)
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
