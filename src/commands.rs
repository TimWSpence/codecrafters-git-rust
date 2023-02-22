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
    let mut buf = Vec::new();
    read_digest(digest, &mut buf)?;
    // Split object at first null byte to strip length header
    let s = strip_header(&buf);
    let content = str::from_utf8(s)?;
    print!("{}", content);
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
    let digest = compute_digest(&buf)?;
    write_digest(&digest, &mut buf)?;
    println!("{}", digest);
    Ok(())
}

pub fn ls_tree(digest: &str) -> Result<()> {
    let mut buf = Vec::new();
    read_digest(digest, &mut buf)?;
    // let s = strip_header(&buf)?;
    // for line in s.lines() {
    //     let mut sections = line.split(" ");
    //     let _mode = sections.next().unwrap();
    //     let name = sections.next().unwrap();
    //     println!("{}", name);
    // }
    Ok(())
}

fn read_digest(digest: &str, buf: &mut Vec<u8>) -> Result<()> {
    let dir = &digest[..2];
    let file = &digest[2..];
    let f = File::open(format!(".git/objects/{}/{}", dir, file))?;
    let reader = BufReader::new(f);
    let mut z = ZlibDecoder::new(reader);
    z.read_to_end(buf)?;
    Ok(())
}

fn write_digest(digest: &str, buf: &Vec<u8>) -> Result<()> {
    let dir = &digest[..2];
    let file = &digest[2..];
    fs::create_dir_all(format!(".git/objects/{}", dir))?;
    fs::write(format!(".git/objects/{}/{}", dir, file), buf)?;
    Ok(())
}

fn compute_digest(buf: &Vec<u8>) -> Result<String> {
    let mut hasher = Sha1::new();
    hasher.update(&buf);
    let mut digest = String::new();
    for byte in hasher.finalize().iter() {
        use std::fmt::Write;
        write!(&mut digest, "{:02x}", byte)?;
    }
    Ok(digest)
}

fn strip_header(buf: &Vec<u8>) -> &[u8] {
    let null_byte = buf.iter().position(|b| *b == 0x0).unwrap();
    println!("Null byte: {}", null_byte);
    &buf[(null_byte + 1)..]
}

fn parse_tree(buf: &[u8]) -> Result<Vec<TreeEntry>> {
    Ok(Vec::new())
}

struct TreeEntry<'a> {
    //Don't care to interpret this
    mode: &'a str,
    name: &'a str,
    digest: &'a [u8],
}
