use rvemu_rust::Cpu;

fn main() {
    println!("Iniciando simulador RV32I (rvemu-rust)...");

    let mut cpu = Cpu::new();

    // Programa simulado em Assembly RV32:
    // 1. addi x1, x0, 10       -> x1 = 10
    // 2. addi x2, x0, 25       -> x2 = 25
    // 3. add  x3, x1, x2       -> x3 = 35
    // 4. sub  x4, x3, x1       -> x4 = 25
    // 5. slli x5, x1, 2        -> x5 = 10 << 2 = 40
    // 6. lui  x6, 0x12345      -> x6 = 0x12345000
    cpu.bus.write32(0x0, 0x00a00093).unwrap(); // addi x1, x0, 10
    cpu.bus.write32(0x4, 0x01900113).unwrap(); // addi x2, x0, 25
    cpu.bus.write32(0x8, 0x002081b3).unwrap(); // add  x3, x1, x2
    cpu.bus.write32(0xc, 0x40118233).unwrap(); // sub  x4, x3, x1
    cpu.bus.write32(0x10, 0x00209293).unwrap(); // slli x5, x1, 2
    cpu.bus.write32(0x14, 0x12345337).unwrap(); // lui  x6, 0x12345

    println!("Executando 6 instruções...");
    for _ in 0..6 {
        cpu.step().expect("Erro na execução da instrução");
    }

    println!("\nPrograma finalizado com sucesso!");
    cpu.dump_registers();
}
