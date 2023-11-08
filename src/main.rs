use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use qoi_img::*;

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
    if args[1] == "encode" {
        let mut f: File = File::open(&args[2]).unwrap();
        let mut x = parse_binary_to_tuple(&mut f);
        let buf = encode(x.0, x.1, x.2, x.3, &mut x.4);
        OpenOptions::new().write(true).create(true).truncate(true).open(&args[3]).unwrap().write_all(buf.as_slice()).unwrap();
        if args.len() > 3 {
            OpenOptions::new().write(true).create(true).truncate(true).open(&args[4]).unwrap().write_all(format_output(&buf, true, x.2 as usize).as_bytes()).unwrap();
        }
    } else if args[1] == "decode" {
        let mut f: File = File::open(&args[2]).unwrap();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        buf = decode(&buf);
        OpenOptions::new().write(true).create(true).truncate(true).open(&args[3]).unwrap().write_all(buf.as_slice()).unwrap();
        if args.len() > 3 {
            OpenOptions::new().write(true).create(true).truncate(true).open(&args[4]).unwrap().write_all(format_output(&buf, false, buf[8] as usize).as_bytes()).unwrap();
        }
    } else {
        println!("unrecognized option\nusage:\n    qoi-img [encode|decode] [input-file] [output-file] [debug-file]?");
    }
}