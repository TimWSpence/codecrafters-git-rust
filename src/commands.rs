use std::fs;
use std::fs::DirEntry;
use std::io::Cursor;
use std::io::Write;
use std::os::unix::prelude::MetadataExt;
use std::path::PathBuf;
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

pub fn hash_object(file: &str) -> Result<Vec<u8>> {
    let file = File::open(file)?;
    let len = file.metadata()?.len();
    let mut b = BufReader::new(file);
    let mut input = Vec::new();
    input.write(format!("blob {}\x00", len).as_bytes())?;
    b.read_to_end(&mut input)?;
    let digest = compute_digest(&input);
    let sha = format_digest(&digest)?;
    let mut z = ZlibEncoder::new(Cursor::new(input), Compression::fast());
    let mut buf = Vec::new();
    z.read_to_end(&mut buf)?;
    write_digest(&sha, &mut buf)?;
    println!("{}", sha);
    Ok(digest)
}

pub fn ls_tree(digest: &str) -> Result<()> {
    let mut buf = Vec::new();
    read_digest(digest, &mut buf)?;
    let b = strip_header(&buf);
    let entries = parse_tree(b)?;
    for entry in entries {
        println!("{}", entry.name);
    }
    Ok(())
}

pub fn write_tree() -> Result<()> {
    write_root(".")?;
    Ok(())
}

fn write_root(root: &str) -> Result<TreeEntry> {
    let mut entries: Vec<DirEntry> = fs::read_dir(root)?.map(|e| e.unwrap()).collect();
    entries.sort_by(|x, y| x.path().cmp(&y.path()));
    let mut buf = Vec::new();
    //TODO what is the length here?
    write!(&mut buf, "tree {}\x00", entries.len())?;
    for entry in entries {
        if entry.metadata().unwrap().is_file() {
            let digest = hash_object(entry.path().to_str().unwrap())?;
            let mode = entry.metadata()?.mode();
            let n = entry.file_name();
            let name = n.to_str().unwrap();
            write!(&mut buf, "1{:0o} {}\x00", mode, name)?;
            buf.append(&mut digest.to_vec());
        } else {
            if entry.metadata().unwrap().is_dir() {
                let path = entry.path();
                let t = write_root(path.to_str().unwrap())?;
                let mode = entry.metadata()?.mode();
                let n = entry.file_name();
                let name = n.to_str().unwrap();
                write!(&mut buf, "{:0o} {}\x00", mode, name)?;
                buf.append(&mut t.digest.to_vec());
            } else {
                panic!(
                    "{} is neither a file or a directory",
                    entry.path().to_str().unwrap()
                )
            }
        }
    }
    //TODO hash buf and compress and write to to filesystem
    let digest = compute_digest(&buf);
    let sha = format_digest(&digest)?;
    let mut z = ZlibEncoder::new(Cursor::new(buf), Compression::fast());
    let mut out = Vec::new();
    z.read_to_end(&mut out)?;
    write_digest(&sha, &out)?;
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

fn compute_digest(buf: &Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(&buf);
    hasher.finalize().to_vec()
}

fn format_digest(digest: &Vec<u8>) -> Result<String> {
    let mut buf = String::new();
    for byte in digest.iter() {
        use std::fmt::Write;
        write!(&mut buf, "{:02x}", byte)?;
    }
    Ok(buf)
}

fn strip_header(buf: &Vec<u8>) -> &[u8] {
    let null_byte = buf.iter().position(|b| *b == 0x0).unwrap();
    &buf[(null_byte + 1)..]
}

fn parse_tree(buf: &[u8]) -> Result<Vec<TreeEntry>> {
    fn find(target: u8, buf: &[u8]) -> usize {
        let mut idx = 0;
        for byte in &buf[idx..] {
            if *byte == target {
                break;
            }
            idx += 1;
        }
        idx
    }

    let mut idx = 0;
    let mut res = Vec::new();
    while idx < buf.len() {
        let sp = find(0x20, &buf[idx..]);
        let mode = str::from_utf8(&buf[idx..(idx + sp)])?;
        idx += sp;
        // Space separator
        assert!(buf[idx] == 0x20);
        idx += 1;
        let null_byte = find(0x00, &buf[idx..]);
        let name = str::from_utf8(&buf[idx..(idx + null_byte)])?;
        idx += null_byte;
        // Null byte separator
        assert!(buf[idx] == 0x00);
        idx += 1;
        // 20 byte digest
        let digest = &buf[idx..(idx + 20)];
        idx += 20;
        res.push(TreeEntry { mode, name, digest });
    }
    Ok(res)
}

struct TreeEntry<'a> {
    //Don't care to interpret this
    mode: &'a str,
    name: &'a str,
    digest: &'a [u8],
}
