use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;

use hex::FromHexError;
use to_binary::BinaryString;
use crate::codes::{get_arguments, get_enc};

mod tables;
mod codes;

extern crate rev_slice;

//use std::env;

enum Syntax {
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
}

enum Args {
    Three( String, String, String),
    Two( String, String),
    One( String),
    None,
}

impl Display for Args {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Args::Three(a, b, c) => {write!(f, "{0}, {1}, {2}", a, b, c)}
            Args::Two(a, b) => {write!(f, "{0}, {1}", a, b)}
            Args::One(a) => {write!(f, "{0}", a)}
            Args::None => {Ok(())}
        }
    }
}

enum Encoding {
    // s, t, d, a, f
    Register(i8, i8, i8, i8, i8),
    Immediate(i8, i8, i8, i16),
    Jump(i8, i32),
}

enum Line<'a> {
    Instr(&'a InstrCode<'a>, Args),
    Label(String),
    Data(Vec<u8>),
}

fn rem_spaces(inp : String) -> String {
    if inp.len() == 0 {
        return String::from("");
    }
    let mut begin : usize = 0;
    let mut end : usize = inp.len();
    while inp[begin..begin+(1 as usize)].eq(" ") {
        begin+=1;
    }
    while inp[end-1..end].eq(" ") && end>begin {
        end-=1;
    }
    //let sub : &'a str = &inp[begin..end];
    //let ret : String = String::from(sub);
    return String::from(&inp[begin..end]);
}

fn pass1(assem : &String) -> Vec<(Line,u32)>
{
    let mut lines : Vec<(Line, u32)> = vec!();
    let mut labels : Vec<(String,u32)> = vec!();
    let mut cur_label : Option<String> = None;
    let mut curline = 0;
    for line in assem.lines()
    {
        curline += 1;

        //println!("{}",curline);

        let line_nc_dirty = { // Removes comments, which start with #
            let pos_opt: Option<usize> = line.find('#');
            match pos_opt
            {
                Some(n) => String::from(&line[0..n]),
                None => line.to_string()
            }
        };
        let line_nc = rem_spaces(line_nc_dirty);
        //println!("{}",line_nc);

        if line_nc.len()==0 {
            continue;
        }

        if line_nc[..1].ne(" ") && line_nc[..1].ne("\t")
        {
            cur_label = None;
        }
        else if cur_label == None
        {
            println!("Code indented without label\nLine {0}: {1}", curline, line);
        }

        let line_nl = {
            let pos_opt: Option<usize> = line_nc.find(':');
            match pos_opt {
                None => line_nc,
                Some(pos) => {
                    let lname = String::from(&line_nc[0.. pos]);

                    if let Some(_) = lname.find(" ") {
                        println!("Invalid label name \"{0}\" found.\nLine {1}: {2}", lname, curline, line);
                    }

                    if {
                        let mut fail = false;
                        for (label, lline) in &labels {
                            if label.eq(&lname) {
                                println!("Duplicate label {0} found.\nFirst was found at {1} and second was found at {2}.", lname, lline, curline);
                                fail = true;
                                break;
                            }
                        }
                        !fail
                    } {
                        lines.push((Line::Label(lname), curline));
                        labels.push((String::from(&line_nc[0.. pos]), curline));
                        cur_label = Some(String::from(&line_nc[0.. pos]));
                    }
                    rem_spaces(String::from(&line_nc[pos+ (1 as usize)..]))
                }
            }
        };

        if line_nl.len() == 0 {
            continue;
        }
        if line_nl[..1].eq(".")
        {
            let pos_opt = line_nl.find(" ");
            //todo!("Directinges");
            match pos_opt {
                Some(n) => {
                    let directive = String::from(&line_nl[0..n]);
                    let dir_data = String::from(&line_nl[n..]);
                    if directive.eq(".asciiz") {
                        let begin_opt = dir_data.find("\"");
                        let begin = match begin_opt {
                            Some(n) => n as isize,
                            None => -1
                        };
                        let end_opt = dir_data[begin as usize + 1 as usize..].find("\"");
                        let end: isize = match end_opt {
                            Some(n) => n as isize,
                            None => -1
                        };
                        if begin == -1 || end == -1 {
                            println!("Directive .asciiz is missing beginning or ending parentheses\nLine {0}: {1}", curline, line);
                            continue;
                        }
                        let ascii = dir_data[begin as usize+(1 as usize)..end as usize-(1 as usize)].as_bytes();
                        let mut byte_vec : Vec<u8> = Vec::new();
                        for i in ascii {
                            byte_vec.push(*i);
                        }
                        byte_vec.push(0);
                        lines.push((Line::Data(byte_vec), curline));
                    }
                }
                None => {}
            }
            continue;
        }

        //println!("{}",line_nl);

        let line_nl2 = line_nl.clone();
        let code : &InstrCode = get_code(line_nl2);
        if code.name == ""
        {
            println!("Invalid instruction code {0} found.\nLine {1}: {2}", code.name, curline, line);
        }

        let line_args : String = {
            let pos_opt : Option<usize> = line.find(' ');

            match pos_opt {
                None => {if let Syntax::Syscall = code.syntax {
                    String::from(line_nl)
                }
                else {
                    println!("No arguments found when some are expected\nLine {0}: {1}", curline, line);
                    String::from(&line_nl[(code.name.len())..])
                }}
                Some(pos) => {rem_spaces(String::from(&line_nl[pos..]))}
            }
        };

        //println!("{}",line_args);

        let adata : Args = get_arguments(line_args, code);

        lines.push((Line::Instr(code, adata), curline));
    }

    return lines;
}

