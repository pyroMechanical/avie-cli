use std::io::{self, BufRead};
mod uci;

fn main() {
    let stdin = io::stdin();
    'cli: loop {
        for line in stdin.lock().lines() {
            match line {
                Ok(line) => {
                    let should_quit = uci::process_uci_command(line.as_str());
                    if should_quit {
                        break 'cli;
                    }
                }
                Err(e) => eprintln!("{}", e)
            }
        }
    }
}