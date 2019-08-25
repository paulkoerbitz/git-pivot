use git2::Commit;

pub trait PerCommitStatistic {
    fn process_commit(&mut self, commit: &Commit) -> ();
    fn print_result(&self) -> ();
}