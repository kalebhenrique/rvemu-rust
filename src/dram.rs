pub const DRAM_SIZE: usize = 1024 * 1024; // 1 MB de memória RAM padrão

#[derive(Debug, PartialEq, Eq)]
pub enum MemoryError {
    OutOfBounds(u32),
}

#[derive(Debug, Clone)]
pub struct Dram {
    pub data: Vec<u8>,
}

impl Dram {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    // Leitura de 1 byte (8 bits)
    pub fn read8(&self, addr: u32) -> Result<u8, MemoryError> {
        let index = addr as usize;
        if index < self.data.len() {
            Ok(self.data[index])
        } else {
            Err(MemoryError::OutOfBounds(addr))
        }
    }

    // Leitura de half-word (16 bits) em Little-Endian
    pub fn read16(&self, addr: u32) -> Result<u16, MemoryError> {
        let index = addr as usize;
        if index + 1 < self.data.len() {
            let bytes = [self.data[index], self.data[index + 1]];
            Ok(u16::from_le_bytes(bytes))
        } else {
            Err(MemoryError::OutOfBounds(addr))
        }
    }

    // Leitura de word (32 bits) em Little-Endian
    pub fn read32(&self, addr: u32) -> Result<u32, MemoryError> {
        let index = addr as usize;
        if index + 3 < self.data.len() {
            let bytes = [
                self.data[index],
                self.data[index + 1],
                self.data[index + 2],
                self.data[index + 3],
            ];
            Ok(u32::from_le_bytes(bytes))
        } else {
            Err(MemoryError::OutOfBounds(addr))
        }
    }

    // Escrita de 1 byte (8 bits)
    pub fn write8(&mut self, addr: u32, val: u8) -> Result<(), MemoryError> {
        let index = addr as usize;
        if index < self.data.len() {
            self.data[index] = val;
            Ok(())
        } else {
            Err(MemoryError::OutOfBounds(addr))
        }
    }

    // Escrita de half-word (16 bits) em Little-Endian
    pub fn write16(&mut self, addr: u32, val: u16) -> Result<(), MemoryError> {
        let index = addr as usize;
        if index + 1 < self.data.len() {
            let bytes = val.to_le_bytes();
            self.data[index] = bytes[0];
            self.data[index + 1] = bytes[1];
            Ok(())
        } else {
            Err(MemoryError::OutOfBounds(addr))
        }
    }

    // Escrita de word (32 bits) em Little-Endian
    pub fn write32(&mut self, addr: u32, val: u32) -> Result<(), MemoryError> {
        let index = addr as usize;
        if index + 3 < self.data.len() {
            let bytes = val.to_le_bytes();
            self.data[index..index + 4].copy_from_slice(&bytes);
            Ok(())
        } else {
            Err(MemoryError::OutOfBounds(addr))
        }
    }

    // Carrega um bloco de bytes diretamente na memória
    pub fn load(&mut self, addr: u32, bytes: &[u8]) {
        let start = addr as usize;
        let end = start + bytes.len();
        if end <= self.data.len() {
            self.data[start..end].copy_from_slice(bytes);
        }
    }
}

impl Default for Dram {
    fn default() -> Self {
        Self::new(DRAM_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dram_read_write_byte() {
        let mut dram = Dram::new(1024);
        dram.write8(0x10, 0xAB).unwrap();
        assert_eq!(dram.read8(0x10).unwrap(), 0xAB);
    }

    #[test]
    fn test_little_endian_half_word() {
        let mut dram = Dram::new(1024);
        // 0xBBAA gravado em Little-Endian: 0xAA no byte 0, 0xBB no byte 1
        dram.write16(0x0, 0xBBAA).unwrap();

        assert_eq!(dram.read8(0x0).unwrap(), 0xAA);
        assert_eq!(dram.read8(0x1).unwrap(), 0xBB);
        assert_eq!(dram.read16(0x0).unwrap(), 0xBBAA);
    }

    #[test]
    fn test_little_endian_word() {
        let mut dram = Dram::new(1024);
        // 0x12345678: LSB é 0x78, MSB é 0x12
        dram.write32(0x0, 0x1234_5678).unwrap();

        assert_eq!(dram.read8(0x0).unwrap(), 0x78);
        assert_eq!(dram.read8(0x1).unwrap(), 0x56);
        assert_eq!(dram.read8(0x2).unwrap(), 0x34);
        assert_eq!(dram.read8(0x3).unwrap(), 0x12);

        assert_eq!(dram.read32(0x0).unwrap(), 0x1234_5678);
    }

    #[test]
    fn test_out_of_bounds() {
        let mut dram = Dram::new(1024);
        assert_eq!(dram.read8(1024), Err(MemoryError::OutOfBounds(1024)));
        assert_eq!(
            dram.write32(1022, 0x1234),
            Err(MemoryError::OutOfBounds(1022))
        );
    }
}
