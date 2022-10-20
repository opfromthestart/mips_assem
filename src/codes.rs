use core::option::Option;
use core::option::Option::{None, Some};
use core::result::Result::{Err, Ok};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use crate::{Encoding, rem_spaces};
use crate::tables::{as_register, InstrCode};

pub enum Syntax {
    ArithLog,
    DivMult,
    Shift,
    ShiftV,
    JumpR,
    MoveFrom,
    MoveTo,
    ArithLogI,
    LoadI,
    Branch,
    BranchZ,
    LoadStore,
    Jump,
    Trap,
    Syscall,
    S2ArithLog,
}


#[derive(Clone)]
pub enum Args<T> {
    Three( T, T, T),
    Two( T, T),
    One( T),
    None,
}

#[derive(Clone)]
pub enum Arg {
    Reg(i8),
    Imm(i32),
    Label(String)
}

trait Binary {
    fn to_bin(&self, lbl_adr : &HashMap<String, u32>) -> Option<u32>;
}

impl Binary for Arg {
    fn to_bin(&self, lbl_adr: &HashMap<String, u32>) -> Option<u32> {
        match self {
            Arg::Reg(r) => Some(*r as u32),
            Arg::Imm(n) => Some(*n as u32),
            Arg::Label(l) => {match lbl_adr.get(l) {
                None => {None}
                Some(i) => {Some(*i)}
            }
            }
        }
    }
}

impl Display for Arg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Arg::Reg(r) => {write!(f, "Reg{}", r)}
            Arg::Imm(i) => {write!(f, "{}", i)}
            Arg::Label(l) => {write!(f, "{}", l)}
        }
    }
}

impl Display for Args<String> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Args::Three(a, b, c) => {write!(f, "{0}, {1}, {2}", a, b, c)}
            Args::Two(a, b) => {write!(f, "{0}, {1}", a, b)}
            Args::One(a) => {write!(f, "{0}", a)}
            Args::None => {Ok(())}
        }
    }
}

pub fn get_argument<S : Into<String>>(arg_line : S, sep : Option<String>) -> (String, String) {
    let sep = match sep {
        Some(s) => s,
        None => ",".to_string()
    };

    let arg_line_s = arg_line.into();
    let pos = match arg_line_s.find(&sep) {
        Some(n) => n,
        None => {
            match arg_line_s.find(' ') {
                Some(n) => n,
                None => return (arg_line_s, "".to_owned())
            }
        }
    };

    //let asub : &'a str = &arg_line[0..pos_opt.unwrap()];
    //let astr : String = String::from(asub);
    let arg : String = rem_spaces(&arg_line_s[0..pos]);

    //let rsub : &'a str = &arg_line[0..pos_opt.unwrap()+(1 as usize)];
    //let rstr : String = String::from(rsub);
    let rest : String = rem_spaces(&arg_line_s[pos+(1 as usize)..]);
    return (arg, rest);
}

pub fn get_arguments<S : Into<String>>(arg_line : S, instruction: &InstrCode) -> Args<String>
{
    match instruction.syntax
    {
        Syntax::ArithLog |
        Syntax::S2ArithLog |
        Syntax::Shift |
        Syntax::ShiftV |
        Syntax::ArithLogI |
        Syntax::Branch => {
            let (arg1, arg_line1) = get_argument(arg_line, None);
            let (arg2, arg_line2) = get_argument(arg_line1, None);
            let (arg3, _) = get_argument(arg_line2, None);
            return Args::Three(arg1, arg2, arg3);
        }
        Syntax::DivMult |
        Syntax::LoadI |
        Syntax::BranchZ => {
            let (arg1, arg_line1) = get_argument(arg_line, None);
            let (arg2, _) = get_argument(arg_line1, None);
            return Args::Two(arg1, arg2);
        }
        Syntax::JumpR |
        Syntax::MoveFrom |
        Syntax::MoveTo |
        Syntax::Jump |
        Syntax::Trap => {
            let (arg1, _) = get_argument(arg_line, None);
            return Args::One(arg1);
        }
        Syntax::LoadStore => {
            let (arg1, arg_line1)  = get_argument(arg_line, None);
            let (arg2, arg_line2) = get_argument(arg_line1, Some("(".to_string()));
            let lp = arg_line2.find(")");

            let arg3 : String = match lp {
                Some(n) => String::from(&arg_line2[0..n]),
                None => {println!("Third argument not given properly in 0($t0) format."); String::from(arg_line2)}
            };
            return Args::Three(arg1, arg2, arg3);
        }
        Syntax::Syscall => Args::None
    }
}

