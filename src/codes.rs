use core::option::Option;
use core::option::Option::{None, Some};
use core::result::Result::{Err, Ok};
use std::collections::HashMap;
use crate::{Args, Encoding, parse_num, rem_spaces};
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

pub fn get_argument(arg_line : String, sep : Option<String>) -> (String, String) {
    let sep = match sep {
        Some(s) => s,
        None => ",".to_string()
    };

    let mut pos_opt = arg_line.find(&sep);

    if pos_opt == None
    {
        //std::cerr << "Expected " << num_args << " arguments and got " << argpos << std::endl;
        pos_opt = arg_line.find(' ');
        if pos_opt == None {
            //let mt : &'a str = "";
            //let mt2 : String = String::from(mt);
            return (arg_line, String::from(""));
        }
    }

    //let asub : &'a str = &arg_line[0..pos_opt.unwrap()];
    //let astr : String = String::from(asub);
    let arg : String = rem_spaces(String::from(&arg_line[0..pos_opt.unwrap()]));

    //let rsub : &'a str = &arg_line[0..pos_opt.unwrap()+(1 as usize)];
    //let rstr : String = String::from(rsub);
    let rest : String = rem_spaces(String::from(&arg_line[pos_opt.unwrap()+(1 as usize)..]));
    return (arg, rest);
}

pub fn get_arguments(arg_line : String, instruction: &InstrCode) -> Args
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

pub fn get_enc(instr : &InstrCode, args: Args, lbl_adr : &HashMap<String, u32>, line: u32, adr: u32) -> Encoding {
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
                                    println!("{}", i_m);
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
                                    println!("{}", i_m);
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
