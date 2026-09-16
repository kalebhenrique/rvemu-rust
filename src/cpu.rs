pub const REGISTERS_COUNT: usize = 32;

pub const ABI_REG_NAMES: [&str; REGISTERS_COUNT] = [
    "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0", "a1", "a2", "a3", "a4",
    "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11", "t3", "t4",
    "t5", "t6",
];

#[derive(Debug, Clone)]
pub struct Cpu {
    pub regs: [u32; REGISTERS_COUNT],
    pub pc: u32,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            regs: [0; REGISTERS_COUNT],
            pc: 0,
        }
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
}