fn parse_num(arg : &String) -> Result<i32, String> {
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

fn get_bin(enc : Encoding) -> Vec<u8> {
    match enc {
        Encoding::Register(s, t, d, a, f) => {
            let mut b: Vec<u8> = Vec::new();
            b.push((s >> 3) as u8);
            b.push((((s as i16) << 5)%256 + t as i16) as u8);
            b.push((((d as i16) << 3)%256 + (a >> 2) as i16) as u8);
            b.push((((a as i16) << 6)%256 + f as i16) as u8);
            b
        }
        Encoding::Immediate(o, s, t, i) => {
            let mut b: Vec<u8> = Vec::new();
            b.push((o << 2 + s >> 3) as u8);
            b.push((((s as i16) << 5)%256 + t as i16) as u8);
            b.push((i >> 8) as u8);
            b.push((i) as u8);
            b
        }
        Encoding::Jump(o, i) => {
            let mut b: Vec<u8> = Vec::new();
            b.push((o << 2 + i / (2 << 24)) as u8);
            b.push((i >> 16) as u8);
            b.push((i >> 8) as u8);
            b.push((i) as u8);
            b
        }
    }
}

fn pass2(lines : Vec<(Line, u32)>) -> Vec<u8> {
    let mut machine_code = Vec::new();
    let mut lbl_adr : HashMap<String, u32> = HashMap::new();
    let mut counter : u32 = 0x1000;

    //TODO i did a goof, this needs to be 2 parts
    for (i, ln) in lines {
        match i {
            Line::Label(name) => {
                if name.eq("START") {
                    let mut upd_lbl_adr = HashMap::new();
                    for (lbl, c) in &lbl_adr {
                        upd_lbl_adr.insert(String::from(lbl), c+4);
                    }
                    for (lbl, c) in upd_lbl_adr {
                        lbl_adr.insert(lbl, c);
                    }
                    let mut i = 0;
                    for b in get_bin(Encoding::Jump(2, (counter as i32 - 0x1000) >> 2 + 1)) {
                        machine_code.insert(i, b);
                        i+=1;
                    }
                    counter += 4;
                }
                lbl_adr.insert(name, counter);
            }
            //n => {counter += get_line_length(n);}
            Line::Instr(instr, args) => {
                let enc = get_enc(instr, args, &lbl_adr, ln, counter);
                let data = get_bin(enc);
                for i in data {
                    machine_code.push(i)
                }
                counter += 4;
            }
            Line::Data(data) => {
                for i in &data {
                    machine_code.push(*i);
                }
                counter += data.len() as u32;
            }
        }
    }

    for (i, l) in lbl_adr {
        println!("{} {}", i, l);
    }

    machine_code
}

fn main() {
    std::env::set_var("RUST_BACKTRACE","1");

    let args : Vec<String> = std::env::args().collect();
    let fdata = std::fs::read_to_string(&args[1]);
    match fdata {
        Ok(data) => {
            let lines = pass1(&data);

            for (line, ln) in &lines {
                match line {
                    Line::Instr(code, args) => {println!("Instruction {0} with {1}", code.name, args)}
                    Line::Label(name) => {println!("Label `{0}`", name)}
                    Line::Data(data) => {
                        print!("Data ");
                        for i in data {
                            print!("{}, ",i);
                        }
                        println!();
                    }
                }
            }


            let data = pass2(lines);
            let hex = hex::encode(data);
            println!("{}", hex);

            let bin_res = BinaryString::from_hex(hex);
            match bin_res {
                Ok(bin) => {println!("{}", bin);}
                Err(_) => {println!("Invalid hex???");}
            }
        }
        Err(_) => {println!("File \"{0}\" not found.", &args[1])}
    }
    //println!("\"{0}\"", arg_nospace);
}
