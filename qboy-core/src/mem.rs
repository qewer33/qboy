pub trait Mem {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);
}

#[derive(Copy, Clone)]
pub struct FlatMem {
    mem: [u8; 0x10000],
}

impl FlatMem {
    pub fn new() -> Self {
        FlatMem { mem: [0; 0x10000] }
    }

    pub fn load(&mut self, addr: u8, bytes: &[u8]) {
        let start = addr as usize;
        let end = start + bytes.len();
        self.mem[start..end].copy_from_slice(bytes);
    }
}

impl Mem for FlatMem {
    fn read(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn write(&mut self, addr: u16, val: u8) {
        self.mem[addr as usize] = val;
    }
}
