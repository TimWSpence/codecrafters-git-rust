#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;

use flate2::bufread::ZlibDecoder;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

fn main() {
    // Uncomment this block to pass the first stage
    let args: Vec<String> = env::args().collect();
    if args[1] == "init" {
        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
        println!("Initialized git directory")
    } else if args[1] == "cat-file" {
        if args[2] != "-p" {
            println!("Incorrect arg format");
        } else {
            let sha = &args[3];
            let dir = &sha[..2];
            let rest = &sha[2..];
            let f = File::open(format!(".git/objects/{}/{}", dir, rest)).unwrap();
            let reader = BufReader::new(f);
            let mut z = ZlibDecoder::new(reader);
            let mut s = String::new();
            z.read_to_string(&mut s).unwrap();
            print!("{}", s);
        }
    } else {
        println!("unknown command: {}", args[1])
    }
}
