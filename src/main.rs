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
            let sha = &args[3];
            cat_file(sha)
        }
    } else {
        println!("unknown command: {}", args[1]);
        Ok(())
    }
}
