use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;

/*
qoi_header {
    char magic[4]; // magic bytes "qoif"
    uint32_t width; // image width in pixels (BE)
    uint32_t height; // image height in pixels (BE)
    uint8_t channels; // 3 = RGB, 4 = RGBA
    uint8_t colorspace; // 0 = sRGB with linear alpha
// 1 = all channels linear
};
*/

const MAGIC : [u8; 4] = [0x71, 0x6f, 0x69, 0x66];

static HEXVALS : [char; 16] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];

// type QoiOp = (u8, u8); // expected value, bits to mask
type PIX = (u8, u8, u8, u8); // pixel representation

struct Pixels {
    bin: Vec<u8>,
    cc: usize,
    i: usize,
    pub l: usize
}

impl Pixels {
    fn new(bin: Vec<u8>, cc: usize) -> Self {
        let l = bin.len();
        Pixels {bin, cc, i:0, l}
    }
}

impl Iterator for Pixels {
    type Item = PIX;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.l {
            return None;
        }
        let p: PIX = (self.bin[self.i], self.bin[self.i+1], self.bin[self.i+2], match self.cc==3{true=>255,_=>self.bin[self.i+3]});
        self.i += self.cc;
        return Some(p);
    }
}

#[derive(Debug)]
enum QoiOps {
    RGB = 0,
    RGBA = 1,
    INDEX = 2,
    DIFF = 3,
    LUMA = 4,
    RUN = 5
}

impl QoiOps {
    // fn get_mask(&self) -> QoiOp {
    //     match self {
    //         Self::RGB => (254, 255),
    //         Self::RGBA => (255, 255),
    //         Self::INDEX => (0, 0b11000000),
    //         Self::DIFF => (0b01000000, 0b11000000),
    //         Self::LUMA => (0b10000000, 0b11000000),
    //         Self::RUN => (0b11000000, 0b11000000)
    //     }
    // }
    fn get_op(b: u8) -> Self {
        if b == 254 {return Self::RGB}
        if b == 255 {return Self::RGBA}
        let c = (b & 192) >> 6;
        match c {
            0 => Self::INDEX,
            1 => Self::DIFF,
            2 => Self::LUMA,
            3 => Self::RUN,
            _ => {unreachable!()}
        }
    }
}

fn u32_to_u8arr(n: u32) -> [u8; 4] {
    u32::to_be_bytes(n)
}

fn u8arr_to_u32(arr: &[u8; 4]) -> u32 {
    u32::from_be_bytes(*arr)
}

fn mv_un_tosized_buf(unbuf: &[u8], buf: &mut [u8; 4]) {
    buf[0] = unbuf[0];
    buf[1] = unbuf[1];
    buf[2] = unbuf[2];
    buf[3] = unbuf[3];
}

fn get_apos(p: PIX) -> usize {
    return (p.0 as usize * 3 + p.1 as usize * 5 + p.2 as usize * 7 + p.3 as usize * 11) % 64;
}

fn gen_dqoi(buf: &Vec<u8>) -> Vec<u8> {
    let mut finbytes: Vec<u8> = Vec::with_capacity(22);
    finbytes.extend_from_slice(&buf[4..14]);
    let mut lpix: PIX = (0, 0, 0, 255);
    let mut pixarr: [PIX; 64] = [(0,0,0,0); 64];
    let l = buf.len();
    let pl = l - 8;
    let mut i: usize = 14usize;
    while i < pl {
        let oc = QoiOps::get_op(buf[i]);
        println!("{}, {:?}", buf[i], oc);
        match oc as u8 {
            0 => {
                lpix = (buf[i+1],buf[i+2],buf[i+3],lpix.3);
                pixarr[get_apos(lpix)] = lpix;
                println!("{:?}", lpix);
                i += 4;
                finbytes.extend_from_slice(&[lpix.0,lpix.1,lpix.2,lpix.3]);
            },
            1 => {
                lpix = (buf[i+1],buf[i+2],buf[i+3],buf[i+4]);
                pixarr[get_apos(lpix)] = lpix;
                println!("{:?}", lpix);
                i += 4;
                finbytes.extend_from_slice(&[lpix.0,lpix.1,lpix.2,lpix.3]);
            },
            2 => {
                lpix = pixarr[(buf[i]&63) as usize];
                finbytes.extend_from_slice(&[lpix.0,lpix.1,lpix.2,lpix.3]);
            },
            5 => {
                let s = &[lpix.0,lpix.1,lpix.2,lpix.3];
                for _ in 0..(buf[i]&63) {
                    finbytes.extend_from_slice(s);
                }
            },
            _ => {unreachable!()}
        };
        i += 1;
    }
    return finbytes;
}

