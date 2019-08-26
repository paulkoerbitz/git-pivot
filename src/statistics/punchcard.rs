use chrono::prelude::*;
use git2::Commit;

use crate::statistics::PerCommitStatistic;

pub struct Punchcard {
    punches: [[u32; 24]; 7],
}

impl Punchcard {
    pub fn new() -> Punchcard {
        Punchcard {
            punches: [[0; 24]; 7]
        }
    }

    fn commits_for_weekday(&self, weekday: &Weekday) -> &[u32; 24] {
        &self.punches[weekday.num_days_from_monday() as usize]
    }

    fn at(&mut self, weekday: &Weekday, hour: u32) -> &mut u32 {
        &mut self.punches[weekday.num_days_from_monday() as usize][hour as usize]
    }
}

impl PerCommitStatistic for Punchcard {
    fn process_commit(&mut self, commit: &Commit) -> () {
        let ctime = commit.time();
        let datetime = FixedOffset::east(ctime.offset_minutes() * 60).timestamp(ctime.seconds(), 0);
        let count = self.at(&datetime.weekday(), datetime.hour());
        *count += 1;
    }

    fn print_result(&self) -> () {
        let header: Vec<String> = (0..24).map(|hour| { left_pad(hour, 5) }).collect();
        println!("Punchcard");
        println!("=========\n");
        println!("    | {} |", header.join(" | "));
        println!("----+-{}-+", &["-----"; 24].join("-+-"));
        for day in &["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] {
            let weekday = day.parse::<Weekday>().unwrap();
            let commits: Vec<String> = self.commits_for_weekday(&weekday).into_iter().map(|hour| { left_pad(*hour, 5) }).collect();
            println!(
                "{} | {} |",
                day,
                commits.join(" | ")
            );
        }
    }
}

fn left_pad(hour: u32, size: usize) -> String {
    let hour = hour.to_string();
    let mut result = String::new();
    while hour.len() + result.len() < size {
        result.push(' ');
    }
    result.push_str(&hour);
    result
}