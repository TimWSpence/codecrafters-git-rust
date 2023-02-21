use std::fs;
use std::io::Cursor;
use std::io::Write;
use std::str;

use anyhow::Result;
use flate2::bufread::ZlibDecoder;
use flate2::bufread::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
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

pub fn hash_object(file: &str) -> Result<()> {
    let file = File::open(file)?;
    let len = file.metadata()?.len();
    let mut b = BufReader::new(file);
    let mut input = Vec::new();
    input.write(format!("blob {}\x00", len).as_bytes())?;
    b.read_to_end(&mut input)?;
    let mut z = ZlibEncoder::new(Cursor::new(input), Compression::fast());
    let mut buf = Vec::new();
    z.read_to_end(&mut buf)?;
    let mut hasher = Sha1::new();
    hasher.update(&buf);
    let mut sha1 = String::new();
    for byte in hasher.finalize().iter() {
        use std::fmt::Write;
        write!(&mut sha1, "{:02x}", byte)?;
    }
    let dir = &sha1[..2];
    fs::create_dir_all(format!(".git/objects/{}", dir))?;
    let filename = &sha1[2..];
    fs::write(format!(".git/objects/{}/{}", dir, filename), buf)?;
    println!("{}", sha1);
    Ok(())
}
