use crate::engine::Engine;
use crate::interprocess_communication::print_command;
use crate::logging::Log;
use crate::queue::ThreadSafeQueue;
use game_sdk::gamerules::{calculate_legal_moves, get_result, is_game_finished};
use game_sdk::{Action, ActionList, Color, GameState, PieceType};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{env, thread};

pub const LOG_DIR: &str = "referee_logs/";
pub const REFEREE_ERROR_LOG: &str = "error.log";
pub const FEN_LOG: &str = "fens.txt";
mod engine;
mod interprocess_communication;
pub mod logging;
mod queue;
#[derive(Debug)]
pub struct Config {
    pub threads: usize,
    pub games: usize,
    pub engine1_path: String,
    pub engine2_path: String,
}

pub struct GameTask {
    pub opening: GameState,
    pub engine1_is_red: bool,
    pub game_id: usize,
    pub engine1: Engine,
    pub engine2: Engine,
}
pub struct TaskResult {
    pub game_id: usize,
    pub engine1: Engine,
    pub engine2: Engine,
    fens: Vec<String>,
}
fn main() {
    //Step1. Parse config
    let mut config = Config {
        threads: 1,
        games: 1000,
        engine1_path: "".to_owned(),
        engine2_path: "".to_owned(),
    };
    let args: Vec<String> = env::args().collect();
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "-n" | "n" => {
                config.games = args[index + 1]
                    .parse::<usize>()
                    .expect("Amount of games given is not a correct number");
                index += 2;
            }
            "-t" | "t" => {
                config.threads = args[index + 1]
                    .parse::<usize>()
                    .expect("Amount of threads given is not a correct number");
                index += 2;
            }
            "-p1" | "p1" => {
                config.engine1_path = args[index + 1].to_owned();
                index += 2;
            }
            "-p2" | "p2" => {
                config.engine2_path = args[index + 1].to_owned();
                index += 2;
            }
            _ => {
                index += 1;
            }
        }
    }
    //Step2. Game Loop
    game_loop(config);
}
fn game_loop(config: Config) {
    let mut engine1 = Engine::from_path(&config.engine1_path);
    let mut engine2 = Engine::from_path(&config.engine2_path);
    let game_rounds = (config.games as f64 / 2.0).ceil() as usize;
    //Setup games
    let queue: Arc<ThreadSafeQueue<GameTask>> = Arc::new(ThreadSafeQueue::new(
        load_random_openings(game_rounds, &engine1, &engine2),
    ));
    let games = queue.len();
    println!("Prepared {} games! Starting!", games);
    let result_queue: Arc<ThreadSafeQueue<TaskResult>> =
        Arc::new(ThreadSafeQueue::new(Vec::with_capacity(config.threads)));
    let error_log = Arc::new(Mutex::new(Log::init(
        &format!("{}{}", LOG_DIR, REFEREE_ERROR_LOG),
        false,
    )));
    let mut fen_log = Log::init(&format!("{}{}", LOG_DIR, FEN_LOG), true);

    //Start all childs
    let mut childs = Vec::with_capacity(config.threads);
    for _ in 0..config.threads {
        let queue_clone = queue.clone();
        let res_clone = result_queue.clone();
        let log_clone = error_log.clone();
        childs.push(thread::spawn(move || {
            while let Some(task) = queue_clone.pop() {
                println!("Starting game {}", task.game_id);
                let res = play_game(task, log_clone.clone());
                res_clone.push(res)
            }
        }))
    }
    let mut results_collected = 0;
    while results_collected < games {
        thread::sleep(Duration::from_millis(50));
        if let Some(result) = result_queue.pop() {
            results_collected += 1;
            engine1.add(&result.engine1);
            engine2.add(&result.engine2);
            println!("*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*");
            println!("Game {} finished!", result.game_id);
            let (elo_gain, other_info1, _) = engine1.get_elo_gain();
            println!("{}", elo_gain);
            let (elo_gain, other_info2, _) = engine2.get_elo_gain();
            println!("{}", elo_gain);
            println!("{}", other_info1);
            println!("{}", other_info2);
            println!("*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*-*");

            //Write fens
            result.fens.iter().for_each(|fen| {
                fen_log.log(&format!("{}\n", fen), false);
                ()
            })
        }
    }
    for child in childs {
        child.join().expect("Could not join thread!");
    }
    println!("Exiting...");
}
pub fn load_random_openings(n: usize, engine1: &Engine, engine2: &Engine) -> Vec<GameTask> {
    let mut res = Vec::with_capacity(n * 2);
    for id in 0..n {
        let mut opening = GameState::random();
        loop {
            //Make 2 random moves
            let mut al = ActionList::default();
            for _ in 0..2 {
                calculate_legal_moves(&opening, &mut al);
                opening.make_action(get_random_setmove(&al))
            }
            calculate_legal_moves(&opening, &mut al);
            if al.size == 1 && al[0] == Action::SkipMove {
                opening = GameState::random();
            } else {
                break;
            }
        }
        res.push(GameTask {
            opening: opening.clone(),
            engine1_is_red: true,
            game_id: 2 * id,
            engine1: engine1.clone(),
            engine2: engine2.clone(),
        });
        res.push(GameTask {
            opening,
            engine1_is_red: false,
            game_id: 2 * id + 1,
            engine1: engine1.clone(),
            engine2: engine2.clone(),
        });
    }
    res
}
pub fn get_random_setmove(al: &ActionList) -> Action {
    for i in 0..al.size {
        match al[i] {
            Action::SetMove(pt, _) => {
                if match pt {
                    PieceType::BEE => false,
                    _ => true,
                } {
                    return al[i];
                }
            }
            _ => continue,
        }
    }
    panic!("No setmove(wihtout bee setting) in al ")
}
pub fn play_game(game: GameTask, error_log: Arc<Mutex<Log>>) -> TaskResult {
    let write_error = |msg| {
        let mut log = error_log.lock().unwrap();
        log.log(msg, true);
    };
    let mut engine1 = game.engine1;
    let mut engine2 = game.engine2;
    let mut al = ActionList::default();
    let mut state = game.opening;
    let mut fens = Vec::with_capacity(58);

    let (mut e1_process, mut e1stdin, mut e1stdout, mut e1stderr) = engine1.get_handles();
    let (mut e2_process, mut e2stdin, mut e2stdout, mut e2stderr) = engine2.get_handles();
    let (mut e1log, mut e2log) = (
        Log::init(
            &format!("{}{}_game{}.log", LOG_DIR, engine1.name, game.game_id),
            false,
        ),
        Log::init(
            &format!("{}{}_game{}.log", LOG_DIR, engine2.name, game.game_id),
            false,
        ),
    );

    while !is_game_finished(&state) {
        let is_engine1 = state.color_to_move == Color::RED && game.engine1_is_red
            || state.color_to_move == Color::BLUE && !game.engine1_is_red;
        let action: Action;
        let score: Option<i16>;
        if is_engine1 {
            let res =
                engine1.request_move(&state, &mut e1stdin, e1stdout, &mut e1stderr, &mut e1log);
            action = res.0;
            score = res.1;
            e1stdout = res.2;
        } else {
            let res =
                engine2.request_move(&state, &mut e2stdin, e2stdout, &mut e2stderr, &mut e2log);
            action = res.0;
            score = res.1;
            e2stdout = res.2;
        }
        fens.push(format!("{}//{:?}", state.to_fen(), score));
        calculate_legal_moves(&state, &mut al);
        if al.find_action(action).is_none() {
            if is_engine1 {
                engine1.disqs += 1;
            } else {
                engine2.disqs += 1;
            }
            write_error(&format!(
                "Engine {} sent an invalid move {} in state: {}! Disqualifying..",
                if is_engine1 {
                    engine1.name.clone()
                } else {
                    engine2.name.clone()
                },
                action.to_string(),
                state.to_fen()
            ));
            break;
        } else {
            state.make_action(action);
            if is_game_finished(&state) {
                let winner = get_result(&state);
                if winner.is_none() {
                    engine1.draws += 1;
                    engine2.draws += 1;
                } else if winner.unwrap() == Color::RED {
                    if game.engine1_is_red {
                        engine1.wins += 1;
                        engine2.losses += 1;
                    } else {
                        engine1.losses += 1;
                        engine2.wins += 1;
                    }
                } else {
                    if game.engine1_is_red {
                        engine1.losses += 1;
                        engine2.wins += 1;
                    } else {
                        engine1.wins += 1;
                        engine2.losses += 1;
                    }
                }
            }
        }
    }
    fens.push(format!("{}//GameOver", state.to_fen()));

    //Close threads
    print_command(&mut e1stdin, "exit\n".to_owned());
    print_command(&mut e2stdin, "exit\n".to_owned());
    std::thread::sleep(Duration::from_millis(25));
    match e1_process.try_wait() {
        Err(_) => e1_process.kill().expect("Could not kill Engine 1"),
        _ => {}
    };
    match e2_process.try_wait() {
        Err(_) => e2_process.kill().expect("Could not kill Engine 2"),
        _ => {}
    };
    TaskResult {
        game_id: game.game_id,
        engine1,
        engine2,
        fens,
    }
}
