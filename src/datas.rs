use std::{fs::File, io::Read};

pub(crate) const MAGIC : [u8; 4] = [0x71, 0x6f, 0x69, 0x66];

pub(crate) static HEXVALS : [char; 16] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];

pub type PIX = (u8, u8, u8, u8); // pixel representation

#[derive(Clone)]
pub struct Pixels {
    bin: Vec<u8>,
    cc: usize,
    i: usize,
    pub l: usize
}

impl Pixels {
    pub fn new(bin: Vec<u8>, cc: usize) -> Self {
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
pub enum QoiOps {
    RGB = 0,
    RGBA = 1,
    INDEX = 2,
    DIFF = 3,
    LUMA = 4,
    RUN = 5
}

impl QoiOps {
    pub fn get_op(b: u8) -> Self {
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

pub fn pix_eq(p1: PIX, p2: PIX) -> bool {
    return p1.0 == p2.0 && p1.1 == p2.1 && p1.2 == p2.2 && p1.3 == p2.3;
}

pub(crate) fn u32_to_u8arr(n: u32) -> [u8; 4] {
    u32::to_be_bytes(n)
}

pub(crate) fn u8arr_to_u32(arr: &[u8; 4]) -> u32 {
    u32::from_be_bytes(*arr)
}

pub(crate) fn mv_un_tosized_buf(unbuf: &[u8], buf: &mut [u8; 4]) {
    buf[0] = unbuf[0];
    buf[1] = unbuf[1];
    buf[2] = unbuf[2];
    buf[3] = unbuf[3];
}

pub(crate) fn get_apos(p: PIX) -> usize {
    return (p.0 as usize * 3 + p.1 as usize * 5 + p.2 as usize * 7 + p.3 as usize * 11) % 64;
}

pub fn parse_binary_to_tuple(f: &mut File) -> (u32, u32, u8, u8, Pixels) {
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

pub(crate) fn addsub_with_wrap(n1: u8, n2: i16) -> u8 {
    let mut r = n1 as i16 + n2;
    if r > 255 {
        r -= 256;
    }
    if r < 0 {
        r += 256;
    }
    return r as u8;
}

fn dif_test_helper(n1: u8, n2: u8) -> bool {
    if n1 == n2 {return true;}
    if addsub_with_wrap(n1, -2) == n2 || addsub_with_wrap(n1, -1) == n2 || addsub_with_wrap(n1, 1) == n2 {
        return true;
    }
    return false;
}

pub(crate) fn get_diff_val(n1: u8, n2: u8) -> u8 {
    if n1 == n2 {return 2;}
    if addsub_with_wrap(n1, -2) == n2 {
        return 0;
    }
    if addsub_with_wrap(n1, -1) == n2 {
        return 1;
    }
    if addsub_with_wrap(n1, 1) == n2 {
        return 3;
    }
    return 4;
}

pub(crate) fn test_for_diff(p1: PIX, p2: PIX) -> bool {
    return dif_test_helper(p1.0, p2.0) && dif_test_helper(p1.1, p2.1) && dif_test_helper(p1.2, p2.2);
}

pub(crate) fn get_lumas(p1: PIX, p2: PIX) -> (u8, u8, u8) {
    let dg: i16 = p2.1 as i16 - p1.1 as i16;
    let dr: i16 = (p2.0 as i16 - p1.0 as i16) - dg;
    let db: i16 = (p2.2 as i16 - p1.2 as i16) - dg;
    if (dg < -32 || dg > 31) || (dr < -8 || dr > 7) || (db < -8 || db > 7) {
        return (64, 64, 64);
    }
    return ((dr+8)as u8, (dg+32)as u8, (db+8)as u8);
}