use crate::datas::*;

pub fn dbgout_convert(dat: &Vec<u8>, oflag: bool) -> String {
    let l: usize = dat.len();
    let mut sres: String = String::with_capacity(l);
    let mut i: usize = 0;
    let mut j: usize = 0;
    let mut buf: [u8; 4] = [0; 4];
    // let w: usize = 12usize;
    let w: usize = 4usize;
    if oflag {
        // magic bytes
        while j < 4 {
            sres.push(HEXVALS[(dat[j] >> 4) as usize]);
            sres.push(HEXVALS[(dat[j] & 0xf) as usize]);
            sres.push(' ');
            j += 1;
        }
        sres.push_str(&format!("( {} )\n", String::from_utf8((&dat[0..4]).into_iter().map(|v|*v).collect()).unwrap()));
        j = 0;
        i += 4;
    }
    // width
    while j < 4 {
        sres.push(HEXVALS[(dat[i+j] >> 4) as usize]);
        sres.push(HEXVALS[(dat[i+j] & 0xf) as usize]);
        sres.push(' ');
        j += 1;
    }
    mv_un_tosized_buf(&dat[i..(i+4)], &mut buf);
    sres.push_str(&format!("( {} )\n", u8arr_to_u32(&buf)));
    i += j;
    j = 0;
    // height
    while j < 4 {
        sres.push(HEXVALS[(dat[i+j] >> 4) as usize]);
        sres.push(HEXVALS[(dat[i+j] & 0xf) as usize]);
        sres.push(' ');
        j += 1;
    }
    mv_un_tosized_buf(&dat[i..(i+4)], &mut buf);
    sres.push_str(&format!("( {} )\n", u8arr_to_u32(&buf)));
    i += j;
    j = 0;
    while j < 2 {
        sres.push(HEXVALS[(dat[i+j] >> 4) as usize]);
        sres.push(HEXVALS[(dat[i+j] & 0xf) as usize]);
        sres.push(' ');
        j += 1;
    }
    i += j;
    sres.push_str(&format!("( {}, {} )\n", dat[i-2], dat[i-1]));
    let pl = l - 8;
    if oflag {
        loop {
            if i >= pl {
                break;
            }
            sres.push(HEXVALS[(dat[i] >> 4) as usize]);
            sres.push(HEXVALS[(dat[i] & 0xf) as usize]);
            sres.push(' ');
            match QoiOps::get_op(dat[i]) as u8 {
                0 => {
                    i += 1;
                    for k in 0..3 {
                        sres.push(HEXVALS[(dat[i+k] >> 4) as usize]);
                        sres.push(HEXVALS[(dat[i+k] & 0xf) as usize]);
                        sres.push(' ');
                    }
                    sres.push_str(&format!("( QoiOps::RGB {} {} {} )\n", dat[i], dat[i+1], dat[i+2]));
                    i += 2;
                },
                1 => {
                    i += 1;
                    for k in 0..4 {
                        sres.push(HEXVALS[(dat[i+k] >> 4) as usize]);
                        sres.push(HEXVALS[(dat[i+k] & 0xf) as usize]);
                        sres.push(' ');
                    }
                    sres.push_str(&format!("( QoiOps::RGBA {} {} {} {} )\n", dat[i], dat[i+1], dat[i+2], dat[i+3]));
                    i += 3;
                },
                2 => {
                    sres.push_str(&format!("( QoiOps::INDEX {} )\n", dat[i]&(!(0b11<<6))));
                },
                3 => {
                    sres.push_str(&format!("( QoiOps::DIFF {} {} {} )\n", ((dat[i]>>4)&3)as i16 - 2, ((dat[i]>>2)&3)as i16 - 2, (dat[i]&3)as i16 - 2));
                },
                4 => {
                    sres.push(HEXVALS[(dat[i+1] >> 4) as usize]);
                    sres.push(HEXVALS[(dat[i+1] & 0xf) as usize]);
                    let gd = ((dat[i]&0b00111111)as i16) - 32;
                    sres.push_str(&format!(" ( QoiOps::LUMA {} {} {} )\n", (dat[i+1]>>4)as i16 - 8 + gd, gd, (dat[i+1]&0xf)as i16 - 8 + gd));
                    i += 1;
                },
                5 => {
                    sres.push_str(&format!("( QoiOps::RUN {} )\n", dat[i]&(!(0b11<<6))));
                },
                _ => {}
            };
            i += 1;
        }
        sres.push_str("00 00 00 00 00 00 00 00 ( QOI END MARKER )\n");
        return sres;
    }
    sres.push(HEXVALS[(dat[i] >> 4) as usize]);
    sres.push(HEXVALS[(dat[i] & 0xf) as usize]);
    sres.push(' ');
    i += 1;
    println!("{i}, {w}");
    let offset = match oflag {true=>14,_=>10};
    loop {
        if ((i - offset) % w) == 0 {
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