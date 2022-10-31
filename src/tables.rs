use crate::codes::Arg;
use crate::{rem_spaces, Args, Syntax};
use std::collections::HashMap;

static REGS: [&str; 32] = [
    "zero", "at", "v0", "v1", "a0", "a1", "a2", "a3", "t0", "t1", "t2", "t3", "t4", "t5", "t6",
    "t7", "s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7", "t8", "t9", "k0", "k1", "gp", "sp", "fp",
    "ra",
];

pub struct InstrCode<'a> {
    pub(crate) name: &'a str,
    pub(crate) syntax: Syntax,
    pub(crate) code: i8,
}

static CODES: [InstrCode; 53] = [
    InstrCode {
        name: "null",
        syntax: Syntax::Syscall,
        code: -1,
    },
    InstrCode {
        name: "add",
        syntax: Syntax::ArithLog,
        code: 32,
    },
    InstrCode {
        name: "addu",
        syntax: Syntax::ArithLog,
        code: 33,
    },
    InstrCode {
        name: "addi",
        syntax: Syntax::ArithLogI,
        code: 8,
    },
    InstrCode {
        name: "addiu",
        syntax: Syntax::ArithLogI,
        code: 9,
    },
    InstrCode {
        name: "and",
        syntax: Syntax::ArithLog,
        code: 36,
    },
    InstrCode {
        name: "andi",
        syntax: Syntax::ArithLogI,
        code: 12,
    },
    InstrCode {
        name: "div",
        syntax: Syntax::DivMult,
        code: 26,
    },
    InstrCode {
        name: "divu",
        syntax: Syntax::DivMult,
        code: 27,
    },
    InstrCode {
        name: "mult",
        syntax: Syntax::DivMult,
        code: 24,
    },
    InstrCode {
        name: "multu",
        syntax: Syntax::DivMult,
        code: 25,
    },
    InstrCode {
        name: "nor",
        syntax: Syntax::ArithLog,
        code: 39,
    },
    InstrCode {
        name: "or",
        syntax: Syntax::ArithLog,
        code: 37,
    },
    InstrCode {
        name: "ori",
        syntax: Syntax::ArithLogI,
        code: 13,
    },
    InstrCode {
        name: "sll",
        syntax: Syntax::Shift,
        code: 0,
    },
    InstrCode {
        name: "sllv",
        syntax: Syntax::ShiftV,
        code: 4,
    },
    InstrCode {
        name: "sra",
        syntax: Syntax::Shift,
        code: 3,
    },
    InstrCode {
        name: "srav",
        syntax: Syntax::ShiftV,
        code: 7,
    },
    InstrCode {
        name: "srl",
        syntax: Syntax::Shift,
        code: 2,
    },
    InstrCode {
        name: "srlv",
        syntax: Syntax::ShiftV,
        code: 6,
    },
    InstrCode {
        name: "sub",
        syntax: Syntax::ArithLog,
        code: 34,
    },
    InstrCode {
        name: "subu",
        syntax: Syntax::ArithLog,
        code: 35,
    },
    InstrCode {
        name: "xor",
        syntax: Syntax::ArithLog,
        code: 38,
    },
    InstrCode {
        name: "xori",
        syntax: Syntax::ArithLogI,
        code: 14,
    },
    InstrCode {
        name: "lhi",
        syntax: Syntax::LoadI,
        code: 25,
    },
    InstrCode {
        name: "llo",
        syntax: Syntax::LoadI,
        code: 24,
    },
    InstrCode {
        name: "slt",
        syntax: Syntax::ArithLog,
        code: 42,
    },
    InstrCode {
        name: "sltu",
        syntax: Syntax::ArithLog,
        code: 41,
    },
    InstrCode {
        name: "slti",
        syntax: Syntax::ArithLogI,
        code: 10,
    },
    InstrCode {
        name: "sltiu",
        syntax: Syntax::ArithLogI,
        code: 9,
    },
    InstrCode {
        name: "beq",
        syntax: Syntax::Branch,
        code: 4,
    },
    InstrCode {
        name: "bne",
        syntax: Syntax::Branch,
        code: 5,
    },
    InstrCode {
        name: "blez",
        syntax: Syntax::BranchZ,
        code: 6,
    },
    InstrCode {
        name: "bgtz",
        syntax: Syntax::BranchZ,
        code: 7,
    },
    InstrCode {
        name: "j",
        syntax: Syntax::Jump,
        code: 2,
    },
    InstrCode {
        name: "jal",
        syntax: Syntax::Jump,
        code: 3,
    },
    InstrCode {
        name: "jr",
        syntax: Syntax::JumpR,
        code: 8,
    },
    InstrCode {
        name: "jalr",
        syntax: Syntax::JumpR,
        code: 9,
    },
    InstrCode {
        name: "lb",
        syntax: Syntax::LoadStore,
        code: 32,
    },
    InstrCode {
        name: "lbu",
        syntax: Syntax::LoadStore,
        code: 36,
    },
    InstrCode {
        name: "lh",
        syntax: Syntax::LoadStore,
        code: 33,
    },
    InstrCode {
        name: "lhu",
        syntax: Syntax::LoadStore,
        code: 37,
    },
    InstrCode {
        name: "lw",
        syntax: Syntax::LoadStore,
        code: 35,
    },
    InstrCode {
        name: "sb",
        syntax: Syntax::LoadStore,
        code: 40,
    },
    InstrCode {
        name: "sh",
        syntax: Syntax::LoadStore,
        code: 41,
    },
    InstrCode {
        name: "sw",
        syntax: Syntax::LoadStore,
        code: 43,
    },
    InstrCode {
        name: "mfhi",
        syntax: Syntax::MoveFrom,
        code: 16,
    },
    InstrCode {
        name: "mflo",
        syntax: Syntax::MoveFrom,
        code: 18,
    },
    InstrCode {
        name: "mthi",
        syntax: Syntax::MoveTo,
        code: 17,
    },
    InstrCode {
        name: "mtlo",
        syntax: Syntax::MoveTo,
        code: 19,
    },
    InstrCode {
        name: "trap",
        syntax: Syntax::Trap,
        code: 26,
    },
    InstrCode {
        name: "syscall",
        syntax: Syntax::Syscall,
        code: 12,
    },
    InstrCode {
        name: "mul",
        syntax: Syntax::S2ArithLog,
        code: 2,
    },
];

