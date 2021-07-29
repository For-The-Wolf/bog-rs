use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use actix_web::{web, App, HttpServer};
use actix_files as fs;
use tera::Tera;
use web_server::single_player;

mod bog;
mod web_server;
mod game_state;
// mod ode_check;


mod responses{
    use serde::{Deserialize, Serialize};
    use actix_rt::time::Instant;
    use std::time::Duration;
    use super::game_state;

    // #[derive(Serialize, Clone,  PartialEq, Eq)]
    // pub struct GamePollResponse{
    
    #[derive(Serialize, Deserialize, Debug)]
    pub enum ButtonState{
        Active,
        Inactive,
    }

    #[derive(Serialize, Clone,  PartialEq, Eq)]
    pub struct LobbyPollResponse{
        pub player_list: Vec<(String, usize)>,
        pub status: bool,
    }
    impl LobbyPollResponse{
        pub fn from_room( game: &game_state::Game) -> Self{
            let mut player_list = Vec::<(String,usize)>::new();
            for player in game.players.values(){
                player_list.push((player.name.clone(), player.score));
            }
            let status = match game.status{
                game_state::GameStatus::InLobby => false,
                game_state::GameStatus::InProgress => true,
            };
            Self{player_list, status}
        }
    }
    #[derive(Serialize, Clone,  PartialEq, Eq)]
    pub struct GamePollResponse{
        pub time: Duration,
        pub valid_guesses: Vec<(String, bool)>,
        pub score: usize,
        pub status: bool,
    }
    impl GamePollResponse{
        pub fn from_room_and_player(player_id: &String, room: &game_state::Game) -> Self{
            let player = room.players.get(player_id).unwrap();
            let mut valid_guesses = Vec::<(String,bool)>::new();
            for guess in &player.valid_guesses{
                match guess.found{
                    game_state::Found::Once => valid_guesses.push((guess.word.clone(), true)),
                    game_state::Found::Twice => valid_guesses.push((guess.word.clone(), false)),
                }
            }
            let time = Instant::now() - room.start_time;
            let score = player.score;
            let status = match room.status{
                game_state::GameStatus::InLobby => false,
                game_state::GameStatus::InProgress => true,
            };
            Self{time, valid_guesses, score,status}
        }
    }
    #[derive(Serialize, Clone,  PartialEq, Eq)]
    pub struct GuessResponse{
        pub valid_guesses: Vec<(String, bool)>,
        pub score: usize,
    }
    impl GuessResponse{
        pub fn from_player(player: &game_state::Player) -> Self{
            let mut valid_guesses = Vec::<(String,bool)>::new();
            for guess in &player.valid_guesses{
                match guess.found{
                    game_state::Found::Once => valid_guesses.push((guess.word.clone(), true)),
                    game_state::Found::Twice => valid_guesses.push((guess.word.clone(), false)),
                }
            }
            let score = player.score;
            Self{valid_guesses, score}
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let game_state = web::Data::new(Arc::new(Mutex::new(game_state::GameState::new())));
    HttpServer::new(move || {
        let tera = Tera::new("templates/**/*.html").unwrap();
        let dictionary = bog::TrieNode::build_trie("./dict/other_word_list.txt");
        let score_map: HashMap<usize, usize> = game_state::SCORES.iter().cloned().collect();
        App::new()
            .app_data(game_state.clone())
            .data(tera)
            .data(score_map)
            .data(dictionary)
            .route("/", web::get().to(single_player))
            .route("/beta", web::get().to(web_server::index))
            .route("/beta/single_player", web::get().to(web_server::single_player))
            .route(
                "/eval_guess/{room}/{player}/{word}",
                web::post().to(web_server::eval_guess),
            )
            .route("/beta/create_room/{room_name}", web::post().to(web_server::create_lobby))
            .route("/beta/insert_player/{room_id}/{player_name}", web::post().to(web_server::insert_player))
            .route("/beta/lobby/{room_id}/{player_id}", web::get().to(web_server::lobby))
            .route("/beta/poll_lobby/{button_state}/{room_id}", web::get().to(web_server::poll_lobby))
            .route("/beta/multi_player/{room_id}/{player_id}", web::get().to(web_server::multi_player))
            .route("/beta/poll_game/{room_id}/{player_id}", web::post().to(web_server::poll_game))
            .service(fs::Files::new("/letters", "./templates/letters/"))
            .service(fs::Files::new("/", "./templates/"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
