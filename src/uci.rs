use avie_core::board::{BoardState, Move, Promotion};
use avie_core::{File, Rank};
use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, multispace0, multispace1, one_of},
    combinator::opt,
    multi::separated_list0,
    sequence::{terminated, tuple},
};
use std::ops::DerefMut;
use std::sync::{atomic::Ordering, Arc};
use std::collections::HashSet;
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
    let from = SQUARES[mov.from as usize].to_owned();
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

fn number_from_string(line: &str) -> Option<(u64, &str)> {
    match nom::character::complete::u64::<&str, ()>(line) {
        Ok((rest, value)) => {
            Some((value, rest))
        },
        Err(_) => None,
    }
}

///assumes that the time is in milliseconds, which is always true for uci times
fn duration_from_string(line: &str) -> Option<(std::time::Duration, &str)> {
    match nom::character::complete::u64::<&str, ()>(line) {
        Ok((rest, millis)) => {
            Some((std::time::Duration::from_millis(millis), rest))
        },
        Err(_) => None,
    }
}

struct SearchArguments {
    searchmoves: Vec<Move>,
    ponder: bool,
    wtime: std::time::Duration,
    btime: std::time::Duration,
    winc: std::time::Duration,
    binc: std::time::Duration,
    movestogo: u64,
    depth: u64,
    nodes: u64,
    mate: u64,
    movetime: std::time::Duration,
    infinite: bool
}

///if multiple of the same arguments are sent, the last one is what will be used.
fn get_search_arguments(mut line: &str) -> SearchArguments{
    let mut searchmoves: Vec<Move> = vec![];
    let mut ponder = false;
    let mut wtime = std::time::Duration::from_millis(0);
    let mut btime = std::time::Duration::from_millis(0);
    let mut winc = std::time::Duration::from_millis(0);
    let mut binc = std::time::Duration::from_millis(0);
    let mut movestogo = 0;
    let mut depth = 0;
    let mut nodes = 0;
    let mut mate = 0;
    let mut movetime = std::time::Duration::from_millis(0);
    let mut infinite = false;
    
    'args: loop {
        match next_token(line) {
            Err(_) => break 'args,
            Ok((rest, token)) => {
                line = rest;
                match token {
                    "searchmoves" => {
                        match parse_moves(line) {
                            Err(_) => (),
                            Ok((rest, moves)) => {
                                line = rest;
                                searchmoves = moves;
                            }
                        }
                    },
                    "ponder" => ponder = true,
                    "wtime" => {
                        if let Some((duration, rest)) = duration_from_string(line){
                            line = rest;
                            wtime = duration;
                        }
                    }
                    "btime" => {
                        if let Some((duration, rest)) = duration_from_string(line){
                            line = rest;
                            btime = duration;
                        }
                    }
                    "winc" => {
                        if let Some((duration, rest)) = duration_from_string(line){
                            line = rest;
                            winc = duration;
                        }
                    }
                    "binc" => {
                        if let Some((duration, rest)) = duration_from_string(line){
                            line = rest;
                            binc = duration;
                        }
                    }
                    "movestogo" => {
                        if let Some((duration, rest)) = number_from_string(line){
                            line = rest;
                            movestogo = duration;
                        }
                    }
                    "depth" => {
                        if let Some((duration, rest)) = number_from_string(line){
                            line = rest;
                            depth = duration;
                        }
                    }
                    "nodes" => {
                        if let Some((duration, rest)) = number_from_string(line){
                            line = rest;
                            nodes = duration;
                        }
                    }
                    "mate" => {
                        if let Some((duration, rest)) = number_from_string(line){
                            line = rest;
                            mate = duration;
                        }
                    }
                    "movetime" => {
                        if let Some((duration, rest)) = duration_from_string(line){
                            line = rest;
                            movetime = duration;
                        }
                    }
                    "infinite" => infinite = true,
                    _ => ()
                }
            }
        }
    }

    SearchArguments {
        searchmoves,
        ponder,
        wtime,
        btime,
        winc,
        binc,
        movestogo,
        depth,
        nodes,
        mate,
        movetime,
        infinite,
    }
}

fn begin_search(engine_state: &mut EngineState, args: SearchArguments ) {
    if engine_state.search_thread.is_some() {
        return;
    }
    
    engine_state.should_stop = Arc::new(false.into());
    let state = engine_state.search_state.clone();
    let should_stop = engine_state.should_stop.clone();
    engine_state.search_thread = Some(std::thread::spawn(move || {
        if let Ok(mut state) = state.lock() {
            let stop = should_stop.as_ref();
            let search_state = state.deref_mut();
            let SearchState {
                move_array,
                board,
                transposition_table,
            } = search_state;
            *move_array = [Move::new(0, 0, Promotion::None); 218];
            let moves = board.generate_moves(move_array);
            let moves = if !args.searchmoves.is_empty() {
                let set1: HashSet<Move> = args.searchmoves.into_iter().collect();
                let set2: HashSet<Move> = moves.iter().map(|x| *x).collect();
                let intersection = set1.intersection(&set2);
                let mut len = 0;
                for (index, mov) in intersection.enumerate() {
                    len = index;
                    moves[index] = *mov;
                }
                moves.split_at_mut(len + 1).0
            } else {
                moves
            };
            let should_stop = should_stop.clone();
            if !args.movetime.is_zero() && !(args.ponder || args.infinite) {
                std::thread::spawn(move || {
                    std::thread::sleep(args.movetime);
                    should_stop.store(true, Ordering::Relaxed);
                });
            }
            let result = avie_core::evaluate::choose_best_move(
                board,
                moves,
                transposition_table,
                stop,
            );
            if let Some((mov, _score)) = result {
                println!("bestmove {}", move_to_long_algebraic(&mov))
            }
            else {
                println!("bestmove 0000")
            }
        } else {
            todo!()
        }
    }));
}

fn next_token<'a>(string: &'a str) -> IResult<&'a str, &'a str> {
    terminated(alpha1, multispace0)(string)
}

pub fn process_uci_command(commands: &str, engine_state: &mut EngineState){
    match next_token(commands) {
        Err(_) => return,
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
                    _ => return,
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
                                    return;
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
                        Err(_) => return,
                    }
                }
                "go" => {
                    begin_search(engine_state, get_search_arguments(rest));
                }
                "stop" => {
                    engine_state.should_stop.store(true, Ordering::Relaxed);
                }
                "quit" => {
                    engine_state.should_stop.store(true, Ordering::Relaxed);
                    engine_state.should_quit = true;
                }
                "" => return,
                _ => process_uci_command(rest, engine_state),
            }
        }
    }
}
