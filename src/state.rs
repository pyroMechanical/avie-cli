use avie_core::{board::{BoardState, Move, Promotion}, evaluate::MoveData};
use std::{thread::JoinHandle, collections::HashMap, sync::{Mutex, Arc}};

pub struct EngineState {
    pub search_state: Arc<Mutex<SearchState>>,
    pub debug: bool,
    pub should_quit: bool,
    pub should_stop: Arc<std::sync::atomic::AtomicBool>,
    pub search_thread: Option<JoinHandle<Option<(Move, i64)>>>
}

pub struct SearchState {
    pub move_array: [Move; 218],
    pub board: BoardState,
    pub transposition_table: HashMap<u64, MoveData>
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            move_array: [Move::new(0, 0, Promotion::None); 218],
            board: BoardState::default(),
            transposition_table: HashMap::new(),
        }
    }
}

impl EngineState {
    pub fn new() -> Self {
        Self {
            search_state: Arc::new(Mutex::new(SearchState::new())),
            debug: false,
            should_quit: false,
            should_stop: Arc::new(false.into()),
            search_thread: None
        }
    }
}