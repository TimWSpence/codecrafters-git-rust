#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;

use anyhow::Result;
mod commands;
use commands::*;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args[1] == "init" {
        init()
    } else if args[1] == "cat-file" {
        if args[2] != "-p" {
            println!("Incorrect arg format");
            Ok(())
        } else {
            let digest = &args[3];
            cat_file(digest)
        }
    } else if args[1] == "hash-object" {
        if args[2] != "-w" {
            println!("Incorrect arg format");
            Ok(())
        } else {
            let file = &args[3];
            hash_object(file)
        }
    } else if args[1] == "ls-tree" {
        if args[2] != "--name-only" {
            println!("Incorrect arg format");
            Ok(())
        } else {
            let digest = &args[3];
            ls_tree(digest)
        }
    } else {
        println!("unknown command: {}", args[1]);
        Ok(())
    }
}