pub fn get_code<S: Into<String>>(line: S) -> &'static InstrCode<'static> {
    let line_str = line.into();
    let parts: Vec<String> = line_str.split(" ").map(String::from).collect();
    for code in &CODES {
        if parts[0] == code.name {
            return code;
        }
    }
    return &CODES[0];
}

pub fn as_register<S: Into<String>>(arg: S) -> Result<i8, ()> {
    let mut i: i8 = 0;

    let name = &(arg.into())[1..];
    while i < 32 {
        if REGS[i as usize] == name {
            return Ok(i);
        }
        i += 1;
    }
    Err(())
}

pub fn get_ops(
    opfile: Option<&str>,
) -> HashMap<String, Box<dyn Fn(Args<Arg>) -> Vec<&'static InstrCode<'static>>>> {
    todo!();
    let fname = match opfile {
        None => "res/PseudoOps.txt",
        Some(s) => s,
    };

    let instr_table = HashMap::new();

    let total = std::fs::read_to_string(fname);

    match total {
        Ok(tot) => {
            for line in tot.lines() {
                let line_nc_dirty: &str = {
                    // Removes comments, which start with #
                    let pos_opt: Option<usize> = line.find('#');
                    match pos_opt {
                        Some(n) => &line[0..n],
                        None => line,
                    }
                };
                let line_nc = rem_spaces(line_nc_dirty);

                if line_nc.len() == 0 {
                    continue;
                }

                println!("{}", line);
            }
        }
        _ => {
            println!("PseudoOps.txt not found.");
        }
    };

    instr_table
}
