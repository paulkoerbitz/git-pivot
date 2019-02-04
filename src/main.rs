#[macro_use]
extern crate serde_derive;
extern crate docopt;
extern crate git2;

use std::collections::HashMap;
use git2::{Repository};
use docopt::Docopt;
use chrono::prelude::*;

const USAGE: &'static str = "
Git Stats

Usage:
    git-stats [--path=<path>] [--from=<from_date>] [--until=<until_date>]

Options:
    --path=<path>         Path to the repository [default: .]
    --from=<start_date>   Start date
    --until=<end_date>    End date
";


#[derive(Debug, Deserialize)]
struct Args {
    flag_path: String,
    flag_from: Option<String>,
    flag_until: Option<String>,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let path = args.flag_path;
    let from = args.flag_from.and_then(|f| { DateTime::parse_from_str(&f, "%Y-%m-%d").ok() });
    let until = args.flag_until.and_then(|u| { DateTime::parse_from_str(&u, "%Y-%m-%d").ok() });

    let repo = Repository::open(&path).unwrap();
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push_head().unwrap();
    let mut commits_by_author = HashMap::new();

    for oid in revwalk {
        let commit = repo.find_commit(oid.unwrap()).unwrap();
        let time = commit.time();
        if from.is_some() && from.unwrap().timestamp() > time.seconds() {
            continue;
        }
        if until.is_some() && until.unwrap().timestamp() < time.seconds() {
            continue;
        }
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