fn parse_num<S : Into<String>>(arg_i : S) -> Result<i32, String> {
    let arg = arg_i.into();
    if arg.len() < 2 {
        return match arg.parse() {
            Ok(n) => (Ok(n)),
            Err(e) => (Err(e.to_string()))
        };
    }
    let pref = &arg[..2];
    if pref == "0x" {
        let res = hex::decode(&arg[2..]);
        return match res {
            Ok(v) => {
                let mut res : i32 = 0;
                let mut i = 0;
                while i < v.len() {
                    res <<= 8;
                    res += v[i] as i32;
                    i += 1;
                    if i==4 {
                        break;
                    }
                }
                Ok(res)
            }
            Err(e) => {Err(e.to_string())}
        };
    }
    if pref == "0b" {
        return match i32::from_str_radix(&arg[2..], 2) {
            Ok(n) => {Ok(n)}
            Err(e) => {Err(e.to_string())}
        };
    }
    return match arg.parse() {
            Ok(n) => (Ok(n)),
            Err(e) => (Err(e.to_string()))
        };
}

pub fn get_argument2<S : Into<String>>(arg_s : S) -> Arg {
    let arg_str = rem_spaces(arg_s);

    let reg_test = as_register(&arg_str);
    match reg_test {
        Ok(n) => Arg::Reg(n),
        Err(_) => {
            let num_test = parse_num(&arg_str);
            match num_test {
                Ok(n) => Arg::Imm(n),
                Err(_) => Arg::Label(arg_str.into())
            }
        }
    }
}

pub fn get_arguments2<S : Into<String>>(arg_line_s : S) -> Args<Arg> {
    let arg_line = rem_spaces(arg_line_s);
    if arg_line.len() == 0 {
        return Args::None;
    }
    let is_store : Option<usize> = arg_line.find("(");

    match is_store {
        None => {
            let (p1, arg1) = match arg_line.find(",") {
                None => {
                    return Args::One(get_argument2(arg_line));
                }
                Some(n) => (n, get_argument2(&arg_line[..n]))
            };

            let argl2 = &arg_line[p1+(1 as usize)..];
            let p2 : Option<usize> = argl2.find(",");
            return match p2 {
                None => Args::Two(arg1, get_argument2(argl2)),
                Some(n) => Args::Three(arg1, get_argument2(&argl2[..n]),
                                              get_argument2(&argl2[n+(1 as usize)..]))
            };
        }
        Some(p2) => {
            let p3 = match arg_line.find(")") {
                Some(n) => n,
                None => {println!("Missing ending parenthesis in {}.", arg_line); arg_line.len()}
            };

            let p1 : Option<usize> = arg_line.find(",");
            return match p1 {
                None => Args::Two(get_argument2(&arg_line[..p2]),
                                     get_argument2(&arg_line[p2+(1 as usize)..p3])),
                Some(n) => Args::Three(get_argument2(&arg_line[..n]),
                                            get_argument2(&arg_line[n+(1 as usize)..p2]),
                                            get_argument2(&arg_line[p2+(1 as usize)..p3]))
            };
        }
    };
}

