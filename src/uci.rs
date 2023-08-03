use avie_core::board::{BoardState, Move, Promotion};
use avie_core::{File, Rank};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, multispace0, multispace1, one_of};
use nom::combinator::opt;
use nom::multi::separated_list0;
use nom::sequence::{terminated, tuple};
use nom::IResult;
use std::sync::Arc;
use std::ops::DerefMut;
use std::sync::atomic::Ordering;

use crate::state::{EngineState, SearchState};

static AVIE_VERSION: &str = "0.0.1";

fn move_to_long_algebraic(mov: &Move) -> String {
    static SQUARES: [&str; 64] = [
        "h1", "g1", "f1", "e1", "d1", "c1", "b1", "a1", "h2", "g2", "f2", "e2", "d2", "c2", "b2",
        "a2", "h3", "g3", "f3", "e3", "d3", "c3", "b3", "a3", "h4", "g4", "f4", "e4", "d4", "c4",
        "b4", "a4", "h5", "g5", "f5", "e5", "d5", "c5", "b5", "a5", "h6", "g6", "f6", "e6", "d6",
        "c6", "b6", "a6", "h7", "g7", "f7", "e7", "d7", "c7", "b7", "a7", "h8", "g8", "f8", "e8",
        "d8", "c8", "b8", "a8",
    ];
    let mut from = SQUARES[mov.from as usize].to_owned();
    let to = SQUARES[mov.to as usize];
    let promotion = match mov.promotion {
        Promotion::None => "",
        Promotion::Knight => "n",
        Promotion::Bishop => "b",
        Promotion::Rook => "r",
        Promotion::Queen => "q",
    };
    from + to + promotion
}

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

fn parse_long_algebraic(input: &str) -> IResult<&str, Move> {
    let (rest, (from_file, from_rank, to_file, to_rank, promotion)) = tuple((
        one_of("abcdefghABCDEFGH"),
        one_of("12345678"),
        one_of("abcdefghABCDEFGH"),
        one_of("12345678"),
        opt(one_of("nbrqNBRQ")),
    ))(input)?;
    let from_file: File = from_file.to_ascii_lowercase().try_into().unwrap();
    let from_rank: Rank = from_rank.to_ascii_lowercase().try_into().unwrap();
    let to_file: File = to_file.to_ascii_lowercase().try_into().unwrap();
    let to_rank: Rank = to_rank.to_ascii_lowercase().try_into().unwrap();
    let promotion: Promotion = match promotion {
        None => Promotion::None,
        Some(char) => match char.to_ascii_lowercase() {
            'n' => Promotion::Knight,
            'b' => Promotion::Bishop,
            'r' => Promotion::Rook,
            'q' => Promotion::Queen,
            _ => Promotion::None,
        },
    };
    let from: u8 = from_file.to_u8() + from_rank.to_u8() * 8;
    let to: u8 = to_file.to_u8() + to_rank.to_u8() * 8;
    Ok((
        rest,
        Move {
            from,
            to,
            promotion,
        },
    ))
}

fn parse_moves(input: &str) -> IResult<&str, Vec<Move>> {
    separated_list0(multispace1, parse_long_algebraic)(input)
}

static STARTPOS: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn accept_new_position<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    let (rest, tag) = terminated(alt((tag("startpos"), tag("fen"))), multispace1)(input)?;
    match tag {
        "startpos" => return Ok((rest, STARTPOS)),
        "fen" => return avie_core::parse::fen_string()(rest),
        _ => unreachable!(),
    }
}

fn next_token<'a>(string: &'a str) -> IResult<&'a str, &'a str> {
    terminated(alpha1, multispace0)(string)
}

pub fn process_uci_command(commands: &str, engine_state: &mut EngineState) -> bool {
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
                        engine_state.debug = true;
                    }
                    Ok((_, "off")) => {
                        engine_state.debug = false;
                    }
                    _ => parse_error(),
                },
                "isready" => {
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
                            let state = match state {
                                Ok(state) => state,
                                Err(e) => {
                                    println!("error: {:?}", e);
                                    return false;
                                }
                            };
                            if let Ok(mut engine_state) = engine_state.search_state.try_lock() {
                                engine_state.board = BoardState::new(state);
                                match next_token(rest) {
                                    Ok((rest, token)) => match token {
                                        "moves" => {
                                            println!("{}", rest);
                                            let move_list = parse_moves(rest);
                                            match move_list {
                                                Err(e) => println!("error: {:#?}", e),
                                                Ok(move_list) => {
                                                    for m in move_list.1 {
                                                        engine_state.board.make_move(m);
                                                    }
                                                }
                                            }
                                            println!("{:#?}", engine_state.board);
                                        }
                                        _ => (),
                                    },
                                    Err(_) => (),
                                }
                            }
                        }
                        Err(_) => parse_error(),
                    }
                }
                "go" => {
                    //todo!() handle case where existing thread is running
                    engine_state.should_stop = Arc::new(false.into());
                    let state = engine_state.search_state.clone();
                    let should_stop = engine_state.should_stop.clone();
                    engine_state.search_thread = Some(std::thread::spawn(move ||{
                        if let Ok(mut state) = state.lock() {
                            let should_stop = should_stop.as_ref();
                            let search_state = state.deref_mut();
                            let SearchState{move_array, board, transposition_table} = search_state;
                            *move_array = [Move::new(0, 0, Promotion::None);218];
                            let mut moves = board.generate_moves(move_array);
                            avie_core::evaluate::choose_best_move(board, moves, transposition_table, should_stop)
                        } else {
                            todo!()
                        }
                    }));
                }
                "stop" => {
                    engine_state.should_stop.store(true, Ordering::Relaxed);
                    let handle = std::mem::replace(&mut engine_state.search_thread, None);
                    if let Some(handle) = handle {
                        let result = handle.join();
                        
                        match result {
                            Ok(Some((mov, score))) => {
                                if engine_state.debug {
                                    println!("score {}", score as f64 / 100f64)
                                };
                                println!("bestmove {}", move_to_long_algebraic(&mov))
                            },
                            _ => println!("bestmove 0000")
                        }
                    };
                }
                "quit" => {
                    engine_state.should_stop.store(true, Ordering::Relaxed);
                    engine_state.should_quit = true;
                }
                _ => parse_error(),
            }
        }
    }
    false
}
