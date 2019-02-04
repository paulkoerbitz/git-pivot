#[macro_use]
extern crate serde_derive;
extern crate docopt;
extern crate git2;

use std::collections::HashMap;
use git2::{Repository};
use docopt::Docopt;

const USAGE: &'static str = "
Git Stats

Usage:
  git-stats [-p <path> | --path=<path>]

Options:
  -p <path> --path=<path>       Path to the repository [default: .]
";


#[derive(Debug, Deserialize)]
struct Args {
    flag_path: String,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let path = args.flag_path;
    let repo = Repository::open(&path).unwrap();
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push_head().unwrap();
    let mut commits_by_author = HashMap::new();
    for oid in revwalk {
        let commit = repo.find_commit(oid.unwrap()).unwrap();
        let author = commit.author();
        author.name().map(|name| {
          let count = commits_by_author.entry(name.to_string()).or_insert(0);
          *count += 1;
        });
    }
    let mut commits_by_author_vec: Vec<(&String, &i32)> = commits_by_author.iter().collect();
    commits_by_author_vec.sort_by(|left, right| { right.1.cmp(left.1) });
    for (name, count) in commits_by_author_vec {
        println!("{}: {}", name, count);
    }
}
