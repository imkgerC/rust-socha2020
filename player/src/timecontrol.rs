#[derive(Copy, Clone)]
pub enum Timecontrol {
    Infinite,
    MoveTime(u64),
}
impl Timecontrol {
    pub fn time_over(&self, elapsed: u64) -> bool {
        match self {
            Timecontrol::Infinite => false,
            Timecontrol::MoveTime(movetime) => elapsed >= *movetime,
        }
    }

    pub fn time_left(&self, elapsed: u64) -> i64 {
        match self {
            Timecontrol::Infinite => 2000,
            Timecontrol::MoveTime(movetime) => *movetime as i64 - elapsed as i64,
        }
    }
}
