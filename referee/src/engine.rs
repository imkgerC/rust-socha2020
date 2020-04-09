use crate::interprocess_communication::{block_on_output, print_command};
use crate::logging::Log;
use game_sdk::{Action, GameState};
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio};
use std::time::Instant;

#[derive(Copy, Clone)]
pub struct EngineStats {
    pub moves_played: usize,
    pub avg_depth: f64,
    pub avg_nps: f64,
    pub avg_timeused: f64,
}
impl EngineStats {
    pub fn to_string(&self) -> String {
        let avg_depth = if self.moves_played == 0 {
            -1.
        } else {
            self.avg_depth / self.moves_played as f64
        };
        let avg_nps = if self.moves_played == 0 {
            -1.
        } else {
            self.avg_nps / self.moves_played as f64
        };
        let avg_timeused = if self.moves_played == 0 {
            -1.
        } else {
            self.avg_timeused / self.moves_played as f64
        };
        format!(
            "Avg. Depth: {:.2}, Avg. Nps: {:.2}, Avg. TimeUsed: {:.2}",
            avg_depth, avg_nps, avg_timeused
        )
    }
    pub fn add(&mut self, other: EngineStats) {
        let sum = (self.moves_played + other.moves_played) as f64;
        self.avg_depth = (self.avg_depth * self.moves_played as f64
            + other.avg_depth * other.moves_played as f64)
            / sum;
        self.avg_nps = (self.avg_nps * self.moves_played as f64
            + other.avg_nps * other.moves_played as f64)
            / sum;
        self.avg_timeused = (self.avg_timeused * self.moves_played as f64
            + other.avg_timeused * other.moves_played as f64)
            / sum;
        self.moves_played += other.moves_played;
    }
}
impl Default for EngineStats {
    fn default() -> Self {
        EngineStats {
            moves_played: 0,
            avg_depth: 0.,
            avg_nps: 0.,
            avg_timeused: 0.,
        }
    }
}
#[derive(Clone)]
pub struct Engine {
    pub name: String,
    pub path: String,
    pub wins: usize,
    pub draws: usize,
    pub losses: usize,
    pub disqs: usize,
    pub stats: EngineStats,
}
impl Engine {
    pub fn get_handles(&self) -> (Child, ChildStdin, ChildStdout, ChildStderr) {
        let mut process = Command::new(self.path.clone())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|_| panic!("Failed to start engine {}!", self.path));
        let input = process.stdin.take().unwrap();
        let output = process.stdout.take().unwrap();
        let stderr = process.stderr.take().unwrap();
        (process, input, output, stderr)
    }

    pub fn add(&mut self, other: &Engine) {
        self.stats.add(other.stats);
        self.wins += other.wins;
        self.draws += other.draws;
        self.losses += other.losses;
        self.disqs += other.disqs;
    }

    pub fn get_elo_gain(&self) -> (String, String, f64) {
        //Derived from 1. E_A= 1/(1+10^(-DeltaElo/400)) and 2. |X/N-p|<=1.96*sqrt(N*p*(1-p))/n
        let n: f64 = (self.wins + self.draws + self.losses) as f64;
        let x_a: f64 = self.wins as f64 + self.draws as f64 / 2.0;
        let (elo_gain, elo_bounds) = if n >= 1. || x_a >= 0. {
            let p_a: f64 = x_a / n;
            let k: f64 = (1.96 * 1.96 + 2.0 * x_a) / (-1.0 * 1.96 * 1.96 - n);
            let q = -1.0 * x_a * x_a / (n * (-1.96 * 1.96 - n));
            let root = ((k / 2.0) * (k / 2.0) - q).sqrt();
            let p_a_upper: f64 = -k / 2.0 + root;
            let curr = get_elo_gain(p_a);
            (curr, get_elo_gain(p_a_upper) - curr)
        } else {
            (0., 0.)
        };
        (
            format!(
                "{:40} {:.2}   +/- {:.2}   +{}   ={}   -{}  sc {:.1}%",
                self.name,
                elo_gain,
                elo_bounds,
                self.wins,
                self.draws,
                self.losses,
                100. * (self.wins as f64 + self.draws as f64 / 2.)
                    / (self.wins + self.draws + self.losses) as f64,
            ),
            format!(
                "{:40} disq {} dep {:.2} nps {:.0} time {:.0}",
                self.name,
                self.disqs,
                self.stats.avg_depth,
                self.stats.avg_nps,
                self.stats.avg_timeused
            ),
            elo_gain,
        )
    }

    pub fn from_path(path: &str) -> Self {
        let name: Vec<&str> = path.split("/").collect();
        let name = name[name.len() - 1].replace(".exe", "").replace(".jar", "");
        Engine {
            name,
            path: path.to_owned(),
            wins: 0,
            draws: 0,
            losses: 0,
            disqs: 0,
            stats: EngineStats::default(),
        }
    }

    pub fn request_move(
        &mut self,
        game_state: &GameState,
        stdin: &mut ChildStdin,
        stdout: ChildStdout,
        stderr: &mut ChildStderr,
        engine_log: &mut Log,
    ) -> (Option<Action>, Option<i16>, ChildStdout) {
        let request = format!("requestmove {}\n", game_state.to_fen());
        let now = Instant::now();
        print_command(stdin, request);
        let (output, stdout) = block_on_output(
            stdout,
            Box::new(|s: String| s.contains("bestmove") && !s.contains("info")),
            stderr,
            engine_log,
        );
        let elapsed = now.elapsed().as_millis() as usize;
        engine_log.log(output.as_str(), false);
        let mut stats = EngineStats::default();
        stats.moves_played = 1;
        stats.avg_timeused = elapsed as f64;
        let mut action = None;
        let mut score = None;
        let lines: Vec<&str> = output.split("\n").collect();
        lines.iter().for_each(|&line| {
            if line.starts_with("bestmove") {
                let action_str = line.split(" ").collect::<Vec<&str>>()[1..].join(" ");
                action = Some(Action::from_string(action_str));
            } else if line.starts_with("info") {
                let args = line.split(" ").collect::<Vec<&str>>();
                let mut index = 1;
                while index < args.len() {
                    match args[index] {
                        "nps" => {
                            stats.avg_nps = args[index + 1].parse::<f64>().unwrap();
                            index += 2;
                        }
                        "depth" => {
                            stats.avg_depth = args[index + 1].parse::<f64>().unwrap();
                            index += 2;
                        }
                        "score" => {
                            score = Some(args[index + 1].parse::<i16>().unwrap());
                            index += 2;
                        }
                        _ => index += 1,
                    }
                }
            }
            ()
        });
        self.stats.add(stats);
        (action, score, stdout)
    }
}
pub fn get_elo_gain(p_a: f64) -> f64 {
    -1.0 * (1.0 / p_a - 1.0).ln() * 400.0 / (10.0 as f64).ln()
}
