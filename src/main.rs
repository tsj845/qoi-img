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
        tmp += &(String::from(item) + match item.ends_with(" ") {true=>"",_=>" "});
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

// fn to_bintext (data : Vec<u8>) -> Vec<String> {
//     let mut f : Vec<String> = Vec::new();
//     for byte in data {
//         let mut build : String = String::new();
//         for i in 0..7 {
//             if byte & !(1u8 << i) == byte {
//                 build.insert(0, '0');
//             } else {
//                 build.insert(0, '1');
//             }
//         }
//         f.push(build);
//     }
//     return f;
// }

fn hashpx (pixel : [u8; 4]) -> u8 {
    return ((pixel[0] as usize * 3 + pixel[1] as usize * 5 + pixel[2] as usize * 7 + pixel[3] as usize * 11) % 64) as u8;
}

fn encode (filename : &str) -> io::Result<Vec<u8>> {
    let mut data : Vec<u8> = Vec::new();
    let mut file : fs::File = fs::File::open(filename)?;
    let mut s : String = String::new();
    let mut rb : bool = false;
    match file.read_to_string(&mut s) {Ok(_)=>{data=preprocess(s)},Err(_)=>{rb=true}};
    if rb {
        file.read_to_end(&mut data)?;
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
    // println!("HASHED BLACK : {}", hashpx(prev_px));
    array.insert(hashpx(prev_px), prev_px);
    // let mut runlen : u64 = 0;
    let mut runlen : u8 = 0;
    loop {
        if i >= l {
            break;
        }
        let bytes : [u8; 4] = [data[i], data[i+1], data[i+2], data[i+3]];
        println!("BYTES: {:?}", bytes);
        let mut failed : bool = false;
        if bytes == prev_px {
            println!("INC RUN: {:?}, {:?}", bytes, prev_px);
            // increment run length
            runlen += 1;
        } else if runlen > 0 {
            println!("RUN DONE: {:?}, {:?}", bytes, prev_px);
            // runlen -= 1;
            // add run length byte
            // while runlen >= 63 {
            //     runlen -= 62;
            //     stream.push(0b11111101);
            // }
            // if runlen > 0 {
            //     stream.push((0b11 << 6) | (runlen as u8 - 1));
            // }
            let v : u8 = 0b11 << 6 | (runlen - 1);
            stream.push(match v == 0xff || v == 0xfe {true=>{return Err(io::Error::new(io::ErrorKind::OutOfMemory, "too many run pixels"))},_=>v});
            runlen = 0;
            i -= 4;
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
                    println!("1 BYTE DIFF");
                    stream.push(0b01000000 | (((dr + 2) << 4) as u8 & 0b00110000) | (((dg + 2) << 2) as u8 & 0b00001100) | ((db + 2) as u8 & 0b00000011));
                // 2 - byte diff
                } else {
                    println!("2 BYTE DIFF");
                    if ((dg + 32) as u8) & 0b111111 == dg as u8 && ((dr - dg + 8) as u8) & 0x0f == (dr - dg + 8) as u8 && ((db - dg + 8) as u8) & 0x0f == (db - dg + 8) as u8 {
                        stream.push(0b10000000 | (dg + 32) as u8);
                        stream.push((((dr - dg + 8) << 4) as u8 & 0xf0) | (((db - dg + 8)) as u8 & 0x0f));
                    } else {
                        failed = true;
                    }
                }
            } else {
                failed = true;
            }
            // do full rgb / rgba
            if failed {
                println!("INDEXING");
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
    if runlen > 0 {
        println!("RUN LAST");
        // runlen -= 1;
        // add run length byte
        // while runlen >= 63 {
        //     runlen -= 62;
        //     stream.push(0b11111101);
        // }
        // if runlen > 0 {
        //     stream.push((0b11 << 6) | (runlen as u8 - 1));
        // }
        let v : u8 = 0b11000000 | (runlen - 1);
        stream.push(match v == 0xff || v == 0xfe {true=>{return Err(io::Error::new(io::ErrorKind::OutOfMemory, "too many run pixels"))},_=>v});
    }
    // end marker
    stream.extend([0; 7]);
    stream.push(0x01);
    // println!("{:?}", stream);
    return Ok(stream);
}

fn decode (filename : &str) -> io::Result<Vec<u8>> {
    // println!("decoding");
    let mut data : Vec<u8> = Vec::new();
    let mut file : fs::File = fs::File::open(filename)?;
    let mut s : String = String::new();
    let mut rb : bool = false;
    match file.read_to_string(&mut s) {Ok(_)=>{data=preprocess(s)},Err(_)=>{rb=true}};
    if rb {
        file.read_to_end(&mut data)?;
    }
    // pop the magic bits
    for _ in 0..4 {
        data.remove(0);
    }
    let mut array : HashMap<u8, [u8; 4]> = HashMap::new();
    let mut stream : Vec<u8> = Vec::new();
    // pop width and height + other meta data
    for _ in 0..10 {
        stream.push(data.remove(0));
    }
    // println!("{:?}", stream);
    let mut i : usize = 0;
    let l : usize = data.len();
    let mut prev_px : [u8; 4] = [0, 0, 0, 255];
    array.insert(hashpx(prev_px), prev_px);
    let tooshort = Err(io::Error::new(io::ErrorKind::InvalidData, "data too short"));
    loop {
        // end marker not found before EOF, invalid data
        if i+7 >= l {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "no end marker"));
        }
        // end marker detection
        if data[i..i+8] == [0, 0, 0, 0, 0, 0, 0, 1] {
            break;
        }
        let byte : u8 = data[i];
        // set rgba data
        if byte == 0xff || byte == 0xfe {
            // rgba
            if byte == 0xff {
                if i + 4 >= l {
                    return tooshort;
                }
                prev_px = [data[i+1], data[i+2], data[i+3], data[i+4]];
                stream.extend(prev_px);
                i += 4;
            // rgb only
            } else {
                if i + 3 >= l {
                    return tooshort;
                }
                prev_px = [data[i+1], data[i+2], data[i+3], prev_px[3]];
                stream.extend(prev_px);
                i += 3;
            }
            array.insert(hashpx(prev_px), prev_px);
        // runlen byte
        } else if byte & 0b11u8 << 6 == 0b11u8 << 6 {
            for _ in 0..((byte & 0b00111111u8) + 1) {
                stream.extend(prev_px);
            }
        // array index
        } else if byte & !(0b11u8 << 6) == byte {
            let b = array.get(&byte);
            if b.is_none() {
                println!("{}", byte);
                println!("{}", hashpx([0, 255, 0, 255]));
                return Err(io::Error::new(io::ErrorKind::InvalidData, "bad index"));
            }
            prev_px = b.unwrap().clone();
            stream.extend(prev_px);
        // small diff
        } else if byte & 0b01111111u8 == byte {
            println!("SMALL DIFF");
            let dr : i16 = ((byte & (0b11u8 << 4)) >> 4) as i16 - 2;
            println!("{}", byte & (0b111u8 << 4));
            let dg : i16 = ((byte & (0b11u8 << 2)) >> 2) as i16 - 2;
            let db : i16 = (byte & 0b11u8) as i16 - 2;
            println!("{}, {}, {}", dr, dg, db);
            prev_px = [((prev_px[0] as i16 + dr) % 256) as u8, ((prev_px[1] as i16 + dg) % 256) as u8, ((prev_px[2] as i16 + db) % 256) as u8, prev_px[3]];
            stream.extend(prev_px);
        // big diff
        } else if byte & 0b10111111u8 == byte {
            if i + 1 >= l {
                return tooshort;
            }
            println!("BIG DIFF");
            let dg : i16 = (byte & 0b00111111u8) as i16 - 32;
            let byte1 : u8 = data[i+1];
            let dr : i16 = ((byte1 & 0b11110000u8) as i16 - 8) + dg;
            let db : i16 = ((byte1 & 0b00001111u8) as i16 - 8) + dg;
            println!("{}, {}, {}", dg, dr, db);
            prev_px = [((prev_px[0] as i16 + dr) % 256) as u8, ((prev_px[1] as i16 + dg) % 256) as u8, ((prev_px[2] as i16 + db) % 256) as u8, prev_px[3]];
            stream.extend(prev_px);
            i += 1;
        }
        i += 1;
    }
    return Ok(stream);
}

