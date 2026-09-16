// Opcodes base do RV32I
pub const OP_LOAD: u32 = 0x03;
pub const OP_FENCE: u32 = 0x0F;
pub const OP_IMM: u32 = 0x13;
pub const OP_AUIPC: u32 = 0x17;
pub const OP_STORE: u32 = 0x23;
pub const OP_REG: u32 = 0x33;
pub const OP_LUI: u32 = 0x37;
pub const OP_BRANCH: u32 = 0x63;
pub const OP_JALR: u32 = 0x67;
pub const OP_JAL: u32 = 0x6F;
pub const OP_SYSTEM: u32 = 0x73;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instruction(pub u32);

impl Instruction {
    // Extrai o opcode (bits 6:0)
    pub fn opcode(&self) -> u32 {
        self.0 & 0x7F
    }

    // Extrai registrador destino rd (bits 11:7)
    pub fn rd(&self) -> usize {
        ((self.0 >> 7) & 0x1F) as usize
    }

    // Extrai função de 3 bits funct3 (bits 14:12)
    pub fn funct3(&self) -> u32 {
        (self.0 >> 12) & 0x07
    }

    // Extrai primeiro registrador fonte rs1 (bits 19:15)
    pub fn rs1(&self) -> usize {
        ((self.0 >> 15) & 0x1F) as usize
    }

    // Extrai segundo registrador fonte rs2 (bits 24:20)
    pub fn rs2(&self) -> usize {
        ((self.0 >> 20) & 0x1F) as usize
    }

    // Extrai função de 7 bits funct7 (bits 31:25)
    pub fn funct7(&self) -> u32 {
        (self.0 >> 25) & 0x7F
    }

    // Imediato Tipo-I (12 bits com extensão de sinal)
    pub fn imm_i(&self) -> u32 {
        ((self.0 as i32) >> 20) as u32
    }

    // Imediato Tipo-S (bits [31:25] e [11:7] com extensão de sinal)
    pub fn imm_s(&self) -> u32 {
        let hi = (self.0 as i32 >> 25) << 5;
        let lo = (self.0 >> 7) & 0x1F;
        (hi as u32) | lo
    }

    // Imediato Tipo-B (offset de branch com extensão de sinal, bit 0 é sempre 0)
    pub fn imm_b(&self) -> u32 {
        let sign = ((self.0 as i32 >> 31) as u32) & 0xFFFF_F000;
        let b11 = (self.0 & 0x0000_0080) << 4;
        let b10_5 = (self.0 & 0x7E00_0000) >> 20;
        let b4_1 = (self.0 & 0x0000_0F00) >> 7;
        sign | b11 | b10_5 | b4_1
    }

    // Imediato Tipo-U (20 bits superiores com zeros nos 12 bits inferiores)
    pub fn imm_u(&self) -> u32 {
        self.0 & 0xFFFF_F000
    }

    // Imediato Tipo-J (offset de 20 bits para JAL com extensão de sinal, bit 0 é 0)
    pub fn imm_j(&self) -> u32 {
        let sign = ((self.0 as i32 >> 31) as u32) & 0xFFF0_0000;
        let b19_12 = self.0 & 0x000F_F000;
        let b11 = (self.0 & 0x0010_0000) >> 9;
        let b10_1 = (self.0 & 0x7FE0_0000) >> 20;
        sign | b19_12 | b11 | b10_1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_addi_positive() {
        let inst = Instruction(0x02a00093);
        assert_eq!(inst.opcode(), OP_IMM);
        assert_eq!(inst.rd(), 1);
        assert_eq!(inst.funct3(), 0);
        assert_eq!(inst.rs1(), 0);
        assert_eq!(inst.imm_i(), 42);
    }

    #[test]
    fn test_decode_addi_negative_sign_extension() {
        let inst = Instruction(0xff010113);
        assert_eq!(inst.opcode(), OP_IMM);
        assert_eq!(inst.rd(), 2);
        assert_eq!(inst.rs1(), 2);
        assert_eq!(inst.imm_i() as i32, -16);
        assert_eq!(inst.imm_i(), 0xFFFF_FFF0);
    }

    #[test]
    fn test_decode_store_type_s() {
        let inst = Instruction(0x00512823);
        assert_eq!(inst.opcode(), OP_STORE);
        assert_eq!(inst.funct3(), 2);
        assert_eq!(inst.rs1(), 2);
        assert_eq!(inst.rs2(), 5);
        assert_eq!(inst.imm_s(), 16);
    }

    #[test]
    fn test_decode_store_type_s_negative() {
        let inst = Instruction(0xfe512e23);
        assert_eq!(inst.opcode(), OP_STORE);
        assert_eq!(inst.imm_s() as i32, -4);
    }

    #[test]
    fn test_decode_branch_type_b() {
        let inst = Instruction(0x00208863);
        assert_eq!(inst.opcode(), OP_BRANCH);
        assert_eq!(inst.funct3(), 0);
        assert_eq!(inst.rs1(), 1);
        assert_eq!(inst.rs2(), 2);
        assert_eq!(inst.imm_b() as i32, 16);
    }

    #[test]
    fn test_decode_branch_type_b_negative() {
        let inst = Instruction(0xfe208ce3);
        assert_eq!(inst.opcode(), OP_BRANCH);
        assert_eq!(inst.imm_b() as i32, -8);
    }

    #[test]
    fn test_decode_upper_type_u() {
        let inst = Instruction(0x123450b7);
        assert_eq!(inst.opcode(), OP_LUI);
        assert_eq!(inst.rd(), 1);
        assert_eq!(inst.imm_u(), 0x1234_5000);
    }

    #[test]
    fn test_decode_jump_type_j() {
        let inst = Instruction(0x014000ef);
        assert_eq!(inst.opcode(), OP_JAL);
        assert_eq!(inst.rd(), 1);
        assert_eq!(inst.imm_j() as i32, 20);
    }
}
