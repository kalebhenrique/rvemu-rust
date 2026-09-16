use crate::instruction::*;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub enum AssemblerError {
    UnknownInstruction(String),
    InvalidRegister(String),
    InvalidImmediate(String),
    InvalidMemoryOperand(String),
    InvalidArguments(String),
}

pub struct Assembler;

impl Assembler {
    pub fn assemble(source: &str) -> Result<Vec<u8>, AssemblerError> {
        let words = Self::assemble_words(source)?;
        let mut bytes = Vec::with_capacity(words.len() * 4);
        for word in words {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        Ok(bytes)
    }

    pub fn assemble_words(source: &str) -> Result<Vec<u32>, AssemblerError> {
        let mut labels = HashMap::new();
        let mut parsed_lines = Vec::new();
        let mut current_pc: u32 = 0;

        // Passo 1: Limpar comentários, identificar labels e calcular endereços
        for raw_line in source.lines() {
            let line = raw_line.split('#').next().unwrap();
            let line = line.split("//").next().unwrap().trim();

            if line.is_empty() {
                continue;
            }

            let mut remaining = line;
            if let Some(colon_idx) = remaining.find(':') {
                let label = remaining[..colon_idx].trim().to_string();
                labels.insert(label, current_pc);
                remaining = remaining[colon_idx + 1..].trim();
            }

            if remaining.is_empty() {
                continue;
            }

            let tokens = Self::tokenize(remaining);
            let expanded_list = Self::expand_pseudo(&tokens)?;

            for exp in expanded_list {
                parsed_lines.push((current_pc, exp));
                current_pc += 4;
            }
        }

        // Passo 2: Codificar cada instrução com os labels resolvidos
        let mut instructions = Vec::with_capacity(parsed_lines.len());
        for (pc, tokens) in parsed_lines {
            let word = Self::encode_instruction(pc, &tokens, &labels)?;
            instructions.push(word);
        }

        Ok(instructions)
    }

    fn tokenize(line: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut parts = line.split_whitespace();
        if let Some(mnemonic) = parts.next() {
            tokens.push(mnemonic.to_lowercase());
            let rest = parts.collect::<Vec<_>>().join(" ");
            for arg in rest.split(',') {
                let trimmed = arg.trim();
                if !trimmed.is_empty() {
                    tokens.push(trimmed.to_string());
                }
            }
        }
        tokens
    }

    fn expand_pseudo(tokens: &[String]) -> Result<Vec<Vec<String>>, AssemblerError> {
        if tokens.is_empty() {
            return Ok(vec![]);
        }
        let mnemonic = tokens[0].as_str();
        match mnemonic {
            "nop" => Ok(vec![vec!["addi".into(), "zero".into(), "zero".into(), "0".into()]]),
            "mv" => {
                if tokens.len() != 3 {
                    return Err(AssemblerError::InvalidArguments("mv rd, rs".into()));
                }
                Ok(vec![vec!["addi".into(), tokens[1].clone(), tokens[2].clone(), "0".into()]])
            }
            "li" => {
                if tokens.len() != 3 {
                    return Err(AssemblerError::InvalidArguments("li rd, imm".into()));
                }
                Ok(vec![vec!["addi".into(), tokens[1].clone(), "zero".into(), tokens[2].clone()]])
            }
            "j" => {
                if tokens.len() != 2 {
                    return Err(AssemblerError::InvalidArguments("j target".into()));
                }
                Ok(vec![vec!["jal".into(), "zero".into(), tokens[1].clone()]])
            }
            "ret" => Ok(vec![vec!["jalr".into(), "zero".into(), "ra".into(), "0".into()]]),
            _ => Ok(vec![tokens.to_vec()]),
        }
    }

    fn encode_instruction(
        pc: u32,
        tokens: &[String],
        labels: &HashMap<String, u32>,
    ) -> Result<u32, AssemblerError> {
        let mnemonic = tokens[0].as_str();

        match mnemonic {
            "add" => Self::encode_r(tokens, OP_REG, 0x0, 0x00),
            "sub" => Self::encode_r(tokens, OP_REG, 0x0, 0x20),
            "sll" => Self::encode_r(tokens, OP_REG, 0x1, 0x00),
            "slt" => Self::encode_r(tokens, OP_REG, 0x2, 0x00),
            "sltu" => Self::encode_r(tokens, OP_REG, 0x3, 0x00),
            "xor" => Self::encode_r(tokens, OP_REG, 0x4, 0x00),
            "srl" => Self::encode_r(tokens, OP_REG, 0x5, 0x00),
            "sra" => Self::encode_r(tokens, OP_REG, 0x5, 0x20),
            "or" => Self::encode_r(tokens, OP_REG, 0x6, 0x00),
            "and" => Self::encode_r(tokens, OP_REG, 0x7, 0x00),

            "addi" => Self::encode_i(tokens, OP_IMM, 0x0),
            "slti" => Self::encode_i(tokens, OP_IMM, 0x2),
            "sltiu" => Self::encode_i(tokens, OP_IMM, 0x3),
            "xori" => Self::encode_i(tokens, OP_IMM, 0x4),
            "ori" => Self::encode_i(tokens, OP_IMM, 0x6),
            "andi" => Self::encode_i(tokens, OP_IMM, 0x7),
            "slli" => Self::encode_shift(tokens, 0x1, 0x00),
            "srli" => Self::encode_shift(tokens, 0x5, 0x00),
            "srai" => Self::encode_shift(tokens, 0x5, 0x20),

            "lb" => Self::encode_load(tokens, 0x0),
            "lh" => Self::encode_load(tokens, 0x1),
            "lw" => Self::encode_load(tokens, 0x2),
            "lbu" => Self::encode_load(tokens, 0x4),
            "lhu" => Self::encode_load(tokens, 0x5),

            "sb" => Self::encode_store(tokens, 0x0),
            "sh" => Self::encode_store(tokens, 0x1),
            "sw" => Self::encode_store(tokens, 0x2),

            "beq" => Self::encode_branch(pc, tokens, labels, 0x0),
            "bne" => Self::encode_branch(pc, tokens, labels, 0x1),
            "blt" => Self::encode_branch(pc, tokens, labels, 0x4),
            "bge" => Self::encode_branch(pc, tokens, labels, 0x5),
            "bltu" => Self::encode_branch(pc, tokens, labels, 0x6),
            "bgeu" => Self::encode_branch(pc, tokens, labels, 0x7),

            "jal" => Self::encode_jal(pc, tokens, labels),
            "jalr" => Self::encode_jalr(tokens),

            "lui" => Self::encode_u(tokens, OP_LUI),
            "auipc" => Self::encode_u(tokens, OP_AUIPC),

            "ecall" => Ok(0x0000_0073),
            "ebreak" => Ok(0x0010_0073),
            "fence" => Ok(0x0000_000F),

            _ => Err(AssemblerError::UnknownInstruction(mnemonic.to_string())),
        }
    }

    fn encode_r(tokens: &[String], op: u32, funct3: u32, funct7: u32) -> Result<u32, AssemblerError> {
        if tokens.len() != 4 {
            return Err(AssemblerError::InvalidArguments(format!("Uso: {} rd, rs1, rs2", tokens[0])));
        }
        let rd = Self::parse_reg(&tokens[1])?;
        let rs1 = Self::parse_reg(&tokens[2])?;
        let rs2 = Self::parse_reg(&tokens[3])?;
        Ok((funct7 << 25) | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | op)
    }

    fn encode_i(tokens: &[String], op: u32, funct3: u32) -> Result<u32, AssemblerError> {
        if tokens.len() != 4 {
            return Err(AssemblerError::InvalidArguments(format!("Uso: {} rd, rs1, imm", tokens[0])));
        }
        let rd = Self::parse_reg(&tokens[1])?;
        let rs1 = Self::parse_reg(&tokens[2])?;
        let imm = Self::parse_immediate(&tokens[3])?;
        let imm12 = (imm as u32) & 0xFFF;
        Ok((imm12 << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | op)
    }

    fn encode_shift(tokens: &[String], funct3: u32, funct7: u32) -> Result<u32, AssemblerError> {
        if tokens.len() != 4 {
            return Err(AssemblerError::InvalidArguments(format!("Uso: {} rd, rs1, shamt", tokens[0])));
        }
        let rd = Self::parse_reg(&tokens[1])?;
        let rs1 = Self::parse_reg(&tokens[2])?;
        let shamt = (Self::parse_immediate(&tokens[3])? as u32) & 0x1F;
        Ok((funct7 << 25) | (shamt << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | OP_IMM)
    }

    fn encode_load(tokens: &[String], funct3: u32) -> Result<u32, AssemblerError> {
        if tokens.len() != 3 {
            return Err(AssemblerError::InvalidArguments(format!("Uso: {} rd, offset(rs1)", tokens[0])));
        }
        let rd = Self::parse_reg(&tokens[1])?;
        let (offset, rs1) = Self::parse_mem_operand(&tokens[2])?;
        let imm12 = (offset as u32) & 0xFFF;
        Ok((imm12 << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | OP_LOAD)
    }

    fn encode_store(tokens: &[String], funct3: u32) -> Result<u32, AssemblerError> {
        if tokens.len() != 3 {
            return Err(AssemblerError::InvalidArguments(format!("Uso: {} rs2, offset(rs1)", tokens[0])));
        }
        let rs2 = Self::parse_reg(&tokens[1])?;
        let (offset, rs1) = Self::parse_mem_operand(&tokens[2])?;
        let imm = offset as u32;
        let imm11_5 = (imm >> 5) & 0x7F;
        let imm4_0 = imm & 0x1F;
        Ok((imm11_5 << 25) | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | (imm4_0 << 7) | OP_STORE)
    }

    fn encode_branch(
        pc: u32,
        tokens: &[String],
        labels: &HashMap<String, u32>,
        funct3: u32,
    ) -> Result<u32, AssemblerError> {
        if tokens.len() != 4 {
            return Err(AssemblerError::InvalidArguments(format!("Uso: {} rs1, rs2, label", tokens[0])));
        }
        let rs1 = Self::parse_reg(&tokens[1])?;
        let rs2 = Self::parse_reg(&tokens[2])?;
        let offset = Self::resolve_offset(pc, &tokens[3], labels)?;

        let imm = offset as u32;
        let imm12 = (imm >> 12) & 1;
        let imm10_5 = (imm >> 5) & 0x3F;
        let imm4_1 = (imm >> 1) & 0xF;
        let imm11 = (imm >> 11) & 1;

        Ok((imm12 << 31)
            | (imm10_5 << 25)
            | ((rs2 as u32) << 20)
            | ((rs1 as u32) << 15)
            | (funct3 << 12)
            | (imm4_1 << 8)
            | (imm11 << 7)
            | OP_BRANCH)
    }

    fn encode_jal(
        pc: u32,
        tokens: &[String],
        labels: &HashMap<String, u32>,
    ) -> Result<u32, AssemblerError> {
        let (rd, target_str) = if tokens.len() == 3 {
            (Self::parse_reg(&tokens[1])?, &tokens[2])
        } else if tokens.len() == 2 {
            (1 /* ra */, &tokens[1])
        } else {
            return Err(AssemblerError::InvalidArguments("Uso: jal rd, target".into()));
        };

        let offset = Self::resolve_offset(pc, target_str, labels)?;
        let imm = offset as u32;
        let imm20 = (imm >> 20) & 1;
        let imm10_1 = (imm >> 1) & 0x3FF;
        let imm11 = (imm >> 11) & 1;
        let imm19_12 = (imm >> 12) & 0xFF;

        Ok((imm20 << 31)
            | (imm10_1 << 21)
            | (imm11 << 20)
            | (imm19_12 << 12)
            | ((rd as u32) << 7)
            | OP_JAL)
    }

    fn encode_jalr(tokens: &[String]) -> Result<u32, AssemblerError> {
        if tokens.len() == 4 {
            let rd = Self::parse_reg(&tokens[1])?;
            let rs1 = Self::parse_reg(&tokens[2])?;
            let imm = Self::parse_immediate(&tokens[3])?;
            let imm12 = (imm as u32) & 0xFFF;
            Ok((imm12 << 20) | ((rs1 as u32) << 15) | ((rd as u32) << 7) | OP_JALR)
        } else if tokens.len() == 3 {
            let rd = Self::parse_reg(&tokens[1])?;
            let (offset, rs1) = Self::parse_mem_operand(&tokens[2])?;
            let imm12 = (offset as u32) & 0xFFF;
            Ok((imm12 << 20) | ((rs1 as u32) << 15) | ((rd as u32) << 7) | OP_JALR)
        } else {
            Err(AssemblerError::InvalidArguments("Uso: jalr rd, rs1, offset".into()))
        }
    }

    fn encode_u(tokens: &[String], op: u32) -> Result<u32, AssemblerError> {
        if tokens.len() != 3 {
            return Err(AssemblerError::InvalidArguments(format!("Uso: {} rd, imm", tokens[0])));
        }
        let rd = Self::parse_reg(&tokens[1])?;
        let imm = Self::parse_immediate(&tokens[2])? as u32;
        let imm_val = if imm > 0xFFFFF { imm & 0xFFFF_F000 } else { (imm & 0xF_FFFF) << 12 };
        Ok(imm_val | ((rd as u32) << 7) | op)
    }

    fn resolve_offset(pc: u32, target: &str, labels: &HashMap<String, u32>) -> Result<i32, AssemblerError> {
        if let Some(&target_pc) = labels.get(target) {
            Ok((target_pc as i64 - pc as i64) as i32)
        } else {
            Self::parse_immediate(target)
        }
    }

    fn parse_mem_operand(s: &str) -> Result<(i32, usize), AssemblerError> {
        let open_idx = s.find('(').ok_or_else(|| AssemblerError::InvalidMemoryOperand(s.into()))?;
        let close_idx = s.find(')').ok_or_else(|| AssemblerError::InvalidMemoryOperand(s.into()))?;

        let offset_str = s[..open_idx].trim();
        let reg_str = s[open_idx + 1..close_idx].trim();

        let offset = if offset_str.is_empty() { 0 } else { Self::parse_immediate(offset_str)? };
        let reg = Self::parse_reg(reg_str)?;
        Ok((offset, reg))
    }

    pub fn parse_reg(name: &str) -> Result<usize, AssemblerError> {
        let clean = name.trim().to_lowercase();
        if clean.starts_with('x') {
            if let Ok(num) = clean[1..].parse::<usize>() {
                if num < 32 {
                    return Ok(num);
                }
            }
        }
        match clean.as_str() {
            "zero" => Ok(0),
            "ra" => Ok(1),
            "sp" => Ok(2),
            "gp" => Ok(3),
            "tp" => Ok(4),
            "t0" => Ok(5),
            "t1" => Ok(6),
            "t2" => Ok(7),
            "s0" | "fp" => Ok(8),
            "s1" => Ok(9),
            "a0" => Ok(10),
            "a1" => Ok(11),
            "a2" => Ok(12),
            "a3" => Ok(13),
            "a4" => Ok(14),
            "a5" => Ok(15),
            "a6" => Ok(16),
            "a7" => Ok(17),
            "s2" => Ok(18),
            "s3" => Ok(19),
            "s4" => Ok(20),
            "s5" => Ok(21),
            "s6" => Ok(22),
            "s7" => Ok(23),
            "s8" => Ok(24),
            "s9" => Ok(25),
            "s10" => Ok(26),
            "s11" => Ok(27),
            "t3" => Ok(28),
            "t4" => Ok(29),
            "t5" => Ok(30),
            "t6" => Ok(31),
            _ => Err(AssemblerError::InvalidRegister(name.into())),
        }
    }

    pub fn parse_immediate(s: &str) -> Result<i32, AssemblerError> {
        let clean = s.trim();
        let (sign, num_str) = if clean.starts_with('-') {
            (-1, &clean[1..])
        } else if clean.starts_with('+') {
            (1, &clean[1..])
        } else {
            (1, clean)
        };

        let val = if num_str.starts_with("0x") || num_str.starts_with("0X") {
            i64::from_str_radix(&num_str[2..], 16)
        } else if num_str.starts_with("0b") || num_str.starts_with("0B") {
            i64::from_str_radix(&num_str[2..], 2)
        } else {
            num_str.parse::<i64>()
        };

        match val {
            Ok(v) => Ok((v * sign) as i32),
            Err(_) => Err(AssemblerError::InvalidImmediate(s.into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assemble_simple_instructions() {
        let words = Assembler::assemble_words("addi x1, x0, 42").unwrap();
        assert_eq!(words, vec![0x02a00093]);

        let words = Assembler::assemble_words("add x3, x1, x2").unwrap();
        assert_eq!(words, vec![0x002081b3]);
    }

    #[test]
    fn test_assemble_abi_and_case_insensitive() {
        let words = Assembler::assemble_words("ADDI a0, zero, 10").unwrap();
        assert_eq!(words, vec![0x00a00513]);
    }

    #[test]
    fn test_assemble_load_and_store() {
        let words = Assembler::assemble_words("sw ra, 16(sp)").unwrap();
        assert_eq!(words, vec![0x00112823]);

        let words = Assembler::assemble_words("lw a0, 0x100(zero)").unwrap();
        assert_eq!(words, vec![0x10002503]);
    }

    #[test]
    fn test_assemble_labels_and_branches() {
        let source = r#"
            addi x1, zero, 5
        loop:
            beq x1, zero, fim
            addi x1, x1, -1
            jal zero, loop
        fim:
            ecall
        "#;
        let words = Assembler::assemble_words(source).unwrap();
        assert_eq!(words.len(), 5);
        // beq em 0x04 pulando para fim em 0x10 (offset = 12)
        assert_eq!(words[1], 0x00008663);
        // jal em 0x0C pulando para loop em 0x04 (offset = -8)
        assert_eq!(words[3], 0xff9ff06f);
    }

    #[test]
    fn test_assemble_fibonacci_full() {
        let source = r#"
            addi x1, zero, 7
            addi x2, zero, 0
            addi x3, zero, 1
        loop:
            beq  x1, zero, fim
            add  x5, x2, x3
            addi x2, x3, 0
            addi x3, x5, 0
            addi x1, x1, -1
            jal  zero, loop
        fim:
            addi a0, x2, 0
            addi a7, zero, 93
            ecall
        "#;
        let words = Assembler::assemble_words(source).unwrap();
        assert_eq!(words.len(), 12);
    }
}
