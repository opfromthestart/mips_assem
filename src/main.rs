use std::collections::HashMap;

use to_binary::BinaryString;

use crate::codes::{Arg, Args, get_arguments2, get_enc2, Syntax};
use crate::tables::{get_code, InstrCode};

mod tables;
mod codes;

extern crate rev_slice;

// I don't know much about licenses, feel free to use this but you probably shouldn't.

enum Line<'a> {
    Instr(&'a InstrCode<'a>, Args<Arg>),
    Label(String),
    Data(Vec<u8>),
}

#[derive(Clone, Copy)]
enum Section {
    Text(),
    Data(),
}

pub fn rem_spaces<S: Into<String>>(inp_s : S) -> String {
    let inp = inp_s.into();

    if inp.len() == 0 {
        return String::from("");
    }
    let mut begin : usize = 0;
    let mut end : usize = inp.len();
    while inp[begin..begin+(1 as usize)].eq(" ") || inp[begin..begin+(1 as usize)].eq("\t") {
        begin+=1;
        if begin == end {
            return String::from("");
        }
    }
    while inp[end-1..end].eq(" ") || inp[end-1..end].eq("\t") {
        end-=1;
    }
    //let sub : &'a str = &inp[begin..end];
    //let ret : String = String::from(sub);
    return String::from(&inp[begin..end]);
}

fn pass1(assem : &String, start_text_opt : Option<u32>) -> (Vec<(Line,u32, Section)>, HashMap<String, u32>, u32, u32)
{
    let mut lines : Vec<(Line, u32, Section)> = vec!();
    let mut labels : Vec<(String, u32)> = vec!();

    let mut lbl_adr : HashMap<String, u32> = HashMap::new();
    let mut data_lbl_adr : HashMap<String, u32> = HashMap::new();

    let mut cur_label : Option<String> = None;
    let mut cur_section = Section::Text();

    let start_text = match start_text_opt {
        None => {0x1000}
        Some(n) => {n}
    };
    let mut text_counter : u32 = start_text;
    let mut data_counter : u32 = start_text;

    let mut curline = 0;
    for line in assem.lines()
    {
        curline += 1;

        let (counter, adrs) : (&mut u32, _)  = match cur_section {
            Section::Text() => { (&mut text_counter, &mut lbl_adr) }
            Section::Data() => { (&mut data_counter, &mut data_lbl_adr) }
        };

        //println!("{}",curline);

        let line_nc_dirty = { // Removes comments, which start with #
            let pos_opt: Option<usize> = line.find('#');
            match pos_opt
            {
                Some(n) => &line[0..n],
                None => line
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
                    let lname = &line_nc[0..pos];

                    if let Some(_) = lname.find(" ") {
                        println!("Invalid label name \"{0}\" found.\nLine {1}: {2}", lname, curline, line);
                    }

                    if {
                        let mut fail = false;
                        for (label, lline) in &labels {
                            if (*label).eq(&lname) {
                                println!("Duplicate label {0} found.\nFirst was found at {1} and second was found at {2}.", lname, lline, curline);
                                fail = true;
                                break;
                            }
                        }
                        !fail
                    } {
                        if lname.eq("START") {
                            let mut upd_adr: HashMap<String, u32> = HashMap::new();
                            {
                                for (lbl, c) in &mut *adrs {
                                    upd_adr.insert(lbl.into(), *c + 4);
                                }
                            }
                            //for (lbl, c) in upd_adr {
                            //    adrs.insert(lbl, c);
                            //}
                            *adrs = upd_adr;
                        }
                        lines.push((Line::Label(lname.into()), curline, cur_section));
                        labels.push((lname.into(), curline));
                        adrs.insert(line_nc[0..pos].into(), *counter);
                        cur_label = Some(line_nc[0..pos].into());
                    }
                    rem_spaces(&line_nc[pos + (1 as usize)..])
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
                    let directive = &line_nl[0..n];
                    let dir_data = &line_nl[n..];
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
                        *counter += byte_vec.len() as u32;
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

        let code : &InstrCode = get_code(&line_nl);
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

        let adata : Args<Arg> = get_arguments2(line_args);

        lines.push((Line::Instr(code, adata), curline, cur_section));

        *counter += 4;
    }

    for (lbl, c) in data_lbl_adr {
        lbl_adr.insert(lbl, text_counter + c);
    }

    return (lines, lbl_adr, start_text, text_counter);
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

fn pass2(lines : Vec<(Line, u32, Section)>, lbl_adr : &HashMap<String, u32>, start_text : u32, text_counter : u32) -> Vec<u8> {

    let mut text_code = Vec::new();
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
                let enc = get_enc2(instr, args.clone(), &lbl_adr, *ln, counter);
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
    if args.len() < 2 {
        println!("No parameters given, needs 1 or 3");
        println!("Usage:    assembler_rust file");
        println!("          assembler_rust file -o outfile");
        return;
    }
    let fdata = std::fs::read_to_string(&args[1]);

    match fdata {
        Ok(data) => {
            let (lines, lbls, start, text) = pass1(&data, Some(0x400000));

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


            let data = pass2(lines, &lbls, start, text);

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
