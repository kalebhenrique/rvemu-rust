# rvemu-rust

Um emulador da arquitetura **RISC-V de 32 bits (RV32I)** escrito em **Rust**.

Este projeto tem como objetivo consolidar conceitos de **Organização e Arquitetura de Computadores (OAC)** — como banco de registradores, decodificação de instruções, manipulação de memória e _endianness_ — ao mesmo tempo em que serve como projeto de aprendizado prático da linguagem **Rust** e de programação de baixo nível.

---

## Objetivo

Implementar a especificação base **RV32I (Unprivileged ISA)**, permitindo:

- Executar instruções aritméticas, lógicas, de deslocamento e controle de fluxo.
- Gerenciar leituras e escritas na memória física DRAM respeitando o formato **Little-Endian**.
- Carregar e executar pequenos programas compilados a partir de assembly ou C para RISC-V.
- Validar a precisão da emulação através de testes unitários e testes de integração.

---

## Arquitetura do Emulador

### Principais Componentes

1. **CPU (`Cpu`)**: Mantém o estado da máquina (os 32 registradores de 32 bits e o `PC`). Garante a regra fundamental do RISC-V onde o registrador `x0` é fixo em zero.
2. **Barramento (`Bus`)**: Responsável por encaminhar leituras e escritas do processador para a memória ou eventuais periféricos.
3. **Memória (`Dram`)**: Array de bytes (`Vec<u8>`) com operações de leitura e escrita para `u8`, `u16` e `u32` com convenção _Little-Endian_.
4. **Decodificador (`Instruction`)**: Extrai os campos (`opcode`, `rd`, `funct3`, `rs1`, `rs2`, `funct7`) e imediatos com a devida extensão de sinal para os 6 formatos de instrução: **R, I, S, B, U e J**.

---

## Roadmap de Desenvolvimento (Milestones)

- [x] **Milestone 0: Setup & Estruturas Base**
  - [x] Definição da `struct Cpu` (`regs: [u32; 32]`, `pc: u32`).
  - [x] Métodos auxiliares de leitura e escrita em registradores com salvaguarda de `x0`.
- [x] **Milestone 1: Memória & Barramento**
  - [x] Estrutura `Dram` baseada em `Vec<u8>`.
  - [x] Operações de `read8/16/32` e `write8/16/32` (Little-Endian).
  - [x] Tratamento de acessos fora de limites (*OutOfBounds*).
- [x] **Milestone 2: Fetch & Decode**
  - [x] Leitura da instrução de 32 bits apontada pelo `PC`.
  - [x] Decodificação de campos e extensão de sinal para todos os formatos de imediatos.
- [x] **Milestone 3: Instruções Aritméticas e Lógicas (Execute - Parte 1)**
  - [x] Instruções com imediatos (`ADDI`, `SLTI`, `XORI`, `ORI`, `ANDI`, `SLLI`, `SRLI`, `SRAI`).
  - [x] Instruções registrador-registrador (`ADD`, `SUB`, `SLL`, `SLT`, `XOR`, `SRL`, `SRA`, `OR`, `AND`).
  - [x] Aritmética com `wrapping_*` para prevenção de panic por overflow no Rust.
- [ ] **Milestone 4: Saltos, Condicionais e Memória (Execute - Parte 2)**
  - [ ] Branches condicionais (`BEQ`, `BNE`, `BLT`, `BGE`, `BLTU`, `BGEU`).
  - [ ] Saltos incondicionais (`JAL`, `JALR`).
  - [ ] Instruções de Load/Store (`LB`, `LH`, `LW`, `SB`, `SH`, `SW`).
  - [ ] Imediatos superiores (`LUI`, `AUIPC`).
- [ ] **Milestone 5: Sistema, Carregamento de Binários & Testes**
  - [ ] Suporte básico a `ECALL` e `EBREAK`.
  - [ ] Leitor de arquivos binários planos (`.bin`).
  - [ ] Bateria de testes automatizados com pequenos programas assembly (Fibonacci, somatório).

---

## Estrutura de Arquivos Planejada

```text
rvemu-rust/
├── Cargo.toml          # Configuração do projeto e dependências Rust
├── README.md           # Documentação do projeto
└── src/
    ├── main.rs         # Ponto de entrada (CLI/REPL do simulador)
    ├── cpu.rs          # Estado do processador e ciclo de execução
    ├── bus.rs          # Barramento de interconexão
    ├── dram.rs         # Simulação da memória física DRAM
    ├── instruction.rs  # Decodificação de opcodes e imediatos
    └── trap.rs         # Definição de exceções/interrupções (Traps)
```

---

## 🚀 Como Executar

### Pré-requisitos

- [Rust](https://www.rust-lang.org/) (versão estável mais recente recomendada).

### Compilação e Execução

Clone o repositório e navegue até a pasta:

```bash
cargo build
```

Para rodar o emulador:

```bash
cargo run
```

Para rodar a suíte de testes unitários:

```bash
cargo test
```

---

## Referências

- [The RISC-V Instruction Set Manual - Volume I: Unprivileged ISA](https://riscv.org/technical/specifications/)
- [The Rust Programming Language Book](https://doc.rust-lang.org/book/)
- [Writing a RISC-V Emulator in Rust (Tutorial de Referência)](https://book.rvemu.app/)