fn write_decoded (file : &mut fs::File, blst : Vec<u8>) {
    let bytes = to_hextext(blst.clone());
    println!("{:?}", bytes);
    let mut i : usize = 0;
    let l : usize = bytes.len();
    let mut in_header : bool = true;
    let img_w : u32 = ((blst[0] as u32) << 24) | ((blst[1] as u32) << 16) | ((blst[2] as u32) << 8) | (blst[3] as u32);
    // let img_h : u32 = ((blst[4] as u32) << 24) | ((blst[5] as u32) << 16) | ((blst[6] as u32) << 8) | (blst[7] as u32);
    let mut x : u32 = 0;
    let mut inc : usize = 1;
    loop {
        if i >= l {
            break;
        }
        if in_header {
            file.write(bytes[i].as_bytes()).unwrap();
            if i == 9 {
                in_header = false;
                file.write(b"\n").unwrap();
                i -= 3;
                inc = 4;
            } else {
                file.write(b" ").unwrap();
            }
        } else {
            file.write((bytes[i].clone() + " " + &bytes[i+1] + " " + &bytes[i+2] + " " + &bytes[i+3]).as_bytes()).unwrap();
            x += 1;
            if x >= img_w {
                x = 0;
                file.write(b"\n").unwrap();
            } else {
                file.write(b" ").unwrap();
            }
        }
        i += inc;
    }
}

