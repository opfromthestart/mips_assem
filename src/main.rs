use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use to_binary::BinaryString;
use crate::codes::{get_arguments, get_enc, Syntax};
use crate::tables::{get_code, InstrCode};

mod tables;
mod codes;

extern crate rev_slice;

// I don't know much about licenses, feel free to use this but you probably shouldn't.

#[derive(Clone)]
pub enum Args {
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

enum Line<'a> {
    Instr(&'a InstrCode<'a>, Args),
    Label(String),
    Data(Vec<u8>),
}

#[derive(Clone, Copy)]
enum Section {
    Text(),
    Data(),
}

pub fn rem_spaces(inp : String) -> String {
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

fn pass1(assem : &String) -> Vec<(Line,u32, Section)>
{
    let mut lines : Vec<(Line, u32, Section)> = vec!();
    let mut labels : Vec<(String,u32)> = vec!();
    let mut cur_label : Option<String> = None;
    let mut cur_section = Section::Text();
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
                        lines.push((Line::Label(lname), curline, cur_section));
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
                    if directive.eq(".asciiz") || directive.eq(".ascii") {
                        let begin_opt = dir_data.find("\"");
                        let begin = match begin_opt {
                            Some(n) => n as isize,
                            None => -1
                        };
                        let end_opt = dir_data[begin as usize + 1 as usize..].find("\"");
                        let end: isize = match end_opt {
                            Some(n) => (n as isize) + begin + (1 as isize),
                            None => -1
                        };
                        if begin == -1 || end == -1 {
                            println!("Directive .ascii(z) is missing beginning or ending parentheses\nLine {0}: {1}", curline, line);
                            continue;
                        }
                        //TODO: get this to output bytes in the right order
                        let ascii = dir_data[begin as usize+(1 as usize)..end as usize].as_bytes();
                        let mut byte_vec : Vec<u8> = Vec::new();
                        for i in ascii {
                            //print!("{},",i);
                            byte_vec.push(*i);
                        }
                        //println!();
                        if directive.eq(".asciiz") {
                            byte_vec.push(0);
                        }
                        lines.push((Line::Data(byte_vec), curline, cur_section));
                    }
                    else if directive.eq(".data") {
                        cur_section = Section::Data();
                    }
                    else if directive.eq(".data") {
                        cur_section = Section::Text();
                    }
                }
                None => {}
            }
            continue;
        }

        //println!("{}",line_nl);

        let line_nl2 = line_nl.clone();
        let code : &InstrCode = get_code(line_nl2);
        if code.code == -1
        {
            println!("Invalid instruction code found.\nLine {0}: {1}", curline, line);
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

        lines.push((Line::Instr(code, adata), curline, cur_section));
    }

    return lines;
}

