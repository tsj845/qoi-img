use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use qoi_img::datas::*;
use qoi_img::decode::*;
use qoi_img::encode::*;
use qoi_img::dbgout::*;

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
            OpenOptions::new().write(true).create(true).truncate(true).open(&args[4]).unwrap().write_all(dbgout_convert(&buf, true).as_bytes()).unwrap();
        }
    } else if args[1] == "decode" {
        let mut f: File = File::open(&args[2]).unwrap();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        buf = gen_dqoi(&buf);
        OpenOptions::new().write(true).create(true).truncate(true).open(&args[3]).unwrap().write_all(buf.as_slice()).unwrap();
        if args.len() > 3 {
            OpenOptions::new().write(true).create(true).truncate(true).open(&args[4]).unwrap().write_all(dbgout_convert(&buf, false).as_bytes()).unwrap();
        }
    }
}