use super::game_state;
use actix_rt::time::Instant;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// #[derive(Serialize, Clone,  PartialEq, Eq)]
// pub struct GamePollResponse{

#[derive(Serialize, Deserialize, Debug)]
pub enum ButtonState {
    Active,
    Inactive,
}

#[derive(Serialize, Clone, PartialEq, Eq)]
pub struct PlayerResponse {
    name: String,
    score: usize,
    unique_words: Vec<String>,
}

#[derive(Serialize, Clone, PartialEq, Eq)]
pub struct LobbyPollResponse {
    pub player_list: Vec<PlayerResponse>,
    pub status: bool,
    pub is_first: bool,
}
impl LobbyPollResponse {
    pub fn from_room(game: &game_state::Game) -> Self {
        let mut player_list = Vec::<PlayerResponse>::new();
        for player in game.players.values() {
            let score = player.score;
            let name = player.name.clone();
            let unique_words: Vec<String> = player
                .valid_guesses
                .iter()
                .filter_map(|guess| match guess.found {
                    game_state::Found::Once => Some(guess.word.clone()),
                    game_state::Found::Twice => None,
                })
                .collect();
            player_list.push(PlayerResponse {
                name,
                score,
                unique_words,
            });
        }
        let status = match game.status {
            game_state::GameStatus::InLobby => false,
            game_state::GameStatus::InProgress => true,
        };
        let is_first = match game.game_number {
            0 => true,
            _ => false,
        };
        Self {
            player_list,
            status,
            is_first,
        }
    }
}
#[derive(Serialize, Clone, PartialEq, Eq)]
pub struct GamePollResponse {
    pub time: Duration,
    pub valid_guesses: Vec<(String, bool)>,
    pub score: usize,
    pub status: bool,
}
impl GamePollResponse {
    pub fn from_room_and_player(player_id: &String, room: &game_state::Game) -> Self {
        let player = room.players.get(player_id).unwrap();
        let mut valid_guesses = Vec::<(String, bool)>::new();
        for guess in &player.valid_guesses {
            match guess.found {
                game_state::Found::Once => valid_guesses.push((guess.word.clone(), true)),
                game_state::Found::Twice => valid_guesses.push((guess.word.clone(), false)),
            }
        }
        let time = Instant::now() - room.start_time;
        let score = player.score;
        let status = match room.status {
            game_state::GameStatus::InLobby => false,
            game_state::GameStatus::InProgress => true,
        };
        Self {
            time,
            valid_guesses,
            score,
            status,
        }
    }
}
#[derive(Serialize, Clone, PartialEq, Eq)]
pub struct GuessResponse {
    pub valid_guesses: Vec<(String, bool)>,
    pub score: usize,
}
impl GuessResponse {
    pub fn from_player(player: &game_state::Player) -> Self {
        let mut valid_guesses = Vec::<(String, bool)>::new();
        for guess in &player.valid_guesses {
            match guess.found {
                game_state::Found::Once => valid_guesses.push((guess.word.clone(), true)),
                game_state::Found::Twice => valid_guesses.push((guess.word.clone(), false)),
            }
        }
        let score = player.score;
        Self {
            valid_guesses,
            score,
        }
    }
}
