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
        match addr {
            0xFF44 => 0x90, // LY reg hack for test rom pass
            _ => self.mem[addr as usize],
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            // Serial Control Register intercept
            0xFF02 => {
                if (val & 0x80) != 0 {
                    // fetch sb (0xFF01)
                    let sb_val = self.mem[0xFF01];

                    // direct to stderr
                    eprint!("{}", sb_val as char);
                    use std::io::Write;
                    let _ = std::io::stderr().flush();

                    // clear bit 7 to indicate transfer completion
                    self.mem[0xFF02 as usize] = val & 0x7F;
                } else {
                    self.mem[addr as usize] = val;
                }
            }
            _ => {
                self.mem[addr as usize] = val;
            }
        }
    }
}
