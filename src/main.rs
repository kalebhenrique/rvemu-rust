use rvemu_rust::Cpu;

fn main() {
    println!("Iniciando simulador RV32I (rvemu-rust)...");

    let mut cpu = Cpu::new();

    // Programa Assembly RV32:
    // Calcula o 6º número de Fibonacci e grava o resultado na RAM em 0x100.
    //
    // x1 = n (6)
    // x2 = a (0)
    // x3 = b (1)
    //
    // loop (0x0c):
    //   beq  x1, x0, 24   (se x1 == 0, pula 24 bytes para 0x24 -> fim)
    //   add  x5, x2, x3   (x5/t0 = a + b)
    //   addi x2, x3, 0    (a = b)
    //   addi x3, x5, 0    (b = t0)
    //   addi x1, x1, -1   (n -= 1)
    //   jal  x0, -20      (pula -20 bytes de volta para 0x0c)
    //
    // fim (0x24):
    //   sw   x2, 0x100(x0) (grava o resultado 'a' no endereço 0x100 da RAM)

    cpu.bus.write32(0x00, 0x00600093).unwrap(); // addi x1, x0, 6
    cpu.bus.write32(0x04, 0x00000113).unwrap(); // addi x2, x0, 0
    cpu.bus.write32(0x08, 0x00100193).unwrap(); // addi x3, x0, 1
    // Loop em 0x0C
    cpu.bus.write32(0x0C, 0x00008c63).unwrap(); // beq  x1, x0, 24 -> fim (0x24)
    cpu.bus.write32(0x10, 0x003102b3).unwrap(); // add  x5, x2, x3
    cpu.bus.write32(0x14, 0x00018113).unwrap(); // addi x2, x3, 0
    cpu.bus.write32(0x18, 0x00028193).unwrap(); // addi x3, x5, 0
    cpu.bus.write32(0x1C, 0xfff08093).unwrap(); // addi x1, x1, -1
    cpu.bus.write32(0x20, 0xfedff06f).unwrap(); // jal  x0, -20 -> volta para 0x0C
    // Fim em 0x24
    cpu.bus.write32(0x24, 0x10202023).unwrap(); // sw   x2, 0x100(x0)

    println!("Executando programa de Fibonacci em RV32...");

    // Executa até passar da instrução 'sw' em 0x24 (PC == 0x28)
    let mut cycles = 0;
    while cpu.pc != 0x28 {
        cpu.step().expect("Erro na execução da instrução");
        cycles += 1;
    }

    println!("\nPrograma concluído em {} ciclos de clock!", cycles);
    println!("Valor salvo na memória DRAM no endereço 0x100: {}", cpu.bus.read32(0x100).unwrap());

    cpu.dump_registers();
}
