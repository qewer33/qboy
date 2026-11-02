use crate::{bus::Bus, cpu::Cpu};

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
}
