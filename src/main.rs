use std::io::{self, BufRead};
mod uci;
mod state;

use crate::state::EngineState;

fn main() {
    let stdin = io::stdin();
    let mut state = EngineState::new();
    'cli: loop {
        for line in stdin.lock().lines() {
            match line {
                Ok(line) => {
                    uci::process_uci_command(line.as_str(), &mut state);
                    if state.should_quit {
                        break 'cli;
                    }
                }
                Err(e) => eprintln!("{}", e)
            }
        }
    }
}