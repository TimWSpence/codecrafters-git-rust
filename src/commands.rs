use std::fs;
use std::str;

use anyhow::Result;
use flate2::bufread::ZlibDecoder;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

pub fn init() -> Result<()> {
    fs::create_dir(".git")?;
    fs::create_dir(".git/objects")?;
    fs::create_dir(".git/refs")?;
    fs::write(".git/HEAD", "ref: refs/heads/master\n")?;
    println!("Initialized git directory");
    Ok(())
}

pub fn cat_file(digest: &String) -> Result<()> {
    let dir = &digest[..2];
    let rest = &digest[2..];
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
