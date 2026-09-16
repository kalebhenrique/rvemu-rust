use rvemu_rust::Cpu;

fn main() {
    println!("Iniciando simulador RV32I (rvemu-rust)...");

    let mut cpu = Cpu::new();

    // Demonstração: gravando valores nos registradores
    cpu.write_reg(1, 0x1234_5678); // x1 (ra)
    cpu.write_reg(2, 0x7FFF_FFFF); // x2 (sp)
    cpu.write_reg(10, 42); // x10 (a0 - retorno de função / argumento)

    // Tentativa de escrita no x0 (deve ser ignorada)
    cpu.write_reg(0, 0xDEAD_BEEF);

    // Exibe a tabela de registradores
    cpu.dump_registers();
}
