use rvemu_rust::Cpu;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut cpu = Cpu::new();

    if args.len() > 1 {
        let filename = &args[1];

        if filename.ends_with(".asm") || filename.ends_with(".s") {
            println!("Montando código assembly: {}", filename);
            let source = match std::fs::read_to_string(filename) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Erro ao ler arquivo {}: {}", filename, e);
                    std::process::exit(1);
                }
            };

            if let Err(err) = cpu.load_assembly(&source) {
                eprintln!("Erro de montagem (Assembler): {:?}", err);
                std::process::exit(1);
            }
        } else {
            println!("Carregando binário direto: {}", filename);
            if let Err(e) = cpu.load_file(filename) {
                eprintln!("Erro ao abrir arquivo {}: {}", filename, e);
                std::process::exit(1);
            }
        }

        println!("Executando programa...");
        let mut cycles: u64 = 0;
        const MAX_CYCLES: u64 = 10_000_000;

        while !cpu.is_halted && cycles < MAX_CYCLES {
            if let Err(err) = cpu.step() {
                eprintln!("Erro de execução no ciclo {} (PC = 0x{:08x}): {:?}", cycles, cpu.pc, err);
                break;
            }
            cycles += 1;
        }

        if let Some(code) = cpu.exit_code {
            println!("\nPrograma finalizou via ECALL exit() com código de saída: {}", code);
        } else if cpu.is_halted {
            println!("\nPrograma pausado via EBREAK!");
        }

        cpu.dump_registers();
    } else {
        // Modo Demonstração Integrada: Hello World e Fibonacci com Syscalls reais
        println!("┌────────────────────────────────────────────────────────┐");
        println!("│        ⚡ SIMULADOR RISC-V 32 (RV32I) EM RUST           │");
        println!("└────────────────────────────────────────────────────────┘");
        println!("Dica: Você também pode rodar um binário compilado: cargo run -- programa.bin\n");

        // 1. Grava a string "Hello from RISC-V syscall!\n" na DRAM no endereço 0x200
        let msg = b"Hello from RISC-V syscall!\n";
        let msg_addr: u32 = 0x200;
        cpu.bus.load(msg_addr, msg);

        // 2. Monta o programa de demonstração em memória:
        //
        // Parte 1: Imprimir mensagem via Syscall 64 (write)
        //   0x00: addi a0, zero, 1       (fd = 1 = stdout)
        //   0x04: addi a1, zero, 0x200   (buffer = msg_addr)
        //   0x08: addi a2, zero, 27      (count = tamanho da mensagem)
        //   0x0C: addi a7, zero, 64      (syscall id = 64 = write)
        //   0x10: ecall                  (executa write)
        //
        // Parte 2: Calcular Fibonacci(7) = 13
        //   0x14: addi x1, zero, 7       (n = 7)
        //   0x18: addi x2, zero, 0       (a = 0)
        //   0x1C: addi x3, zero, 1       (b = 1)
        // loop (0x20):
        //   0x20: beq  x1, zero, 24      (se n == 0, sai para 0x38)
        //   0x24: add  x5, x2, x3        (t0 = a + b)
        //   0x28: addi x2, x3, 0         (a = b)
        //   0x2C: addi x3, x5, 0         (b = t0)
        //   0x30: addi x1, x1, -1        (n--)
        //   0x34: jal  zero, -20         (volta para 0x20)
        // fim (0x38):
        //   0x38: addi a0, x2, 0         (a0 = resultado de Fibonacci)
        //   0x3C: addi a7, zero, 93      (syscall exit)
        //   0x40: ecall                  (encerra o programa!)

        let program: [u32; 17] = [
            // Parte 1: Syscall write (0x00 a 0x10)
            0x00100513, // 0x00: addi a0, zero, 1
            0x20000593, // 0x04: addi a1, zero, 0x200
            0x01b00613, // 0x08: addi a2, zero, 27
            0x04000893, // 0x0C: addi a7, zero, 64
            0x00000073, // 0x10: ecall
            // Parte 2: Fibonacci(7) (0x14 a 0x40)
            0x00700093, // 0x14: addi x1, zero, 7
            0x00000113, // 0x18: addi x2, zero, 0
            0x00100193, // 0x1C: addi x3, zero, 1
            // Loop em 0x20:
            0x00008c63, // 0x20: beq  x1, zero, 24 -> vai para 0x38
            0x003102b3, // 0x24: add  x5, x2, x3
            0x00018113, // 0x28: addi x2, x3, 0
            0x00028193, // 0x2C: addi x3, x5, 0
            0xfff08093, // 0x30: addi x1, x1, -1
            0xfedff06f, // 0x34: jal  zero, -20 -> volta para 0x20
            // Fim e Exit em 0x38:
            0x00010513, // 0x38: addi a0, x2, 0
            0x05d00893, // 0x3C: addi a7, zero, 93 (syscall exit)
            0x00000073, // 0x40: ecall
        ];

        for (i, inst) in program.iter().enumerate() {
            cpu.bus.write32((i * 4) as u32, *inst).unwrap();
        }

        println!("Executando programa demonstrativo na DRAM...\n");
        let mut cycles: u64 = 0;
        const MAX_CYCLES: u64 = 1_000_000;

        while !cpu.is_halted && cycles < MAX_CYCLES {
            cpu.step().expect("Erro na execução da instrução");
            cycles += 1;
        }

        println!("\nPrograma finalizado em {} ciclos de clock!", cycles);
        if let Some(code) = cpu.exit_code {
            println!("Código de saída do programa (exit status): {}", code);
        }

        cpu.dump_registers();
    }
}