pub fn parse_num(arg : &String) -> Result<i32, String> {
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

pub enum Encoding {
    // o, s, t, d, a, f
    Register(i8, i8, i8, i8, i8, i8),
    Immediate(i8, i8, i8, i16),
    Jump(i8, i32),
}

fn get_bin(enc : Encoding) -> Vec<u8> {
    match enc {
        Encoding::Register(o, s, t, d, a, f) => {
            let mut b: Vec<u8> = Vec::new();
            b.push(((o << 2) + (s >> 3)) as u8);
            b.push(((s << 5) + t) as u8);
            b.push(((d << 3)+ (a >> 2)) as u8);
            b.push(((a << 6) + f) as u8);
            b
        }
        Encoding::Immediate(o, s, t, i) => {
            //println!("{}", o);
            //println!("{}", o<<2);
            let mut b: Vec<u8> = Vec::new();
            b.push(((o<<2) + (s >> 3)) as u8);
            b.push(((s << 5) + t) as u8);
            b.push((i >> 8) as u8);
            b.push((i) as u8);
            b
        }
        Encoding::Jump(o, i) => {
            let mut b: Vec<u8> = Vec::new();
            b.push(((o<<2) + ((i >> 24) as i8)) as u8);
            b.push((i >> 16) as u8);
            b.push((i >> 8) as u8);
            b.push((i) as u8);
            b
        }
    }
}

fn pass2(lines : Vec<(Line, u32, Section)>, start_text_opt : Option<u32>) -> Vec<u8> {
    let mut lbl_adr : HashMap<String, u32> = HashMap::new();
    let mut data_lbl_adr : HashMap<String, u32> = HashMap::new();

    let start_text = match start_text_opt {
        None => {0x1000}
        Some(n) => {n}
    };
    let mut text_counter : u32 = start_text;
    let mut data_counter : u32 = start_text;

    let mut text_code = Vec::new();

    for (i, _, sect) in &lines {

        match i {
            Line::Label(name) => {
                if name.eq("START") {
                    let (counter, adrs) : (&mut u32, _)  = match sect {
                        Section::Text() => { (&mut text_counter, &mut lbl_adr) }
                        Section::Data() => { (&mut data_counter, &mut data_lbl_adr) }
                    };

                    let mut upd_adr : HashMap<String, u32> = HashMap::new();
                    {
                        for (lbl, c) in &mut *adrs {
                            upd_adr.insert(String::from(lbl), *c + 4);
                        }
                    }
                    //for (lbl, c) in upd_adr {
                    //    adrs.insert(lbl, c);
                    //}
                    *adrs = upd_adr;
                    for b in get_bin(Encoding::Jump(2, (*counter as i32) >> 2 + 1)) {
                        text_code.push(b);
                    }
                    *counter += 4;
                }
                let (counter, adrs) : (&mut u32, _)  = match sect {
                    Section::Text() => { (&mut text_counter, &mut lbl_adr) }
                    Section::Data() => { (&mut data_counter, &mut data_lbl_adr) }
                };
                (*adrs).insert(String::from(name), *counter);
            }
            Line::Data(v) => {
                let counter  = match sect {
                    Section::Text() => { &mut text_counter }
                    Section::Data() => { &mut data_counter }
                };
                *counter += v.len() as u32;
            }
            Line::Instr(_, _) => {
                let counter  = match sect {
                    Section::Text() => { &mut text_counter }
                    Section::Data() => { &mut data_counter }
                };

                //println!("{1}:{0}", inst.name, counter);
                *counter += 4;
            }
        }
    }
    for (lbl, c) in data_lbl_adr {
        lbl_adr.insert(lbl, text_counter + c);
    }

    /*
    for (lbl, c) in &lbl_adr {
        println!("{}:{}", lbl, c);
    }

     */

    let mut data_code = Vec::new();

    for (i, ln, sect) in &lines {
        let (machine_code , counter) : (&mut Vec<u8>, _) = match sect {
            Section::Text() => {
                let t : u32 = text_code.len() as u32+start_text;
                (&mut text_code, t)}
            Section::Data() => {
                let t : u32 = data_code.len() as u32+text_counter;
                (&mut data_code, t)}
        };
        match i {
            Line::Instr(instr, args) => {
                //println!("{}:{}", counter, instr.name);
                let enc = get_enc(instr, args.clone(), &lbl_adr, *ln, counter);
                /*
                match enc {
                    Encoding::Register(o, s, t, d, a, f) => {println!("{},{},{},{},{},{}", o,s,t,d,a,f);}
                    Encoding::Immediate(o, s, t, i) => {println!("{},{},{},{}",o, s,t,i);}
                    Encoding::Jump(o, i) => {println!("{},{}", o, i);}
                }
                 */
                let data = get_bin(enc);
                for i in data {
                    machine_code.push(i)
                }
            }
            Line::Data(data) => {
                for i in data {
                    machine_code.push(*i);
                }
            }
            _ => {}
        }
    }

    /*
    for (i, l) in lbl_adr {
        println!("{} {}", i, l);
    }
     */

    for i in data_code {
        text_code.push(i);
    }
    text_code
}

// Tested on own code as well as samples from:
// https://ecs-network.serv.pacific.edu/ecpe-170/tutorials/mips-example-programs
// https://github.com/ffcabbar/MIPS-Assembly-Language-Examples

// Errors:
fn main() {
    std::env::set_var("RUST_BACKTRACE","1");

    let args : Vec<String> = std::env::args().collect();
    let fdata = std::fs::read_to_string(&args[1]);
    match fdata {
        Ok(data) => {
            let lines = pass1(&data);

            /*
            for (line, ln, sect) in &lines {
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
             */


            let data = pass2(lines, Some(0x400000));

            let hex = hex::encode(data);

            /*
            let mut i = 0;
            while i < hex.len() - 8 {
                println!("{}", &hex[i..i+8]);
                i += 8;
            }
            println!("{}", &hex[i..]);
             */

            let bin_res = BinaryString::from_hex(&hex);

            match bin_res {
                Ok(bin) => {
                    let out = String::from({
                        if args.len() == 4 && args[2].eq("-o") {
                            &args[3]
                        } else {
                            let dot_find: Option<usize> = args[1].rfind(".");
                            let slash_find: Option<usize> = args[1].rfind("/");
                            let ln = args[1].len();
                            match dot_find {
                                None => { &args[1] }
                                Some(n) => {
                                    match slash_find {
                                        None => {
                                            &args[1][..ln - n]
                                        }
                                        Some(n2) => {
                                            if n > n2 {
                                                &args[1]
                                            } else {
                                                &args[1][..ln - n]
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    });

                    let mut hex_lined = "".to_string();
                    let mut i = 0;
                    while i < hex.len() - 8 {
                        hex_lined = hex_lined + &hex[i..i+8] + "\n";
                        i += 8;
                    }
                    hex_lined = hex_lined + &hex[i..];

                    match std::fs::write(format!("{}{}",out, ".ho".to_string()), hex_lined) {
                        Ok(_) => {
                            match std::fs::write(format!("{}{}", out, ".bo".to_string()), bin.to_string()) {
                                Ok(_) => {println!("Results written to {0}.bo and {0}.ho", out);}
                                Err(e) => {println!("Could not write binary to file, {}",e)}
                            }
                        }
                        Err(e) => {println!("Could not write hex to file, {}", e)}
                    }
                }
                Err(_) => { println!("Invalid hex???"); }
            }
        }
        Err(_) => {println!("File \"{0}\" not found.", &args[1])}
    }
    //println!("\"{0}\"", arg_nospace);
}
