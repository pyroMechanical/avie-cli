use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, multispace0};
use nom::sequence::terminated;
use nom::IResult;

static AVIE_VERSION: &str = "0.0.1";

fn introduction() {
    println!("id name Avie {}", AVIE_VERSION);
    println!("id author Ruby Rinken & Andrea O'Gara");
}

fn options() {
    //add options here as they come up
}

fn parse_error() {
    todo!()
}

static STARTPOS: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn accept_new_position<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    let (rest, tag) = terminated(alt((tag("startpos"), tag("fen"))), multispace0)(input)?;
    match tag {
        "startpos" => return Ok(( rest, STARTPOS)),
        "fen" => return avie_core::parse::fen_string()(rest),
        _ => unreachable!(),
    }
}

fn next_token<'a>(string: &'a str) -> IResult<&'a str, &'a str> {
    terminated(alpha1, multispace0)(string)
}

pub fn process_uci_command(commands: &str) -> bool{
    match next_token(commands) {
        Err(_) => parse_error(),
        Ok((rest, string)) => {
            match string {
                "uci" => {
                    introduction();
                    options();
                    println!("uciok");
                }
                "debug" => match next_token(rest) {
                    Ok((_, "on")) => {
                        todo!()
                    }
                    Ok((_, "off")) => {
                        todo!()
                    }
                    _ => parse_error(),
                },
                "isready" => {
                    //todo!() any preprocessing here that must happen on startup but after compile time
                    println!("readyok");
                }
                "ucinewgame" => {
                    //nothing... for now
                }
                "position" => {
                    let fen = accept_new_position(rest);
                    match fen {
                        Ok((rest, fen)) => {
                            let state = avie_core::parse::fen_to_game(fen);
                            if let Ok(state) = state {
                                println!("{:?}", state);
                            }
                            match next_token(rest) {
                                Ok((rest, token)) => {
                                    match token {
                                        "moves" => {
                                            todo!()
                                        }
                                        _ => (),
                                    }
                                }
                                Err(_) => ()
                            }
                        }
                        Err(_) => parse_error(),
                    }
                }
                "quit" => {
                    return true;
                }
                _ => parse_error(),
            }
        }
    }
    false
}
