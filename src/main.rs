#[macro_use]
extern crate serde_derive;

use std::collections::HashMap;
use std::cmp::max;
use git2::{Repository, Commit};
use docopt::Docopt;
use chrono::prelude::*;

const DATE_FORMAT: &'static str = "%Y-%m-%d";

fn parse_date(d: String) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_str(&d, DATE_FORMAT).ok()
}

const USAGE: &'static str = "
Git Pivot

Usage:
    git-pivot [--path=<path>] [--from=<from_date>] [--until=<until_date>] [--x-category=<x-category>] [--y-category=<y-category>] [--statistic=<statistic>]

Options:
    --path=<path>              Path to the repository [default: .]
    --from=<start_date>        Start date (format '%Y-%m-%d')
    --until=<end_date>         End date (format '%Y-%m-%d')
    --x-category=<x-category>  Values by which separated on the x-axis
    --y-category=<y-category>  Values by which the summary statistic separated on the y-axis
    --statistic=<statistic>    Summary statistic, can be 'Commits', 'Additions' or 'Deletions'
                               This option can be repeated to list multiple statistics
                               at the same time
";

#[derive(Debug, Deserialize, Copy, Clone)]
enum Statistic {
    Commits,
    Additions,
    Deletions,
}

#[derive(Debug, Deserialize, Copy, Clone)]
enum Category {
    Date,
    Week,
    Month,
    Year,
    DayOfWeek,
    DayOfMonth,
    Hour,
    Author,
    AuthorEmail,
    File,
    Directory
}

#[derive(Debug, Deserialize)]
struct Args {
    flag_path: String,
    flag_from: Option<String>,
    flag_until: Option<String>,
    flag_statistic: Option<Statistic>,
    flag_x_category: Option<Category>,
    flag_y_category: Option<Category>,
}

fn commit_to_datetime(commit: &Commit) -> DateTime<FixedOffset> {
    let (seconds, offset) = (commit.time().seconds(), commit.time().offset_minutes() * 60);
    FixedOffset::east(offset).timestamp(seconds, 0)
}

fn commit_to_cat(commit: &Commit, cat: Option<Category>) -> String {
    match cat {
        None => String::from("*"),
        Some(Category::Date) => commit_to_datetime(&commit).format("%Y-%m-%d").to_string(),
        Some(Category::Week) => commit_to_datetime(&commit).format("%Y-%w").to_string(),
        Some(Category::Month) => commit_to_datetime(&commit).format("%Y-%m").to_string(),
        Some(Category::Year) => commit_to_datetime(&commit).year().to_string(),
        Some(Category::DayOfWeek) => format!("{:?}", commit_to_datetime(&commit).weekday()),
        Some(Category::DayOfMonth) => commit_to_datetime(&commit).format("%d").to_string(),
        Some(Category::Hour) => commit_to_datetime(&commit).hour().to_string(),
        Some(Category::Author) => String::from(commit.author().name().unwrap_or("<UNNOWN>")),
        Some(Category::AuthorEmail) => String::from(commit.author().email().unwrap_or("<UNKNOWN>")),
        Some(val) => format!("{:?}", val)
    }
}

fn commit_to_stat(_commit: &Commit, stat: Statistic) -> i32 {
    match stat {
        Statistic::Commits => 1,
        // do I need to diff here?
        Statistic::Additions => 1,
        Statistic::Deletions => -1,
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let path = args.flag_path;
    let from = args.flag_from.and_then(parse_date);
    let until = args.flag_until.and_then(parse_date);
    let statistic = args.flag_statistic.unwrap_or(Statistic::Commits);
    let x_cat = args.flag_x_category;
    let y_cat = args.flag_y_category;

    let mut stats: HashMap<(String, String), i32> = HashMap::new();

    let repo = Repository::open(&path).unwrap();
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push_head().unwrap();

    for oid in revwalk {
        let commit = repo.find_commit(oid.unwrap()).unwrap();
        let time = commit.time();
        if from.is_some() && from.unwrap().timestamp() > time.seconds() {
            continue;
        }
        if until.is_some() && until.unwrap().timestamp() < time.seconds() {
            continue;
        }
        let key = (commit_to_cat(&commit, x_cat), commit_to_cat(&commit, y_cat));
        let stat = stats.entry(key).or_insert(0);
        *stat += commit_to_stat(&commit, statistic);
    }
    println!("Statistic for {:?}", statistic);
    println!("{}", format_table(&x_cat, &y_cat, &stats));
}

fn first((x, _): &(String, String)) -> String { x.to_string() }
fn second((_, y): &(String, String)) -> String { y.to_string() }

fn format_table(x_cat: &Option<Category>, y_cat: &Option<Category>, data: &HashMap<(String, String), i32>) -> String {
    let x_cats = category_values(x_cat, data, &first);
    let y_cats = category_values(y_cat, data, &second);
    let mut entries: Vec<Vec<String>> = vec![];
    let mut max_width: Vec<usize> = vec![0];
    {
        let mut first_row = vec!["".to_string()];
        for (i, y) in y_cats.iter().enumerate() {
            first_row.push(y.to_string());
            update_max_width(&y, i + 1, &mut max_width);
        }
        entries.push(first_row);
    }
    for x in &x_cats {
        let mut next_row: Vec<String> = vec![x.clone()];
        update_max_width(&x, 0, &mut max_width);
        for (i, y) in y_cats.iter().enumerate() {
            // FIXME: don't clone strings to look up in dict
            let val = data.get(&(x.to_string(), y.to_string())).unwrap_or(&0).to_string();
            update_max_width(&val, i + 1, &mut max_width);
            next_row.push(val);
        }
        entries.push(next_row);
    }
    let mut rows: Vec<String> = entries.iter().map(|row| {
        row.iter().zip(max_width.iter()).map(|(cell, width)|
            left_pad(cell, *width)
        ).join(" | ")
    }).collect();
    {
        let second_row = max_width.iter().map(|width| "-".repeat(*width)).join("-+-");
        rows.insert(1, second_row);
    }
    rows.join("\n")
}

fn update_max_width(s: &str, i: usize, max_width: &mut Vec<usize>) -> () {
    let n_chars = s.chars().count();
    match max_width.get_mut(i) {
        Some(elem) => *elem = max(n_chars, *elem),
        None => max_width.push(n_chars),
    }
}

trait Joinable {
    // FIXME: not sure why I need mut here ...
    fn join(&mut self, sep: &str) -> String;
}

impl<T: Iterator<Item = String>> Joinable for T {
    fn join(&mut self, sep: &str) -> String {
        let mut first = true;
        self.fold(String::new(), |mut acc, x| {
            // FIXME: urgh
            if !first {
                acc.push_str(sep);
            } else {
                first = false;
            }
            acc.push_str(&x);
            acc
        })
    }
}

fn category_values(cat: &Option<Category>, data: &HashMap<(String, String), i32>, selector: &dyn Fn(&(String, String)) -> String) -> Vec<String> {
    match cat {
        Some(Category::Hour) => (0..24).map(|x| x.to_string()).collect(),
        Some(Category::DayOfWeek) => vec!["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"].iter().map(|x| x.to_string()).collect(),
        Some(Category::Author) => {
            let mut authors = data.keys().map(selector).collect::<Vec<String>>();
            authors.sort();
            authors.dedup();
            authors
        }
        _ => vec![String::from("*")],
    }
}

fn left_pad(s: &str, size: usize) -> String {
    let mut result = String::new();
    let s_chars = s.chars().count();
    let mut res_chars = 0;
    while s_chars + res_chars < size {
        result.push(' ');
        res_chars += 1;
    }
    result.push_str(s);
    result
}