pub fn get_enc(instr : &InstrCode, args: Args<String>, lbl_adr : &HashMap<String, u32>, line: u32, adr: u32) -> Encoding {
    match instr.syntax {
        Syntax::ArithLog => {
            let (a1, a2, a3) = match args {
                Args::Three(a1, a2, a3) => (a1, a2, a3),
                _ => {println!("Invalid number of arguments found on line {0}.", line); ("".to_string(), "".to_string(), "".to_string())}
            };
            let d = as_register(&a1);
            let s = as_register(&a2);
            let t = as_register(&a3);

            match d {
                Ok(dr) => {
                    match s {
                        Ok(sr) => {
                            match t {
                                Ok(tr) => {
                                    return Encoding::Register(0,sr, tr, dr, 0, instr.code);
                                }
                                Err(_) => {println!("Register \"{0}\" not found in {1} on line {2}.", a3, instr.name, line)}
                            }
                        }
                        Err(_) => {println!("Register \"{0}\" not found in {1} on line {2}.", a2, instr.name, line)}
                    }
                }
                Err(_) => {println!("Register \"{0}\" not found in {1} on line {2}.", a1, instr.name, line)}
            }

            Encoding::Register(0,0, 0, 0, 0, instr.code)
        }
        Syntax::DivMult => {
            let (a1, a2) = match args {
                Args::Two(a1, a2) => (a1, a2),
                _ => {println!("Invalid number of arguments found on line {0}.", line); ("".to_string(), "".to_string())}
            };
            let s = as_register(&a1);
            let t = as_register(&a2);

            match s {
                Ok(sr) => {
                    match t {
                        Ok(tr) => {
                            return Encoding::Register(0,sr, tr, 0, 0, instr.code);
                        }
                        Err(_) => { println!("Register \"{0}\" not found in {1} on line {2}.", a2, instr.name, line) }
                    }
                }
                Err(_) => { println!("Register \"{0}\" not found in {1} on line {2}.", a1, instr.name, line) }
            }

            Encoding::Register(0,0, 0, 0, 0, instr.code)
        }
        Syntax::Shift => {
            let (a1, a2, a3) = match args {
                Args::Three(a1, a2, a3) => (a1, a2, a3),
                _ => {println!("Invalid number of arguments found on line {0}.", line); ("".to_string(), "".to_string(), "".to_string())}
            };
            let d = as_register(&a1);
            let t = as_register(&a2);
            let a = parse_num(&a3);

            match a {
                Ok(ar) => {
                    match t {
                        Ok(tr) => {
                            match d {
                                Ok(dr) => {
                                    return Encoding::Register(0,0, tr, dr, ar as i8, instr.code);
                                }
                                Err(_) => {println!("Register \"{0}\" not found in {1} on line {2}.", a1, instr.name, line)}
                            }
                        }
                        Err(_) => {println!("Register \"{0}\" not found in {1} on line {2}.", a2, instr.name, line)}
                    }
                }
                Err(_) => {println!("Shift amount \"{0}\" not valid in {1} on line {2}.", a3, instr.name, line)}
            }

            Encoding::Register(0,0, 0, 0, 0, instr.code)
        }
        Syntax::ShiftV => {
            let (a1, a2, a3) = match args {
                Args::Three(a1, a2, a3) => (a1, a2, a3),
                _ => {println!("Invalid number of arguments found on line {0}.", line); ("".to_string(), "".to_string(), "".to_string())}
            };
            let d = as_register(&a1);
            let t = as_register(&a2);
            let s = as_register(&a3);

            match d {
                Ok(dr) => {
                    match s {
                        Ok(sr) => {
                            match t {
                                Ok(tr) => {
                                    return Encoding::Register(0,sr, tr, dr, 0, instr.code);
                                }
                                Err(_) => {println!("Register \"{0}\" not found in {1} on line {2}.", a2, instr.name, line)}
                            }
                        }
                        Err(_) => {println!("Register \"{0}\" not found in {1} on line {2}.", a3, instr.name, line)}
                    }
                }
                Err(_) => {println!("Register \"{0}\" not found in {1} on line {2}.", a1, instr.name, line)}
            }

            Encoding::Register(0,0, 0, 0, 0, instr.code)
        }
        Syntax::JumpR | Syntax::MoveTo => {
            let a1 = match args {
                Args::One(a1) => a1,
                _ => {println!("Invalid number of arguments found on line {0}.", line); "".to_string()}
            };
            let s = as_register(&a1);

            match s {
                Ok(sr) => {
                    return Encoding::Register(0,sr, 0, 0, 0, instr.code);
                }
                Err(_) => { println!("Register \"{0}\" not found in {1} on line {2}.", a1, instr.name, line) }
            }

            Encoding::Register(0,0, 0, 0, 0, instr.code)
        }
        Syntax::MoveFrom => {
            let a1 = match args {
                Args::One(a1) => (a1),
                _ => {println!("Invalid number of arguments found on line {0}.", line); "".to_string()}
            };
            let d = as_register(&a1);

            match d {
                Ok(dr) => {
                    return Encoding::Register(0,0, 0, dr, 0, instr.code);
                }
                Err(_) => { println!("Register \"{0}\" not found in {1} on line {2}.", a1, instr.name, line) }
            }

            Encoding::Register(0,0, 0, 0, 0, instr.code)
        }
        Syntax::ArithLogI => {
            let (a1, a2, a3) = match args {
                Args::Three(a1, a2, a3) => (a1, a2, a3),
                _ => {println!("Invalid number of arguments found on line {0}.", line); ("".to_string(), "".to_string(), "".to_string())}
            };
            let t = as_register(&a1);
            let s = as_register(&a2);
            let i = parse_num(&a3);

            match i {
                Ok(ir) => {
                    match s {
                        Ok(sr) => {
                            match t {
                                Ok(tr) => {
                                    return Encoding::Immediate(instr.code, sr, tr, ir as i16);
                                }
                                Err(_) => {println!("Register \"{0}\" not found in {1} on line {2}.", a1, instr.name, line)}
                            }
                        }
                        Err(_) => {println!("Register \"{0}\" not found in {1} on line {2}.", a2, instr.name, line)}
                    }
                }
                Err(_) => {println!("Immediate value \"{0}\" not valid in {1} on line {2}.", a3, instr.name, line)}
            }

            Encoding::Immediate(instr.code, 0, 0, 0)
        }
        Syntax::LoadI => {
            let (a1, a2) = match args {
                Args::Two(a1, a2) => (a1, a2),
                _ => {println!("Invalid number of arguments found on line {0}.", line); ("".to_string(), "".to_string())}
            };
            let t = as_register(&a1);
            let i = parse_num(&a2);

            match i {
                Ok(ir) => {
                    match t {
                        Ok(tr) => {
                            return Encoding::Immediate(instr.code, 0, tr, ir as i16);
                        }
                        Err(_) => { println!("Register \"{0}\" not found in {1} on line {2}.", a1, instr.name, line) }
                    }
                }
                Err(_) => { println!("Immediate value \"{0}\" not valid in {1} on line {2}.", a2, instr.name, line) }
            }

            Encoding::Immediate(instr.code, 0, 0, 0)
        }
        Syntax::Branch => {
            let (a1, a2, a3) = match args {
                Args::Three(a1, a2, a3) => (a1, a2, a3),
                _ => {println!("Invalid number of arguments found on line {0}.", line); ("".to_string(), "".to_string(), "".to_string())}
            };
            let s = as_register(&a1);
            let t = as_register(&a2);
            let i = lbl_adr.get(&a3);

            match i {
                Some(ir) => {
                    match s {
                        Ok(sr) => {
                            match t {
                                Ok(tr) => {
                                    let i_m : i16 = (((*ir as i32 - adr as i32) >> 2) - 1) as i16;
                                    //println!("{}", i_m);
                                    return Encoding::Immediate(instr.code, sr, tr, i_m);
                                }
                                Err(_) => {println!("Register \"{0}\" not found in {1} on line {2}.", a2, instr.name, line)}
                            }
                        }
                        Err(_) => {println!("Register \"{0}\" not found in {1} on line {2}.", a1, instr.name, line)}
                    }
                }
                None => {println!("Label \"{0}\" not found, in {1} on line {2}.", a3, instr.name, line)}
            }

            Encoding::Immediate(instr.code, 0, 0, 0)
        }
        Syntax::BranchZ => {
            let (a1, a2) = match args {
                Args::Two(a1, a2) => (a1, a2),
                _ => {println!("Invalid number of arguments found on line {0}.", line); ("".to_string(), "".to_string())}
            };
            let s = as_register(&a1);
            let i = lbl_adr.get(&a2);

            match i {
                Some(ir) => {
                    match s {
                        Ok(sr) => {
                            let i_m: i16 = (((*ir as i32 - adr as i32) >> 2) - 1) as i16;
                                    //println!("{}", i_m);
                            return Encoding::Immediate(instr.code, sr, 0, i_m);
                        }
                        Err(_) => { println!("Register \"{0}\" not found in {1} on line {2}.", a1, instr.name, line) }
                    }
                }
                None => { println!("Label \"{0}\" not found, in {1} on line {2}.", a2, instr.name, line) }
            }

            Encoding::Immediate(instr.code, 0, 0, 0)
        }
        Syntax::LoadStore => {
            let (a1, a2, a3) = match args {
                Args::Three(a1, a2, a3) => (a1, a2, a3),
                _ => {println!("Invalid number of arguments found on line {0}.", line); ("".to_string(), "".to_string(), "".to_string())}
            };
            let t = as_register(&a1);
            let i = parse_num(&a2);
            let s = as_register(&a3);

            match i {
                Ok(ir) => {
                    match s {
                        Ok(sr) => {
                            match t {
                                Ok(tr) => {
                                    let i_m : i16 = (((ir as i32 - adr as i32) >> 2) - 1) as i16;
                                    return Encoding::Immediate(instr.code, sr, tr, i_m);
                                }
                                Err(_) => {println!("Register \"{0}\" not found in {1} on line {2}.", a1, instr.name, line)}
                            }
                        }
                        Err(_) => {println!("Register \"{0}\" not found in {1} on line {2}.", a3, instr.name, line)}
                    }
                }
                Err(_) => {println!("Number \"{0}\" not valid, in {1} on line {2}.", a2, instr.name, line)}
            }

            Encoding::Immediate(instr.code, 0, 0, 0)
        }
        Syntax::Jump => {
            let a1 = match args {
                Args::One(a1) => (a1),
                _ => {println!("Invalid number of arguments found on line {0}.", line); "".to_string()}
            };
            let i = lbl_adr.get(&a1);

            match i {
                None => {println!("Label \"{0}\" not found, in {1} on line {2}.", a1, instr.name, line)}
                Some(ir) => {return Encoding::Jump(instr.code, (*ir as i32) >> 2);}
            }

            Encoding::Jump(instr.code, 0)
        }
        Syntax::Trap => {
            let a1 = match args {
                Args::One(a1) => (a1),
                _ => {println!("Invalid number of arguments found on line {0}.", line); "".to_string()}
            };
            let i = parse_num(&a1);

            match i {
                Err(_) => {println!("Number \"{0}\" not valid, in {1} on line {2}.", a1, instr.name, line)}
                Ok(ir) => {return Encoding::Jump(instr.code, ir);}
            }

            Encoding::Jump(instr.code, 0)
        }
        Syntax::Syscall => {Encoding::Jump(0, instr.code as i32)}
        Syntax::S2ArithLog => {
            let (a1, a2, a3) = match args {
                Args::Three(a1, a2, a3) => (a1, a2, a3),
                _ => {println!("Invalid number of arguments found on line {0}.", line); ("".to_string(), "".to_string(), "".to_string())}
            };
            let d = as_register(&a1);
            let s = as_register(&a2);
            let t = as_register(&a3);

            match d {
                Ok(dr) => {
                    match s {
                        Ok(sr) => {
                            match t {
                                Ok(tr) => {
                                    return Encoding::Register(28 ,sr, tr, dr, 0, instr.code);
                                }
                                Err(_) => {println!("Register \"{0}\" not found in {1} on line {2}.", a3, instr.name, line)}
                            }
                        }
                        Err(_) => {println!("Register \"{0}\" not found in {1} on line {2}.", a2, instr.name, line)}
                    }
                }
                Err(_) => {println!("Register \"{0}\" not found in {1} on line {2}.", a1, instr.name, line)}
            }

            Encoding::Register(28, 0, 0, 0, 0, instr.code)
        }
    }
}