fn write_encoded (file : &mut fs::File, blst : Vec<u8>) {
    let bytes : Vec<String> = to_hextext(blst);
    let mut i : usize = 0;
    let l : usize = bytes.len();
    let mut in_header : bool = true;
    loop {
        if i >= l {
            break;
        }
        file.write(bytes[i].as_bytes()).unwrap();
        file.write(b" ").unwrap();
        if in_header && i >= 13 {
            in_header = false;
            file.write(b"\n").unwrap();
        }
        i += 1;
    }
}

fn main () {
    let args : Vec<String> = env::args().collect();
    println!("{}", args[0]);
    if args.len() < 2 {
        println!("enter an operation");
        return;
    } else {
        let ops = ["encode", "decode"];
        let mut s : bool = false;
        for item in ops {
            if item == args[1] {
                s = true;
                break;
            }
        }
        if !s {
            println!("enter a valid operation");
            return;
        }
    }
    if args.len() < 3 {
        println!("enter a file name");
        return;
    }
    if args.len() < 4 {
        println!("enter a destination");
        return;
    }
    let res : Result<Vec<u8>, io::Error>;
    let oper : bool;
    if args[1] == "encode" {
        res = encode(&args[2]);
        oper = false;
    } else if args[1] == "decode" {
        res = decode(&args[2]);
        oper = true;
    } else {
        res = Err(io::Error::new(io::ErrorKind::InvalidInput, "not a valid operation"));
        oper = false;
    }
    if res.is_err() {
        println!("ERROR: {}", res.unwrap_err());
        return;
    }
    println!("done!");
    let mut file : fs::File = match fs::File::create(args[3].clone()) {Ok(x)=>x,Err(_)=>{println!("couldn't open file");return;}};
    let blst = res.unwrap();
    // println!("{:?}", blst);
    if oper {
        write_decoded(&mut file, blst);
    } else {
        write_encoded(&mut file, blst);
    }
    // println!("{}", blst.len());
    // println!("[");
    // for s in to_bintext(blst.clone()) {
    //     println!("{}", s);
    // }
    // println!("]");
    // let bytes = to_hextext(blst.clone());
    // for byte in bytes {
    //     println!("{}", byte);
        // file.write(byte.as_bytes()).unwrap();
        // file.write(b" ").unwrap();
    // }
    // for byte in to_hextext(res.unwrap()) {
    //     println!("byte: {}", byte);
    // }
}