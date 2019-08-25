use std::collections::HashMap;
use git2::Commit;

use crate::statistics::PerCommitStatistic;

pub struct CommitCountByAuthor {
    commits_by_author: HashMap<String, i32>,
}

impl CommitCountByAuthor {
    pub fn new() -> CommitCountByAuthor {
        CommitCountByAuthor { commits_by_author: HashMap::new() }
    }
}

impl PerCommitStatistic for CommitCountByAuthor {
    fn process_commit(&mut self, commit: &Commit) -> () {
        let author = commit.author();
        author.email().map(|name| {
          let count = self.commits_by_author.entry(name.to_string()).or_insert(0);
          *count += 1;
        });
    }

    fn print_result(&self) -> () {
        let mut commits_by_author_vec: Vec<(&String, &i32)> = self.commits_by_author.iter().collect();
        commits_by_author_vec.sort_by(|left, right| { right.1.cmp(left.1) });
        for (name, count) in commits_by_author_vec {
            println!("{}: {}", name, count);
        }
    }
}