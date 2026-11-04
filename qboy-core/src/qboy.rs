use std::fs;

use crate::{
    bus::Bus,
    cpu::{Cpu, registers::Registers},
    mem::FlatMem,
};

pub struct QBoy {
    cpu: Cpu,
    bus: Bus,
}

impl QBoy {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            bus: Bus::default(),
        }
    }

    pub fn exec_cpu_test_rom(&mut self, romc: usize) {
        const ROMS: [&str; 11] = [
            "01-special.gb",
            "02-interrupts.gb",
            "03-op sp,hl.gb",
            "04-op r,imm.gb",
            "05-op rp.gb",
            "06-ld r,r.gb",
            "07-jr,jp,call,ret,rst.gb",
            "08-misc instrs.gb",
            "09-op r,r.gb",
            "10-bit ops.gb",
            "11-op a,(hl).gb",
        ];

        let rom_path = format!("./roms/tests/{}", ROMS[romc - 1]);
        let rom = fs::read(rom_path).unwrap();

        let mut cpu = Cpu::new();
        cpu.regs = Registers::after_boot();

        let mut mem = FlatMem::new();
        mem.load(0, rom.as_slice());

        println!("{}", cpu.regs.log_state(&mem));

        while !cpu.halted {
            cpu.step(&mut mem);
            println!("{}", cpu.regs.log_state(&mem));
        }
    }
}
