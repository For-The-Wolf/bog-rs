use crate::responses;
use actix_rt::time::Instant;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::iter;

use super::bog;

pub static SCORES: [(usize, usize); 13] = [
    (4, 2),
    (5, 4),
    (6, 7),
    (7, 12),
    (8, 20),
    (9, 33),
    (10, 54),
    (11, 88),
    (12, 143),
    (13, 232),
    (14, 376),
    (15, 609),
    (16, 986),
];

#[derive(PartialEq, Eq)]
pub enum GameStatus {
    InLobby,
    InProgress,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Found {
    Once,
    Twice,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Guess {
    pub word: String,
    pub found: Found,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
    pub valid_guesses: VecDeque<Guess>,
    pub score: usize,
}

pub struct Game {
    pub board: bog::BoggleBoard,
    pub name: String,
    pub solutions: Vec<String>,
    pub found_once: Vec<String>,
    pub found_twice: Vec<String>,
    pub players: HashMap<String, Player>,
    pub start_time: Instant,
    pub expiration_time: Instant,
    pub status: GameStatus,
    pub game_number: usize,
}

impl Game {
    fn new(name: String) -> Self {
        let board = bog::BoggleBoard::new();
        let solutions = Vec::<String>::new();
        let found_once = Vec::<String>::new();
        let found_twice = Vec::<String>::new();
        let players = HashMap::<String, Player>::new();
        let start_time = Instant::now();
        let expiration_time = Instant::now();
        let status = GameStatus::InLobby;
        let game_number = 0;
        Game {
            board,
            name,
            solutions,
            found_once,
            found_twice,
            players,
            start_time,
            expiration_time,
            status,
            game_number,
        }
    }
    pub fn new_player(&mut self, player_name: String) -> String {
        let player_token = GameState::generate_token();
        self.players.insert(
            player_token.clone(),
            Player {
                name: player_name,
                valid_guesses: VecDeque::<Guess>::new(),
                score: 0,
            },
        );
        player_token
    }
    pub fn activate(&mut self, trie: &bog::TrieNode, button_state: &responses::ButtonState) {
        match self.status {
            GameStatus::InLobby => match button_state {
                responses::ButtonState::Inactive => {
                    self.status = GameStatus::InProgress;
                    self.game_number += 1;
                    self.board.randomise();
                    self.solutions = self.board.solve(trie).iter().cloned().collect();
                    self.start_time = Instant::now();
                    for player in self.players.values_mut() {
                        player.valid_guesses = VecDeque::new();
                    }
                }
                _ => (),
            },
            _ => (),
        }
    }
}

pub struct GameState {
    pub games: HashMap<String, Game>,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            games: HashMap::<String, Game>::new(),
        }
    }
    fn generate_token() -> String {
        let mut rng = thread_rng();
        let token: String = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .map(char::from)
            .take(12)
            .collect();
        token
    }
    pub fn new_session_single(&mut self) -> String {
        let mut room_id = Self::generate_token();
        while self.games.keys().any(|id| &room_id == id) {
            room_id = Self::generate_token();
        }
        self.games
            .insert(room_id.clone(), Game::new(String::from("single_player")));
        self.games.get_mut(&room_id).unwrap().players.insert(
            String::from("single_player"),
            Player {
                name: String::from("Joe"),
                valid_guesses: VecDeque::<Guess>::new(),
                score: 0,
            },
        );
        room_id
    }
    pub fn new_session_multi(&mut self, room_name: String) -> String {
        let mut room_id = Self::generate_token();
        while self.games.keys().any(|id| &room_id == id) {
            room_id = Self::generate_token();
        }
        self.games.insert(room_id.clone(), Game::new(room_name));
        room_id
    }
    pub fn forget(&mut self, room_id: String) {
        self.games.remove(&room_id);
    }
}
