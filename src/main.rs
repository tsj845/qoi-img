use std::env;
use std::fs;
use std::io::prelude::*;
use std::io;
use std::collections::HashMap;

const MAGIC : [u8; 4] = [0x71, 0x6f, 0x69, 0x66];

fn preprocess (data : String) -> Vec<u8> {
    let mut f : Vec<u8> = Vec::new();
    let mut tmp : String = String::new();
    for item in data.split('\n') {
        if item.starts_with("#") {
            continue;
        }
        tmp += &(String::from(item) + " ");
    }
    tmp.pop();
    let refer : &str = "0123456789abcdef";
    for byte in tmp.split(" ") {
        let b : Vec<char> = byte.chars().collect();
        f.push((refer.match_indices(b[0]).collect::<Vec<_>>()[0].0 * 16 + refer.match_indices(b[1]).collect::<Vec<_>>()[0].0) as u8);
    }
    return f;
}

fn to_hextext (data : Vec<u8>) -> Vec<String> {
    let mut f : Vec<String> = Vec::new();
    let chars : [&str; 16] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f"];
    for item in data {
        let lower : u8 = item % 16;
        let upper : u8 = (item - lower) / 16;
        f.push(chars[upper as usize].to_owned() + chars[lower as usize]);
    }
    return f;
}

fn hashpx (pixel : [u8; 4]) -> u8 {
    return (pixel[0] * 3 + pixel[1] * 5 + pixel[2] * 7 + pixel[3] * 11) % 64;
}

fn encode (filename : &str) -> Result<Vec<u8>, io::Error> {
    let mut data : Vec<u8> = Vec::new();
    let mut file : fs::File = fs::File::open(filename)?;
    let mut s : String = String::new();
    let mut rb : bool = false;
    match file.read_to_string(&mut s) {Ok(_)=>{data=preprocess(s)},Err(_)=>{rb=true}};
    if rb {
        file.read_to_end(&mut data)?;
    }
    for byte in &data {
        println!("{}", byte);
    }
    let mut array : HashMap<u8, [u8; 4]> = HashMap::new();
    let mut stream : Vec<u8> = Vec::new();
    // magic bytes
    stream.extend(MAGIC);
    // other stuff
    stream.extend(&data[0..10]);
    // encode data
    let mut i : usize = 10;
    let l : usize = data.len();
    let mut prev_px : [u8; 4] = [0, 0, 0, 255];
    let mut runlen : u8 = 0;
    loop {
        if i >= l {
            break;
        }
        let bytes : [u8; 4] = [data[i], data[i+1], data[i+2], data[i+3]];
        let mut failed : bool = false;
        if bytes == prev_px {
            // increment run length
            runlen += 1;
        } else if runlen > 0 {
            // add run length byte
            let v : u8 = 0b11000000 | runlen;
            stream.push(match v == 0xff || v == 0xfe {true=>{return Err(io::Error::new(io::ErrorKind::OutOfMemory, "too many run pixels"))},_=>v});
        } else {
            // try to do index
            let hashed : u8 = hashpx(bytes);
            if array.contains_key(&hashed) {
                stream.push(0b00111111 & hashed);
            // try to do diffs
            } else if bytes[3] == prev_px[3] {
                let dr : i8 = bytes[0] as i8 - prev_px[0] as i8;
                let dg : i8 = bytes[1] as i8 - prev_px[1] as i8;
                let db : i8 = bytes[2] as i8 - prev_px[2] as i8;
                // 1 - byte diff
                if (-2..1).contains(&dr) && (-2..1).contains(&dg) && (-2..1).contains(&db) {
                    stream.push(0b01000000 | (((dr + 2) << 4) as u8 & 0b00110000) | (((dg + 2) << 2) as u8 & 0b00001100) | ((db + 2) as u8 & 0b00000011));
                // 2 - byte diff
                } else {
                    if ((dg + 32) as u8) & 0b111111 == dg as u8 && ((dr - dg + 8) as u8) & 0x0f == (dr - dg + 8) as u8 && ((db - dg + 8) as u8) & 0x0f == (db - dg + 8) as u8 {
                        stream.push(0b10000000 | (dg + 32) as u8);
                        stream.push((((dr - dg + 8) << 4) as u8 & 0xf0) | (((db - dg + 8)) as u8 & 0x0f));
                    } else {
                        failed = true;
                    }
                }
            }
            // do full rgb / rgba
            if failed {
                // do full rgba
                if bytes[3] != prev_px[3] {
                    stream.push(0xff);
                    stream.extend(bytes);
                // do full rgb
                } else {
                    stream.push(0xfe);
                    stream.extend([bytes[0], bytes[1], bytes[2]]);
                }
                // enter the pixel into the array
                array.insert(hashed, bytes);
            }
        }
        prev_px = bytes;
        i += 4;
    }
    // end marker
    stream.extend([0; 7]);
    stream.push(0x01);
    return Ok(stream);
}

fn main () {
    let args : Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("enter a file name");
        return;
    }
    let res : Result<Vec<u8>, io::Error> = encode(&args[1]);
    if res.is_err() {
        println!("ERROR: {}", res.unwrap_err());
        return;
    }
    println!("done!");
    for byte in to_hextext(res.unwrap()) {
        println!("byte: {}", byte);
    }
}