pub fn get_enc2(instr : &InstrCode, args: Args<Arg>, lbl_adr : &HashMap<String, u32>, line: u32, adr: u32) -> Encoding {
    match instr.syntax {
        Syntax::ArithLog => {
            let (a1, a2, a3) = match args {
                Args::Three(a1, a2, a3) => (a1, a2, a3),
                _ => {println!("Invalid number of arguments found on line {0}.", line); panic!();}
            };
            let d = a1.to_bin(lbl_adr);
            let s = a2.to_bin(lbl_adr);
            let t = a3.to_bin(lbl_adr);

            match d {
                Some(dr) => {
                    match s {
                        Some(sr) => {
                            match t {
                                Some(tr) => {
                                    return Encoding::Register(0, sr as i8, tr as i8, dr as i8, 0, instr.code);
                                }
                                None => {println!("Register \"{0}\" not found in {1} on line {2}.", a3, instr.name, line)}
                            }
                        }
                        None => {println!("Register \"{0}\" not found in {1} on line {2}.", a2, instr.name, line)}
                    }
                }
                None => {println!("Register \"{0}\" not found in {1} on line {2}.", a1, instr.name, line)}
            }

            Encoding::Register(0,0, 0, 0, 0, instr.code)
        }
        Syntax::DivMult => {
            let (a1, a2) = match args {
                Args::Two(a1, a2) => (a1, a2),
                _ => {println!("Invalid number of arguments found on line {0}.", line); panic!();}
            };
            let s = a1.to_bin(lbl_adr);
            let t = a2.to_bin(lbl_adr);

            match s {
                Some(sr) => {
                    match t {
                        Some(tr) => {
                            return Encoding::Register(0, sr as i8, tr as i8, 0, 0, instr.code);
                        }
                        None => { println!("Register \"{0}\" not found in {1} on line {2}.", a2, instr.name, line) }
                    }
                }
                None => { println!("Register \"{0}\" not found in {1} on line {2}.", a1, instr.name, line) }
            }

            Encoding::Register(0,0, 0, 0, 0, instr.code)
        }
        Syntax::Shift => {
            let (a1, a2, a3) = match args {
                Args::Three(a1, a2, a3) => (a1, a2, a3),
                _ => {println!("Invalid number of arguments found on line {0}.", line); panic!();}
            };
            let d = a1.to_bin(lbl_adr);
            let t = a2.to_bin(lbl_adr);
            let a = a3.to_bin(lbl_adr);

            match a {
                Some(ar) => {
                    match t {
                        Some(tr) => {
                            match d {
                                Some(dr) => {
                                    return Encoding::Register(0, 0, tr as i8, dr as i8, ar as i8, instr.code);
                                }
                                None => {println!("Register \"{0}\" not found in {1} on line {2}.", a1, instr.name, line)}
                            }
                        }
                        None => {println!("Register \"{0}\" not found in {1} on line {2}.", a2, instr.name, line)}
                    }
                }
                None => {println!("Shift amount \"{0}\" not valid in {1} on line {2}.", a3, instr.name, line)}
            }

            Encoding::Register(0,0, 0, 0, 0, instr.code)
        }
        Syntax::ShiftV => {
            let (a1, a2, a3) = match args {
                Args::Three(a1, a2, a3) => (a1, a2, a3),
                _ => {println!("Invalid number of arguments found on line {0}.", line); panic!();}
            };
            let d = a1.to_bin(lbl_adr);
            let t = a2.to_bin(lbl_adr);
            let s = a3.to_bin(lbl_adr);

            match d {
                Some(dr) => {
                    match s {
                        Some(sr) => {
                            match t {
                                Some(tr) => {
                                    return Encoding::Register(0, sr as i8, tr as i8, dr as i8, 0, instr.code);
                                }
                                None => {println!("Register \"{0}\" not found in {1} on line {2}.", a2, instr.name, line)}
                            }
                        }
                        None => {println!("Register \"{0}\" not found in {1} on line {2}.", a3, instr.name, line)}
                    }
                }
                None => {println!("Register \"{0}\" not found in {1} on line {2}.", a1, instr.name, line)}
            }

            Encoding::Register(0,0, 0, 0, 0, instr.code)
        }
        Syntax::JumpR | Syntax::MoveTo => {
            let a1 = match args {
                Args::One(a1) => a1,
                _ => {println!("Invalid number of arguments found on line {0}.", line); panic!()}
            };
            let s = a1.to_bin(lbl_adr);

            match s {
                Some(sr) => {
                    return Encoding::Register(0, sr as i8, 0, 0, 0, instr.code);
                }
                None => { println!("Register \"{0}\" not found in {1} on line {2}.", a1, instr.name, line) }
            }

            Encoding::Register(0,0, 0, 0, 0, instr.code)
        }
        Syntax::MoveFrom => {
            let a1 = match args {
                Args::One(a1) => (a1),
                _ => {println!("Invalid number of arguments found on line {0}.", line); panic!();}
            };
            let d = a1.to_bin(lbl_adr);

            match d {
                Some(dr) => {
                    return Encoding::Register(0, 0, 0, dr as i8, 0, instr.code);
                }
                None => { println!("Register \"{0}\" not found in {1} on line {2}.", a1, instr.name, line) }
            }

            Encoding::Register(0,0, 0, 0, 0, instr.code)
        }
        Syntax::ArithLogI => {
            let (a1, a2, a3) = match args {
                Args::Three(a1, a2, a3) => (a1, a2, a3),
                _ => {println!("Invalid number of arguments found on line {0}.", line); panic!();}
            };
            let t = a1.to_bin(lbl_adr);
            let s = a2.to_bin(lbl_adr);
            let i = a3.to_bin(lbl_adr);

            match i {
                Some(ir) => {
                    match s {
                        Some(sr) => {
                            match t {
                                Some(tr) => {
                                    return Encoding::Immediate(instr.code, sr as i8, tr as i8, ir as i16);
                                }
                                None => {println!("Register \"{0}\" not found in {1} on line {2}.", a1, instr.name, line)}
                            }
                        }
                        None => {println!("Register \"{0}\" not found in {1} on line {2}.", a2, instr.name, line)}
                    }
                }
                None => {println!("Immediate value \"{0}\" not valid in {1} on line {2}.", a3, instr.name, line)}
            }

            Encoding::Immediate(instr.code, 0, 0, 0)
        }
        Syntax::LoadI => {
            let (a1, a2) = match args {
                Args::Two(a1, a2) => (a1, a2),
                _ => {println!("Invalid number of arguments found on line {0}.", line); panic!();}
            };
            let t = a1.to_bin(lbl_adr);
            let i = a2.to_bin(lbl_adr);

            match i {
                Some(ir) => {
                    match t {
                        Some(tr) => {
                            return Encoding::Immediate(instr.code, 0, tr as i8, ir as i16);
                        }
                        None => { println!("Register \"{0}\" not found in {1} on line {2}.", a1, instr.name, line) }
                    }
                }
                None => { println!("Immediate value \"{0}\" not valid in {1} on line {2}.", a2, instr.name, line) }
            }

            Encoding::Immediate(instr.code, 0, 0, 0)
        }
        Syntax::Branch => {
            let (a1, a2, a3) = match args {
                Args::Three(a1, a2, a3) => (a1, a2, a3),
                _ => {println!("Invalid number of arguments found on line {0}.", line); panic!();}
            };
            let s = a1.to_bin(lbl_adr);
            let t = a2.to_bin(lbl_adr);
            let i = a3.to_bin(lbl_adr);

            match i {
                Some(ir) => {
                    match s {
                        Some(sr) => {
                            match t {
                                Some(tr) => {
                                    let i_m : i16 = (((ir as i32 - adr as i32) >> 2) - 1) as i16;
                                    //println!("{}", i_m);
                                    return Encoding::Immediate(instr.code, sr as i8, tr as i8, i_m);
                                }
                                None => {println!("Register \"{0}\" not found in {1} on line {2}.", a2, instr.name, line)}
                            }
                        }
                        None => {println!("Register \"{0}\" not found in {1} on line {2}.", a1, instr.name, line)}
                    }
                }
                None => {println!("Label \"{0}\" not found, in {1} on line {2}.", a3, instr.name, line)}
            }

            Encoding::Immediate(instr.code, 0, 0, 0)
        }
        Syntax::BranchZ => {
            let (a1, a2) = match args {
                Args::Two(a1, a2) => (a1, a2),
                _ => {println!("Invalid number of arguments found on line {0}.", line); panic!();}
            };
            let s = a1.to_bin(lbl_adr);
            let i = a2.to_bin(lbl_adr);

            match i {
                Some(ir) => {
                    match s {
                        Some(sr) => {
                            let i_m: i16 = (((ir as i32 - adr as i32) >> 2) - 1) as i16;
                                    //println!("{}", i_m);
                            return Encoding::Immediate(instr.code, sr as i8, 0, i_m);
                        }
                        None => { println!("Register \"{0}\" not found in {1} on line {2}.", a1, instr.name, line) }
                    }
                }
                None => { println!("Label \"{0}\" not found, in {1} on line {2}.", a2, instr.name, line) }
            }

            Encoding::Immediate(instr.code, 0, 0, 0)
        }
        Syntax::LoadStore => {
            let (a1, a2, a3) = match args {
                Args::Three(a1, a2, a3) => (a1, a2, a3),
                _ => {println!("Invalid number of arguments found on line {0}.", line); panic!();}
            };
            let t = a1.to_bin(lbl_adr);
            let i = a2.to_bin(lbl_adr);
            let s = a3.to_bin(lbl_adr);

            match i {
                Some(ir) => {
                    match s {
                        Some(sr) => {
                            match t {
                                Some(tr) => {
                                    let i_m : i16 = (((ir as i32 - adr as i32) >> 2) - 1) as i16;
                                    return Encoding::Immediate(instr.code, sr as i8, tr as i8, i_m);
                                }
                                None => {println!("Register \"{0}\" not found in {1} on line {2}.", a1, instr.name, line)}
                            }
                        }
                        None => {println!("Register \"{0}\" not found in {1} on line {2}.", a3, instr.name, line)}
                    }
                }
                None => {println!("Number \"{0}\" not valid, in {1} on line {2}.", a2, instr.name, line)}
            }

            Encoding::Immediate(instr.code, 0, 0, 0)
        }
        Syntax::Jump => {
            let a1 = match args {
                Args::One(a1) => (a1),
                _ => {println!("Invalid number of arguments found on line {0}.", line); panic!()}
            };
            let i = a1.to_bin(lbl_adr);

            match i {
                None => {println!("Label \"{0}\" not found, in {1} on line {2}.", a1, instr.name, line)}
                Some(ir) => {return Encoding::Jump(instr.code, (ir as i32) >> 2);}
            }

            Encoding::Jump(instr.code, 0)
        }
        Syntax::Trap => {
            let a1 = match args {
                Args::One(a1) => (a1),
                _ => {println!("Invalid number of arguments found on line {0}.", line); panic!();}
            };
            let i = a1.to_bin(lbl_adr);

            match i {
                None => {println!("Number \"{0}\" not valid, in {1} on line {2}.", a1, instr.name, line)}
                Some(ir) => {return Encoding::Jump(instr.code, ir as i32);}
            }

            Encoding::Jump(instr.code, 0)
        }
        Syntax::Syscall => {Encoding::Jump(0, instr.code as i32)}
        Syntax::S2ArithLog => {
            let (a1, a2, a3) = match args {
                Args::Three(a1, a2, a3) => (a1, a2, a3),
                _ => {println!("Invalid number of arguments found on line {0}.", line); panic!();}
            };
            let d = a1.to_bin(lbl_adr);
            let s = a2.to_bin(lbl_adr);
            let t = a3.to_bin(lbl_adr);

            match d {
                Some(dr) => {
                    match s {
                        Some(sr) => {
                            match t {
                                Some(tr) => {
                                    return Encoding::Register(28, sr as i8, tr as i8, dr as i8, 0, instr.code);
                                }
                                None => {println!("Register \"{0}\" not found in {1} on line {2}.", a3, instr.name, line)}
                            }
                        }
                        None => {println!("Register \"{0}\" not found in {1} on line {2}.", a2, instr.name, line)}
                    }
                }
                None => {println!("Register \"{0}\" not found in {1} on line {2}.", a1, instr.name, line)}
            }

            Encoding::Register(28, 0, 0, 0, 0, instr.code)
        }
    }
}