use rvemu_rust::Cpu;

fn main() {
    println!("Iniciando simulador RV32I (rvemu-rust)...");

    let mut cpu = Cpu::new();

    // 1. Simula um programa em memória na posição 0x0:
    // addi x1, x0, 42   (0x02a00093)
    // sw   x1, 16(x2)   (0x00112823)
    cpu.bus.write32(0x0, 0x02a00093).unwrap();
    cpu.bus.write32(0x4, 0x00112823).unwrap();

    println!("\n--- Fetch & Decode da 1ª instrução (PC: 0x0000) ---");
    cpu.pc = 0x0;
    let inst1 = cpu.fetch().unwrap();
    println!("Palavra binária: 0x{:08x}", inst1.0);
    println!("Opcode : 0x{:02x} (OP_IMM)", inst1.opcode());
    println!("rd     : x{} ({})", inst1.rd(), rvemu_rust::cpu::ABI_REG_NAMES[inst1.rd()]);
    println!("funct3 : {}", inst1.funct3());
    println!("rs1    : x{} ({})", inst1.rs1(), rvemu_rust::cpu::ABI_REG_NAMES[inst1.rs1()]);
    println!("imm_i  : {} (decimal)", inst1.imm_i() as i32);

    println!("\n--- Fetch & Decode da 2ª instrução (PC: 0x0004) ---");
    cpu.pc = 0x4;
    let inst2 = cpu.fetch().unwrap();
    println!("Palavra binária: 0x{:08x}", inst2.0);
    println!("Opcode : 0x{:02x} (OP_STORE)", inst2.opcode());
    println!("rs1    : x{} ({})", inst2.rs1(), rvemu_rust::cpu::ABI_REG_NAMES[inst2.rs1()]);
    println!("rs2    : x{} ({})", inst2.rs2(), rvemu_rust::cpu::ABI_REG_NAMES[inst2.rs2()]);
    println!("imm_s  : {} (decimal)", inst2.imm_s() as i32);
}