fn gen_qoif(width: u32, height: u32, channels: u8, colorspace: u8, pixels: &mut Pixels) -> Vec<u8> {
    let mut finbytes: Vec<u8> = Vec::with_capacity(10); // must have 10-byte header
    finbytes.extend_from_slice(&MAGIC); // do the magic bytes
    finbytes.extend_from_slice(&u32_to_u8arr(width)); // dimensions
    finbytes.extend_from_slice(&u32_to_u8arr(height));
    finbytes.extend_from_slice(&[channels, colorspace]); // pass through
    let mut pixarr: [PIX; 64] = [(0,0,0,0); 64];
    let fbl: usize = pixels.l;
    if fbl % channels as usize != 0 {
        panic!("BINARY DATA NOT ALIGNED");
    }
    let mut lpix: PIX = (0, 0, 0, 255);
    // let mut cpix: PIX;
    let mut runcount: usize = 0;
    // let mut i: usize = 0;
    let mut f = false;
    loop {
        let cpix = match pixels.next(){Some(x)=>x,None=>{f=true;(255-lpix.0,0,0,0)}};
        if cpix == lpix {
            runcount += 1
        } else {
            if runcount > 0 {
                while runcount > 62 {
                    finbytes.push(0b11111101);
                    runcount -= 62;
                }
                finbytes.push(0b11000000 | runcount as u8);
                runcount = 0;
            }
            if f {break;}
            let ind = get_apos(cpix);
            if pixarr[ind] == cpix {
                finbytes.push(ind as u8);
                lpix = cpix;
                continue;
            } else {
                pixarr[ind] = cpix;
            }
            if cpix.3 == lpix.3 {
                finbytes.push(254);
                finbytes.push(cpix.0);
                finbytes.push(cpix.1);
                finbytes.push(cpix.2);
                finbytes.push(lpix.3);
                lpix = cpix;
                continue;
            }
            lpix = cpix;
            finbytes.extend_from_slice(&[cpix.0,cpix.1,cpix.2,cpix.3]);
        }
    }
    finbytes.extend(std::iter::repeat(0).take(8));
    return finbytes;
}

fn parse_binary_to_tuple(f: &mut File) -> (u32, u32, u8, u8, Pixels) {
    let buf: &mut [u8; 4] = &mut [0; 4];
    f.read_exact(buf).unwrap();
    let width: u32 = u8arr_to_u32(buf);
    f.read_exact(buf).unwrap();
    let height: u32 = u8arr_to_u32(buf);
    let buf: &mut [u8; 2] = &mut [0; 2];
    f.read_exact(buf).unwrap();
    let mut bin: Vec<u8> = Vec::new();
    f.read_to_end(&mut bin).unwrap();
    return (width, height, buf[0], buf[1], Pixels::new(bin, buf[0] as usize));
}

fn dbgout_convert(dat: &Vec<u8>) -> String {
    let l: usize = dat.len();
    let mut sres: String = String::with_capacity(l);
    let mut i: usize = 0;
    let mut buf: [u8; 4] = [0; 4];
    // let w: usize = 12usize;
    let w: usize = 1usize;
    // magic bits
    while i < 4 {
        sres.push(HEXVALS[(dat[i] >> 4) as usize]);
        sres.push(HEXVALS[(dat[i] & 0xf) as usize]);
        sres.push(' ');
        i += 1;
    }
    sres.push_str(&format!("( {} )\n", String::from_utf8((&dat[0..4]).into_iter().map(|v|*v).collect()).unwrap()));
    // width
    while i < 8 {
        sres.push(HEXVALS[(dat[i] >> 4) as usize]);
        sres.push(HEXVALS[(dat[i] & 0xf) as usize]);
        sres.push(' ');
        i += 1;
    }
    mv_un_tosized_buf(&dat[4..8], &mut buf);
    sres.push_str(&format!("( {} )\n", u8arr_to_u32(&buf)));
    // height
    while i < 12 {
        sres.push(HEXVALS[(dat[i] >> 4) as usize]);
        sres.push(HEXVALS[(dat[i] & 0xf) as usize]);
        sres.push(' ');
        i += 1;
    }
    mv_un_tosized_buf(&dat[8..12], &mut buf);
    sres.push_str(&format!("( {} )\n", u8arr_to_u32(&buf)));
    while i < 14 {
        sres.push(HEXVALS[(dat[i] >> 4) as usize]);
        sres.push(HEXVALS[(dat[i] & 0xf) as usize]);
        sres.push(' ');
        i += 1;
    }
    sres.push_str(&format!("( {}, {} )\n", dat[i-2], dat[i-1]));
    sres.push(HEXVALS[(dat[i] >> 4) as usize]);
    sres.push(HEXVALS[(dat[i] & 0xf) as usize]);
    sres.push(' ');
    i += 1;
    println!("{i}, {w}");
    loop {
        if ((i - 14) % w) == 0 {
            sres.push('(');
            sres.push(' ');
            for j in 0..w {
                sres.push_str(&format!("{} ", dat[i+j-w]));
            }
            sres.push(')');
            sres.push('\n');
        }
        if i >= l {
            break;
        }
        sres.push(HEXVALS[(dat[i] >> 4) as usize]);
        sres.push(HEXVALS[(dat[i] & 0xf) as usize]);
        sres.push(' ');
        i += 1;
    }
    return sres;
}

fn main() {
    let args : Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("insufficient arguments");
        return;
    }
    println!("{:?}", args);
    // let exts = args[2].find(".").unwrap();
    if args[1] == "encode" {
        let mut f: File = File::open(&args[2]).unwrap();
        let mut x = parse_binary_to_tuple(&mut f);
        let buf = gen_qoif(x.0, x.1, x.2, x.3, &mut x.4);
        println!("{:?}", &buf);
        OpenOptions::new().write(true).create(true).truncate(true).open(&args[3]).unwrap().write_all(buf.as_slice()).unwrap();
        if args.len() > 3 {
            OpenOptions::new().write(true).create(true).truncate(true).open(&args[4]).unwrap().write_all(dbgout_convert(&buf).as_bytes()).unwrap();
        }
        // OpenOptions::new().write(true).open(&(args[2].as_str()[0usize..exts].to_owned()+".qoi")).unwrap().write_all(buf.as_slice()).unwrap();
    } else if args[1] == "decode" {
        let mut f: File = File::open(&args[2]).unwrap();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        buf = gen_dqoi(&buf);
        OpenOptions::new().write(true).create(true).truncate(true).open(&args[3]).unwrap().write_all(buf.as_slice()).unwrap();
    }
}