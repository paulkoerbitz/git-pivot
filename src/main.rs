extern crate git2;

use git2::{Repository};


fn main() {
    let path = ".";
    let repo = match Repository::open(&path) {
        Ok(repo) => repo,
        Err(err) => panic!(format!("Can't open repository at '.': {}", err))
    };
    println!("Hello, world!");
}
