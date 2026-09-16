use crate::bus::Bus;
use crate::dram::MemoryError;
use crate::instruction::{Instruction, OP_AUIPC, OP_IMM, OP_LUI, OP_REG};

pub const REGISTERS_COUNT: usize = 32;

pub const ABI_REG_NAMES: [&str; REGISTERS_COUNT] = [
    "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0", "a1", "a2", "a3", "a4",
    "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11", "t3", "t4",
    "t5", "t6",
];

#[derive(Debug, PartialEq, Eq)]
pub enum CpuError {
    MemoryError(MemoryError),
    IllegalInstruction(u32),
}

impl From<MemoryError> for CpuError {
    fn from(e: MemoryError) -> Self {
        CpuError::MemoryError(e)
    }
}

#[derive(Debug, Clone)]
pub struct Cpu {
    pub regs: [u32; REGISTERS_COUNT],
    pub pc: u32,
    pub bus: Bus,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            regs: [0; REGISTERS_COUNT],
            pc: 0,
            bus: Bus::new(),
        }
    }

    pub fn with_bus(bus: Bus) -> Self {
        Self {
            regs: [0; REGISTERS_COUNT],
            pc: 0,
            bus,
        }
    }

    // Busca a instrução de 32 bits apontada pelo PC
    pub fn fetch(&self) -> Result<Instruction, MemoryError> {
        let word = self.bus.read32(self.pc)?;
        Ok(Instruction(word))
    }

    pub fn read_reg(&self, reg: usize) -> u32 {
        if reg == 0 || reg >= REGISTERS_COUNT {
            0
        } else {
            self.regs[reg]
        }
    }

    pub fn write_reg(&mut self, reg: usize, val: u32) {
        // x0 é hardwired para zero no RISC-V
        if reg != 0 && reg < REGISTERS_COUNT {
            self.regs[reg] = val;
        }
    }

    // Executa uma instrução decodificada
    pub fn execute(&mut self, inst: Instruction) -> Result<(), CpuError> {
        let opcode = inst.opcode();
        let rd = inst.rd();
        let rs1_val = self.read_reg(inst.rs1());
        let rs2_val = self.read_reg(inst.rs2());

        match opcode {
            // Instruções Imediatas (ADDI, SLTI, SLTIU, XORI, ORI, ANDI, SLLI, SRLI, SRAI)
            OP_IMM => {
                let imm = inst.imm_i();
                let shamt = imm & 0x1F;

                let result = match inst.funct3() {
                    0x0 => rs1_val.wrapping_add(imm), // ADDI
                    0x1 => {
                        if inst.funct7() == 0x00 {
                            rs1_val << shamt // SLLI
                        } else {
                            return Err(CpuError::IllegalInstruction(inst.0));
                        }
                    }
                    0x2 => {
                        if (rs1_val as i32) < (imm as i32) {
                            1
                        } else {
                            0
                        }
                    } // SLTI
                    0x3 => {
                        if rs1_val < imm {
                            1
                        } else {
                            0
                        }
                    } // SLTIU
                    0x4 => rs1_val ^ imm, // XORI
                    0x5 => match inst.funct7() {
                        0x00 => rs1_val >> shamt,                   // SRLI
                        0x20 => ((rs1_val as i32) >> shamt) as u32, // SRAI
                        _ => return Err(CpuError::IllegalInstruction(inst.0)),
                    },
                    0x6 => rs1_val | imm, // ORI
                    0x7 => rs1_val & imm, // ANDI
                    _ => return Err(CpuError::IllegalInstruction(inst.0)),
                };

                self.write_reg(rd, result);
                self.pc = self.pc.wrapping_add(4);
            }

            // Instruções Registrador-Registrador (ADD, SUB, SLL, SLT, SLTU, XOR, SRL, SRA, OR, AND)
            OP_REG => {
                let shamt = rs2_val & 0x1F;

                let result = match (inst.funct3(), inst.funct7()) {
                    (0x0, 0x00) => rs1_val.wrapping_add(rs2_val), // ADD
                    (0x0, 0x20) => rs1_val.wrapping_sub(rs2_val), // SUB
                    (0x1, 0x00) => rs1_val << shamt,              // SLL
                    (0x2, 0x00) => {
                        if (rs1_val as i32) < (rs2_val as i32) {
                            1
                        } else {
                            0
                        }
                    } // SLT
                    (0x3, 0x00) => {
                        if rs1_val < rs2_val {
                            1
                        } else {
                            0
                        }
                    } // SLTU
                    (0x4, 0x00) => rs1_val ^ rs2_val,             // XOR
                    (0x5, 0x00) => rs1_val >> shamt,              // SRL
                    (0x5, 0x20) => ((rs1_val as i32) >> shamt) as u32, // SRA
                    (0x6, 0x00) => rs1_val | rs2_val,             // OR
                    (0x7, 0x00) => rs1_val & rs2_val,             // AND
                    _ => return Err(CpuError::IllegalInstruction(inst.0)),
                };

                self.write_reg(rd, result);
                self.pc = self.pc.wrapping_add(4);
            }

            // LUI (Load Upper Immediate)
            OP_LUI => {
                self.write_reg(rd, inst.imm_u());
                self.pc = self.pc.wrapping_add(4);
            }

            // AUIPC (Add Upper Immediate to PC)
            OP_AUIPC => {
                self.write_reg(rd, self.pc.wrapping_add(inst.imm_u()));
                self.pc = self.pc.wrapping_add(4);
            }

            _ => return Err(CpuError::IllegalInstruction(inst.0)),
        }

        Ok(())
    }

    // Executa um ciclo completo: busca a instrução e executa
    pub fn step(&mut self) -> Result<(), CpuError> {
        let inst = self.fetch()?;
        self.execute(inst)
    }

    pub fn dump_registers(&self) {
        println!(
            "┌─────────────────────────────────────────────────────────────────────────────────────────────────────┐"
        );
        println!(
            "│                    ESTADO DOS REGISTRADORES RV32                                                    │"
        );
        println!(
            "├─────────────────────────────────────────────────────────────────────────────────────────────────────┤"
        );
        println!(
            "│  PC: 0x{:08x}                                                                                     │",
            self.pc
        );
        println!(
            "├─────────────────────────────────────────────────────────────────────────────────────────────────────┤"
        );
        for i in (0..REGISTERS_COUNT).step_by(4) {
            println!(
                "│  x{:02} ({:<4}): 0x{:08x} │ x{:02} ({:<4}): 0x{:08x} │ x{:02} ({:<4}): 0x{:08x} │ x{:02} ({:<4}): 0x{:08x}  │",
                i,
                ABI_REG_NAMES[i],
                self.read_reg(i),
                i + 1,
                ABI_REG_NAMES[i + 1],
                self.read_reg(i + 1),
                i + 2,
                ABI_REG_NAMES[i + 2],
                self.read_reg(i + 2),
                i + 3,
                ABI_REG_NAMES[i + 3],
                self.read_reg(i + 3),
            );
        }
        println!(
            "└─────────────────────────────────────────────────────────────────────────────────────────────────────┘"
        );
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_initialization() {
        let cpu = Cpu::new();
        assert_eq!(cpu.pc, 0);
        for i in 0..REGISTERS_COUNT {
            assert_eq!(cpu.read_reg(i), 0);
        }
    }

    #[test]
    fn test_write_and_read_general_registers() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0x1234_5678);
        cpu.write_reg(2, 0xCAFE_BABE);
        cpu.write_reg(31, 0xDEAD_BEEF);

        assert_eq!(cpu.read_reg(1), 0x1234_5678);
        assert_eq!(cpu.read_reg(2), 0xCAFE_BABE);
        assert_eq!(cpu.read_reg(31), 0xDEAD_BEEF);
    }

    #[test]
    fn test_x0_is_hardwired_to_zero() {
        let mut cpu = Cpu::new();
        cpu.write_reg(0, 0xFFFF_FFFF);

        assert_eq!(cpu.read_reg(0), 0);
        assert_eq!(cpu.regs[0], 0);
    }

    #[test]
    fn test_out_of_bounds_handling() {
        let mut cpu = Cpu::new();
        cpu.write_reg(32, 0x1234);
        assert_eq!(cpu.read_reg(32), 0);
    }

    #[test]
    fn test_cpu_access_memory_through_bus() {
        let mut cpu = Cpu::new();
        cpu.bus.write32(0x10, 0xCAFE_BABE).unwrap();
        assert_eq!(cpu.bus.read32(0x10).unwrap(), 0xCAFE_BABE);
    }

    #[test]
    fn test_cpu_fetch() {
        let mut cpu = Cpu::new();
        cpu.bus.write32(0x0, 0x02a00093).unwrap();
        cpu.pc = 0x0;

        let inst = cpu.fetch().unwrap();
        assert_eq!(inst.0, 0x02a00093);
        assert_eq!(inst.opcode(), crate::instruction::OP_IMM);
        assert_eq!(inst.rd(), 1);
        assert_eq!(inst.imm_i(), 42);
    }

    #[test]
    fn test_execute_addi_and_pc_increment() {
        let mut cpu = Cpu::new();
        // addi x1, x0, 15 (0x00f00093)
        cpu.execute(Instruction(0x00f00093)).unwrap();
        assert_eq!(cpu.read_reg(1), 15);
        assert_eq!(cpu.pc, 4);

        // addi x1, x1, -5 (0xffb08093)
        cpu.execute(Instruction(0xffb08093)).unwrap();
        assert_eq!(cpu.read_reg(1), 10);
        assert_eq!(cpu.pc, 8);
    }

    #[test]
    fn test_execute_wrapping_arithmetic() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0xFFFF_FFFF); // -1
        // addi x2, x1, 1 -> deve dar wrap para 0 sem panic!
        // addi x2, x1, 1: imm=1, rs1=1, funct3=0, rd=2, opcode=0x13 -> 0x00108113
        cpu.execute(Instruction(0x00108113)).unwrap();
        assert_eq!(cpu.read_reg(2), 0);
    }

    #[test]
    fn test_execute_add_and_sub() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 20);
        cpu.write_reg(2, 8);

        // add x3, x1, x2 (funct7=0, rs2=2, rs1=1, funct3=0, rd=3, opcode=0x33) -> 0x002081b3
        cpu.execute(Instruction(0x002081b3)).unwrap();
        assert_eq!(cpu.read_reg(3), 28);

        // sub x4, x1, x2 (funct7=0x20, rs2=2, rs1=1, funct3=0, rd=4, opcode=0x33) -> 0x40208233
        cpu.execute(Instruction(0x40208233)).unwrap();
        assert_eq!(cpu.read_reg(4), 12);
    }

    #[test]
    fn test_execute_logical_and_shifts() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0b1100);
        cpu.write_reg(2, 0b1010);

        // and x3, x1, x2 -> 0b1000 = 8 (funct7=0, rs2=2, rs1=1, funct3=7, rd=3, opcode=0x33) -> 0x0020f1b3
        cpu.execute(Instruction(0x0020f1b3)).unwrap();
        assert_eq!(cpu.read_reg(3), 0b1000);

        // or x4, x1, x2 -> 0b1110 = 14 (funct7=0, funct3=6, rd=4) -> 0x0020e233
        cpu.execute(Instruction(0x0020e233)).unwrap();
        assert_eq!(cpu.read_reg(4), 0b1110);

        // xor x5, x1, x2 -> 0b0110 = 6 (funct7=0, funct3=4, rd=5) -> 0x0020c2b3
        cpu.execute(Instruction(0x0020c2b3)).unwrap();
        assert_eq!(cpu.read_reg(5), 0b0110);
    }

    #[test]
    fn test_execute_slt_signed_vs_unsigned() {
        let mut cpu = Cpu::new();
        cpu.write_reg(1, 0xFFFF_FFF6); // -10 em complemento de 2
        cpu.write_reg(2, 5);

        // slt x3, x1, x2 (com sinal: -10 < 5 é TRUE -> 1)
        // funct7=0, funct3=2, rs2=2, rs1=1, rd=3, op=0x33 -> 0x0020a1b3
        cpu.execute(Instruction(0x0020a1b3)).unwrap();
        assert_eq!(cpu.read_reg(3), 1);

        // sltu x4, x1, x2 (sem sinal: 0xFFFF_FFF6 < 5 é FALSE -> 0)
        // funct7=0, funct3=3, rs2=2, rs1=1, rd=4, op=0x33 -> 0x0020b233
        cpu.execute(Instruction(0x0020b233)).unwrap();
        assert_eq!(cpu.read_reg(4), 0);
    }

    #[test]
    fn test_execute_lui_and_auipc() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x100;

        // lui x1, 0x12345 -> 0x123450b7
        cpu.execute(Instruction(0x123450b7)).unwrap();
        assert_eq!(cpu.read_reg(1), 0x1234_5000);

        // auipc x2, 0x01000 (com pc atual = 0x104) -> 0x01000117
        cpu.execute(Instruction(0x01000117)).unwrap();
        assert_eq!(cpu.read_reg(2), 0x0100_0000 + 0x104);
    }

    #[test]
    fn test_cpu_step_sequence() {
        let mut cpu = Cpu::new();
        // Grava duas instruções na DRAM:
        // 0x0: addi x1, x0, 10 (0x00a00093)
        // 0x4: addi x2, x1, 20 (0x01408113)
        cpu.bus.write32(0x0, 0x00a00093).unwrap();
        cpu.bus.write32(0x4, 0x01408113).unwrap();

        cpu.step().unwrap();
        assert_eq!(cpu.read_reg(1), 10);
        assert_eq!(cpu.pc, 4);

        cpu.step().unwrap();
        assert_eq!(cpu.read_reg(2), 30);
        assert_eq!(cpu.pc, 8);
    }
}
