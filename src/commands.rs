use std::env;
use std::fs;
use std::fs::DirEntry;
use std::io::Cursor;
use std::io::Write;
use std::os::unix::prelude::MetadataExt;
use std::path::Path;
use std::path::PathBuf;
use std::str;
use std::time::SystemTime;

use crate::api_client::*;
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
    let digest = do_hash_object(file)?;
    let sha = format_digest(&digest)?;
    println!("{}", &sha);
    Ok(())
}

fn do_hash_object(file: &str) -> Result<Vec<u8>> {
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

// Assumes that it is invoked from the root of the git repository
pub fn write_tree() -> Result<()> {
    let cwd = env::current_dir()?;
    let cwd = cwd.to_str().unwrap();
    let digest = write_root(cwd)?;
    let sha = format_digest(&digest)?;
    println!("{}", sha);
    Ok(())
}

pub fn commit_tree(tree: &str, parent: &str, message: &str) -> Result<()> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut tmp = Vec::new();
    writeln!(&mut tmp, "tree {}", tree)?;
    writeln!(&mut tmp, "parent {}", parent)?;
    writeln!(
        &mut tmp,
        "author Tim Spence <timothywspence@gmail.com> {} +0000",
        now
    )?;
    writeln!(
        &mut tmp,
        "committer Tim Spence <timothywspence@gmail.com> {} +0000",
        now
    )?;
    writeln!(&mut tmp, "")?;
    writeln!(&mut tmp, "{}", message)?;
    let mut buf = Vec::new();
    write!(&mut buf, "commit {}\x00", tmp.len())?;
    buf.append(&mut tmp);
    let digest = compute_digest(&buf);
    let sha = format_digest(&digest)?;
    let mut z = ZlibEncoder::new(Cursor::new(buf), Compression::fast());
    let mut out = Vec::new();
    z.read_to_end(&mut out)?;
    write_digest(&sha, &mut out)?;
    println!("{}", sha);
    Ok(())
}

pub async fn clone(url: &str, path: &str) -> Result<()> {
    let client = ApiClient::new(url);
    let head = client.head().await?;
    client.fetch_pack(&head).await?;
    Ok(())
}

// TODO https://stackoverflow.com/questions/737673/how-to-read-the-mode-field-of-git-ls-trees-output
// git doesn't model all linux file permission
fn write_root(root: &str) -> Result<Vec<u8>> {
    let mut entries: Vec<DirEntry> = fs::read_dir(root)?.map(|e| e.unwrap()).collect();
    entries.sort_by(|x, y| x.path().cmp(&y.path()));
    let mut tmp = Vec::new();
    for entry in entries {
        if entry.metadata().unwrap().is_file() {
            let digest = do_hash_object(entry.path().to_str().unwrap())?;
            let mode = entry.metadata()?.mode();
            let name = entry.file_name();
            let name = name.to_str().unwrap();
            write!(&mut tmp, "{:0o} {}\x00", mode, name)?;
            tmp.append(&mut digest.to_vec());
        } else {
            if entry.metadata().unwrap().is_dir()
                && entry.path().file_name().unwrap().to_str().unwrap() != ".git"
            {
                let path = entry.path();
                let mut t = write_root(path.to_str().unwrap())?;
                let name = entry.file_name();
                let name = name.to_str().unwrap();
                write!(&mut tmp, "40000 {}\x00", name)?;
                tmp.append(&mut t);
            } else {
            }
        }
    }
    let mut buf = Vec::new();
    write!(&mut buf, "tree {}\x00", tmp.len())?;
    buf.append(&mut tmp);
    let digest = compute_digest(&buf);
    let sha = format_digest(&digest)?;
    let mut z = ZlibEncoder::new(Cursor::new(buf), Compression::fast());
    let mut out = Vec::new();
    z.read_to_end(&mut out)?;
    write_digest(&sha, &out)?;
    Ok(digest)
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
    let cwd = env::current_dir().unwrap();
    let cwd = cwd.to_str().unwrap();
    fs::create_dir_all(format!("{}/.git/objects/{}", cwd, dir))?;
    fs::write(format!("{}/.git/objects/{}/{}", cwd, dir, file), buf)?;
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
