use std::env;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io;
use std::collections::HashMap;

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

type QOI_OP = (u8, u8); // expected value, bits to mask
type PIX = (u8, u8, u8, u8); // pixel representation

struct Pixels {
    bin: Vec<u8>,
    cc: usize,
    i: usize,
    pub l: usize
}

impl Pixels {
    fn new(bin: Vec<u8>, cc: usize) -> Self {
        Pixels {bin, cc, i:0, l:0}
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

enum QOI_OPS {
    RGB,
    RGBA,
    INDEX,
    DIFF,
    LUMA,
    RUN
}

impl QOI_OPS {
    fn get_mask (&self) -> QOI_OP {
        match self {
            Self::RGB => (254, 255),
            Self::RGBA => (255, 255),
            Self::INDEX => (0, 0b11000000),
            Self::DIFF => (0b01000000, 0b11000000),
            Self::LUMA => (0b10000000, 0b11000000),
            Self::RUN => (0b11000000, 0b11000000)
        }
    }
}

fn u32_to_u8arr(n: u32) -> [u8; 4] {
    u32::to_be_bytes(n)
}

fn u8arr_to_u32(arr: &[u8; 4]) -> u32 {
    u32::from_be_bytes(*arr)
}

fn gen_qoif(width: u32, height: u32, channels: u8, colorspace: u8, pixels: &mut Pixels) -> Vec<u8> {
    let mut finbytes: Vec<u8> = Vec::with_capacity(22); // must have 14-byte header + 8-byte end marker
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
    let mut i: usize = 0;
    for cpix in pixels {}
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

fn main() {
    let args : Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("insufficient arguments");
        return;
    }
    println!("{:?}", args);
    let exts = args[2].find(".").unwrap();
    if args[1] == "encode" {
        let mut f: File = File::open(&args[2]).unwrap();
        let mut x = parse_binary_to_tuple(&mut f);
        let buf = gen_qoif(x.0, x.1, x.2, x.3, &mut x.4);
        println!("{:?}", &buf);
        OpenOptions::new().write(true).open(&(args[2].as_str()[0usize..exts].to_owned()+".qoi")).unwrap().write_all(buf.as_slice()).unwrap();
    }
}