use crate::dram::{Dram, MemoryError, DRAM_SIZE};

#[derive(Debug, Clone)]
pub struct Bus {
    pub dram: Dram,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            dram: Dram::new(DRAM_SIZE),
        }
    }

    pub fn with_dram_size(size: usize) -> Self {
        Self {
            dram: Dram::new(size),
        }
    }

    pub fn read8(&self, addr: u32) -> Result<u8, MemoryError> {
        self.dram.read8(addr)
    }

    pub fn read16(&self, addr: u32) -> Result<u16, MemoryError> {
        self.dram.read16(addr)
    }

    pub fn read32(&self, addr: u32) -> Result<u32, MemoryError> {
        self.dram.read32(addr)
    }

    pub fn write8(&mut self, addr: u32, val: u8) -> Result<(), MemoryError> {
        self.dram.write8(addr, val)
    }

    pub fn write16(&mut self, addr: u32, val: u16) -> Result<(), MemoryError> {
        self.dram.write16(addr, val)
    }

    pub fn write32(&mut self, addr: u32, val: u32) -> Result<(), MemoryError> {
        self.dram.write32(addr, val)
    }

    pub fn load(&mut self, addr: u32, bytes: &[u8]) {
        self.dram.load(addr, bytes);
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bus_delegation() {
        let mut bus = Bus::with_dram_size(1024);
        bus.write32(0x100, 0xCAFE_BABE).unwrap();
        assert_eq!(bus.read32(0x100).unwrap(), 0xCAFE_BABE);
    }
}
