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
}
