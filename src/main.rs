#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::str;

use anyhow::Result;
use flate2::bufread::ZlibDecoder;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

fn main() -> Result<()> {
    // Uncomment this block to pass the first stage
    let args: Vec<String> = env::args().collect();
    if args[1] == "init" {
        fs::create_dir(".git")?;
        fs::create_dir(".git/objects")?;
        fs::create_dir(".git/refs")?;
        fs::write(".git/HEAD", "ref: refs/heads/master\n")?;
        println!("Initialized git directory");
        Ok(())
    } else if args[1] == "cat-file" {
        if args[2] != "-p" {
            println!("Incorrect arg format");
            Ok(())
        } else {
            let sha = &args[3];
            let dir = &sha[..2];
            let rest = &sha[2..];
            let f = File::open(format!(".git/objects/{}/{}", dir, rest))?;
            let reader = BufReader::new(f);
            let mut z = ZlibDecoder::new(reader);
            let mut buf = Vec::new();
            z.read_to_end(&mut buf)?;
            // Split object at first null byte to strip length header
            let null_byte = buf.iter().position(|b| *b == 0x0).unwrap();
            let s = str::from_utf8(&buf[null_byte..])?;
            print!("{}", s);
            Ok(())
        }
    } else {
        println!("unknown command: {}", args[1]);
        Ok(())
    }
}
