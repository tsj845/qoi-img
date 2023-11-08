use crate::datas::*;

pub fn gen_dqoi(buf: &Vec<u8>) -> Vec<u8> {
    let mut finbytes: Vec<u8> = Vec::with_capacity(22);
    finbytes.extend_from_slice(&buf[4..14]);
    let mut lpix: PIX = (0, 0, 0, 255);
    let mut pixarr: [PIX; 64] = [(0,0,0,0); 64];
    pixarr[get_apos(lpix)] = lpix;
    let l = buf.len();
    let pl = l - 8;
    let mut i: usize = 14usize;
    while i < pl {
        let oc = QoiOps::get_op(buf[i]);
        // println!("{}, {:?}", buf[i], oc);
        match oc as u8 {
            0 => {
                lpix = (buf[i+1],buf[i+2],buf[i+3],lpix.3);
                pixarr[get_apos(lpix)] = lpix;
                i += 3;
                finbytes.extend_from_slice(&[lpix.0,lpix.1,lpix.2,lpix.3]);
            },
            1 => {
                lpix = (buf[i+1],buf[i+2],buf[i+3],buf[i+4]);
                pixarr[get_apos(lpix)] = lpix;
                i += 4;
                finbytes.extend_from_slice(&[lpix.0,lpix.1,lpix.2,lpix.3]);
            },
            2 => {
                lpix = pixarr[(buf[i]&63) as usize];
                finbytes.extend_from_slice(&[lpix.0,lpix.1,lpix.2,lpix.3]);
            },
            3 => {
                let c: PIX = (addsub_with_wrap(lpix.0, ((buf[i]>>4)&3) as i16 - 2), addsub_with_wrap(lpix.1, ((buf[1]>>2)&3) as i16 - 2), addsub_with_wrap(lpix.2, (buf[i]&3) as i16 - 2), lpix.3);
                lpix = c;
                pixarr[get_apos(lpix)] = lpix;
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