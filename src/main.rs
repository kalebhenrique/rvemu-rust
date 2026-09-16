use rvemu_rust::Cpu;

fn main() {
    println!("Iniciando simulador RV32I (rvemu-rust)...");

    let mut cpu = Cpu::new();

    // 1. Registradores
    cpu.write_reg(1, 0x1234_5678); // x1 (ra)
    cpu.write_reg(2, 0x7FFF_FFFF); // x2 (sp)
    cpu.write_reg(10, 42); // x10 (a0)
    cpu.write_reg(0, 0xDEAD_BEEF); // x0 (permanece zero)

    // 2. Memória DRAM via Barramento (Bus)
    let addr = 0x1000;
    let word_val = 0x1234_5678;
    cpu.bus
        .write32(addr, word_val)
        .expect("Falha ao gravar na memória");

    println!("\nGravado 0x{:08x} no endereço 0x{:04x}", word_val, addr);
    println!("Verificando bytes individuais na memória (Little-Endian):");
    println!(
        "  [0x{:04x}] = 0x{:02x} (LSB)",
        addr,
        cpu.bus.read8(addr).unwrap()
    );
    println!(
        "  [0x{:04x}] = 0x{:02x}",
        addr + 1,
        cpu.bus.read8(addr + 1).unwrap()
    );
    println!(
        "  [0x{:04x}] = 0x{:02x}",
        addr + 2,
        cpu.bus.read8(addr + 2).unwrap()
    );
    println!(
        "  [0x{:04x}] = 0x{:02x} (MSB)",
        addr + 3,
        cpu.bus.read8(addr + 3).unwrap()
    );

    let read_back = cpu.bus.read32(addr).expect("Falha ao ler da memória");
    println!(
        "Lido de volta como palavra de 32 bits: 0x{:08x}\n",
        read_back
    );

    cpu.dump_registers();
}
