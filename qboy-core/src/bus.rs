use crate::{cartridge::Cartridge, mem::Mem, ppu::Ppu};

#[derive(Default)]
pub struct Bus {
    cartridge: Cartridge,
    ppu: Ppu,
}

impl Mem for Bus {
    fn read(&self, addr: u16) -> u8 {
        // bus read and writes should be match on address ranges
        // handling different parts of the address space
        todo!()
    }

    fn write(&mut self, addr: u16, val: u8) {
        todo!()
    }
}
