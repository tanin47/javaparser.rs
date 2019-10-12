#[derive(Clone)]
pub struct IdGen {
    pub uuid: usize,
    pub path: String,
    pub runner: usize,
}

impl IdGen {
    pub fn get_next(&mut self, part1: &str, part2: &str) -> String {
        self.runner += 1;
        format!(
            "u{}_{}_{}_{}_{}",
            self.uuid, self.path, part1, part2, self.runner
        )
    }
}
