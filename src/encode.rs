use crate::datas::*;

pub fn gen_qoif(width: u32, height: u32, channels: u8, colorspace: u8, pixels: &mut Pixels) -> Vec<u8> {
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
    pixarr[get_apos(lpix)] = lpix;
    let mut runcount: usize = 0;
    let mut f = false;
    loop {
        let cpix = match pixels.next(){Some(x)=>x,None=>{f=true;(255-lpix.0,0,0,0)}};
        if pix_eq(cpix, lpix) {
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
            if pix_eq(pixarr[ind], cpix) {
                finbytes.push(ind as u8);
                lpix = cpix;
                continue;
            } else {
                pixarr[ind] = cpix;
            }
            if cpix.3 == lpix.3 {
                if test_for_diff(cpix, lpix) {
                    finbytes.push((1 << 6) | (get_diff_val(lpix.0, cpix.0) << 4) | (get_diff_val(lpix.1, cpix.1) << 2) | get_diff_val(lpix.2, cpix.2));
                    lpix = cpix;
                    continue;
                }
                let lumas = get_lumas(lpix, cpix);
                if lumas.0 != 64 {
                    finbytes.push((1 << 7) | lumas.1);
                    finbytes.push((lumas.0 << 4) | lumas.2);
                    lpix = cpix;
                    continue;
                }
                finbytes.push(254);
                finbytes.push(cpix.0);
                finbytes.push(cpix.1);
                finbytes.push(cpix.2);
                lpix = cpix;
            } else {
                println!("ADDING RGBA");
                finbytes.push(255);
                finbytes.push(cpix.0);
                finbytes.push(cpix.1);
                finbytes.push(cpix.2);
                finbytes.push(cpix.3);
                lpix = cpix;
            }
        }
    }
    finbytes.extend(std::iter::repeat(0).take(8));
    return finbytes;
}