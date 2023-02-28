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
            hash_object(file)?;
            Ok(())
        }
    } else if args[1] == "ls-tree" {
        if args[2] != "--name-only" {
            println!("Incorrect arg format");
            Ok(())
        } else {
            let digest = &args[3];
            ls_tree(digest)
        }
    } else if args[1] == "write-tree" {
        write_tree()
    } else if args[1] == "commit-tree" {
        let tree = &args[2];
        assert!(args[3] == "-p");
        let parent = &args[4];
        assert!(args[5] == "-m");
        let message = &args[6];
        commit_tree(tree, parent, message)
    } else {

        println!("unknown command: {}", args[1]);
        Ok(())
    }
}
