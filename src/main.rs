#[macro_use]
extern crate serde_derive;

use std::boxed::Box;
use git2::Repository;
use docopt::Docopt;
use chrono::prelude::*;

mod statistics;

use statistics::*;

const DATE_FORMAT: &'static str = "%Y-%m-%d";

fn parse_date(d: String) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_str(&d, DATE_FORMAT).ok()
}

const USAGE: &'static str = "
Git Stats

Usage:
    git-stats [--path=<path>] [--from=<from_date>] [--until=<until_date>]

Options:
    --path=<path>         Path to the repository [default: .]
    --from=<start_date>   Start date (format '%Y-%m-%d')
    --until=<end_date>    End date (format '%Y-%m-%d')
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
    let from = args.flag_from.and_then(parse_date);
    let until = args.flag_until.and_then(parse_date);

    let repo = Repository::open(&path).unwrap();
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push_head().unwrap();

    let mut statistics: Vec<Box<dyn PerCommitStatistic>> = vec![
        Box::new(CommitCountByAuthor::new()),
        Box::new(Punchcard::new())
    ];

    for oid in revwalk {
        let commit = repo.find_commit(oid.unwrap()).unwrap();
        let time = commit.time();
        if from.is_some() && from.unwrap().timestamp() > time.seconds() {
            continue;
        }
        if until.is_some() && until.unwrap().timestamp() < time.seconds() {
            continue;
        }
        for statistic in &mut statistics {
            statistic.process_commit(&commit);
        }
    }
    for statistic in &statistics {
        statistic.print_result();
    }
}
