use actix_rt::time::Instant;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tera::{Context, Tera};
use crate::game_state::GameStatus;

use super::bog;
use super::game_state;
use super::responses;

static LIFETIME_SECONDS: u64 = 3 * 60;

pub async fn eval_guess(
    req: HttpRequest,
    score_map: web::Data<HashMap<usize, usize>>,
    state: web::Data<Arc<game_state::GameState>>,
) -> impl Responder {
    let guess = &req.match_info().get("word").unwrap().to_lowercase();
    let room_id = req.match_info().get("room").unwrap();
    let player_id = req.match_info().get("player").unwrap();
    let mut room = if let Some(room) = state.games.get_mut(room_id) {
        room
    } else {
        return HttpResponse::BadRequest()
            .reason("Room not found")
            .body("Room not found");
    };
    let room = room.value_mut();
    let player = if let Some(player) = room.players.get_mut(player_id) {
        player
    } else {
        return HttpResponse::BadRequest()
            .reason("Player not found")
            .body("Player not found");
    };
    if room.solutions.contains(&guess)
        && !player
            .valid_guesses
            .iter()
            .any(|existing_guess| &existing_guess.word == guess)
    {
        if room.found_twice.contains(&guess) {
            player.valid_guesses.push_front(game_state::Guess {
                word: guess.clone(),
                found: game_state::Found::Twice,
            });
        } else if room.found_once.contains(&guess) {
            player.valid_guesses.push_front(game_state::Guess {
                word: guess.clone(),
                found: game_state::Found::Twice,
            });
            room.found_twice.push(guess.clone());
        } else {
            player.valid_guesses.push_front(game_state::Guess {
                word: guess.clone(),
                found: game_state::Found::Once,
            });
            room.found_once.push(guess.clone());
            player.score += score_map[&guess.len()];
        }
    }
    let response = responses::GuessResponse::from_player(player);
    let response = serde_json::to_string_pretty(&response).unwrap();
    room.expiration_time = Instant::now() + Duration::from_secs(LIFETIME_SECONDS);
    actix_rt::spawn(check_cleanup(state.clone(), String::from(room_id)));
    HttpResponse::Ok().json(response)
}

async fn check_cleanup(state: web::Data<Arc<game_state::GameState>>, token: String) {
    actix_rt::time::delay_for(Duration::from_secs(LIFETIME_SECONDS)).await;
    let expiration_time = state.games.get(&token).unwrap().expiration_time;
    if Instant::now() >= expiration_time {
        let game = &state.games.get(&token).unwrap();
        println!(
            " --Session {} dropped-- \n
            =====results=====\n
            Board:\n
            {}\n
            Solutions: \n
            {:#?}\n
            Results:\n
            {:#?} \n
            =======end=======",
            &token, &game.board, &game.solutions, &game.players
        );
        state.games.remove(&token);
    }
}

pub async fn poll_lobby(
    req: HttpRequest,
    trie: web::Data<bog::TrieNode>,
    state: web::Data<Arc<game_state::GameState>>,
) -> impl Responder {
    //player list
    let room_id = req.match_info().get("room_id").unwrap();
    let button_state = req.match_info().get("button_state").unwrap();
    let button_state = format!("\"{}\"", button_state);
    let button_state: responses::ButtonState = serde_json::from_str(&button_state).unwrap();
    let mut room = if let Some(room) = state.games.get_mut(room_id) {
        room
    } else {
        return HttpResponse::BadRequest()
            .reason("Room not found")
            .body("Room not found");
    };
    room.activate(&trie, &button_state);
    let response = responses::LobbyPollResponse::from_room(&room);
    let response = serde_json::to_string_pretty(&response).unwrap();
    room.expiration_time = Instant::now() + Duration::from_secs(LIFETIME_SECONDS);
    actix_rt::spawn(check_cleanup(state.clone(), String::from(room_id)));
    HttpResponse::Ok().json(response)
}

pub async fn poll_game(
    req: HttpRequest,
    score_map: web::Data<HashMap<usize, usize>>,
    state: web::Data<Arc<game_state::GameState>>,
) -> impl Responder {
    let room_id = req.match_info().get("room_id").unwrap();
    let player_id = req.match_info().get("player_id").unwrap();
    let mut room = if let Some(room) = state.games.get_mut(room_id) {
        room
    } else {
        return HttpResponse::BadRequest()
            .reason("Room not found")
            .body("Room not found");
    };
    let mut room = room.value_mut();
    let mut player = if let Some(player) = room.players.get_mut(player_id) {
        player
    } else {
        return HttpResponse::BadRequest()
            .reason("Player not found")
            .body("Player not found");
    };
    for i in 0..player.valid_guesses.len() {
        let mut existing_guess = player.valid_guesses.get_mut(i).unwrap();
        if existing_guess.found == game_state::Found::Once {
            if room.found_twice.contains(&existing_guess.word) {
                existing_guess.found = game_state::Found::Twice;
                player.score -= score_map[&existing_guess.word.len()];
            }
        }
    }
    if (Instant::now() - room.start_time).as_secs() > 2 * 60 {
        room.status = GameStatus::InLobby;
    }
    let response =
        responses::GamePollResponse::from_room_and_player(&String::from(player_id), &room);
    let response = serde_json::to_string_pretty(&response).unwrap();
    HttpResponse::Ok().json(response)
}

pub async fn single_player(
    tera: web::Data<Tera>,
    trie: web::Data<bog::TrieNode>,
    state: web::Data<Arc<game_state::GameState>>,
) -> impl Responder {
    let mut data = Context::new();
    let session_token = state.new_session_single();
    let mut game = state.games.get_mut(&session_token).unwrap();
    println!(" --Single-player room {} created-- ", &session_token);
    data.insert("title", "BogChamp");
    data.insert("rows", &game.board.letters);
    let solution_set = game.board.solve(trie.get_ref());
    let solutions: Vec<String> = solution_set.into_iter().collect();
    game.solutions = solutions.clone();
    let sorted = format_solutions(&solutions);
    data.insert("solutions", &sorted);
    data.insert("n_solutions", &solutions.len());
    data.insert("session_token", &session_token);
    let rendered = tera.render("single_player.html", &data).unwrap();
    game.expiration_time = Instant::now() + Duration::from_secs(LIFETIME_SECONDS);
    actix_rt::spawn(check_cleanup(state.clone(), session_token));
    HttpResponse::Ok().body(rendered)
}
pub async fn multi_player(
    req: HttpRequest,
    tera: web::Data<Tera>,
    trie: web::Data<bog::TrieNode>,
    state: web::Data<Arc<game_state::GameState>>,
) -> impl Responder {
    let mut data = Context::new();
    let room_id = req.match_info().get("room_id").unwrap();
    let player_id = req.match_info().get("player_id").unwrap();
    let mut room = if let Some(room) = state.games.get_mut(room_id) {
        room
    } else {
        return HttpResponse::BadRequest()
            .reason("Room not found")
            .body("Room not found");
    };
    let room = room.value_mut();
    let player = if let Some(player) = room.players.get_mut(player_id) {
        player
    } else {
        return HttpResponse::BadRequest()
            .reason("Player not found")
            .body("Player not found");
    };
    let solution_set = room.board.solve(&trie);
    let solutions: Vec<String> = solution_set.into_iter().collect();
    room.solutions = solutions.clone();
    let sorted = format_solutions(&solutions);
    data.insert("title", &format!("BogChamp: {}", &room.name));
    data.insert("solutions", &sorted);
    data.insert("n_solutions", &solutions.len());
    data.insert("room_name", &room.name);
    data.insert("player_name", &player.name);
    data.insert("room_id", room_id);
    data.insert("player_id", &player_id);
    data.insert("rows", &room.board.letters);
    let rendered = tera.render("multi_player.html", &data).unwrap();
    room.expiration_time = Instant::now() + Duration::from_secs(LIFETIME_SECONDS);
    actix_rt::spawn(check_cleanup(state.clone(), String::from(room_id)));
    HttpResponse::Ok().body(rendered)
}
pub async fn lobby(
    req: HttpRequest,
    tera: web::Data<Tera>,
    trie: web::Data<bog::TrieNode>,
    state: web::Data<Arc<game_state::GameState>>,
) -> impl Responder {
    let room_id = req.match_info().get("room_id").unwrap();
    let player_id = req.match_info().get("player_id").unwrap();
    let mut data = Context::new();
    let mut room = if let Some(room) = state.games.get_mut(room_id) {
        room
    } else {
        return HttpResponse::BadRequest()
            .reason("Room not found")
            .body("Room not found");
    };
    let room = room.value_mut();
    let player = if let Some(player) = room.players.get_mut(player_id) {
        player
    } else {
        return HttpResponse::BadRequest()
            .reason("Player not found")
            .body("Player not found");
    };
    let room_name = &room.name;
    let player_name = &player.name;
    let solution_set = room.board.solve(trie.get_ref());
    let solutions: Vec<String> = solution_set.into_iter().collect();
    room.solutions = solutions.clone();
    let sorted = format_solutions(&solutions);
    data.insert("title", &format!("BogChamp: {}", room_name));
    data.insert("room_name", room_name);
    data.insert("room_id", room_id);
    data.insert("player_id", player_id);
    data.insert("player_name", player_name);
    data.insert("rows", &room.board.letters);
    data.insert("solutions", &sorted);
    data.insert("n_solutions", &solutions.len());
    let rendered = tera.render("lobby.html", &data).unwrap();
    HttpResponse::Ok().body(rendered)
}
pub async fn index(tera: web::Data<Tera>) -> impl Responder {
    let mut data = Context::new();
    data.insert("title", "BogChamp");
    let rendered = tera.render("index.html", &data).unwrap();
    HttpResponse::Ok().body(rendered)
}

pub async fn create_lobby(
    req: HttpRequest,
    state: web::Data<Arc<game_state::GameState>>,
) -> impl Responder {
    let room_name = req.match_info().get("room_name").unwrap();
    let room_id = state.new_session_multi(String::from(room_name));
    // let room_ids: Vec<String> = game_state.games.keys().cloned().collect();
    actix_rt::spawn(check_cleanup(state.clone(), room_id.clone()));
    println!(" --Multiplayer room {} created-- ", room_id);
    HttpResponse::Ok().json(room_id)
}
pub async fn insert_player(
    req: HttpRequest,
    state: web::Data<Arc<game_state::GameState>>,
) -> impl Responder {
    let room_id = req.match_info().get("room_id").unwrap_or("");
    let player_name = req.match_info().get("player_name").unwrap_or("");
    if let Some(mut room) = state.games.get_mut(room_id) {
        let room = room.value_mut();
        let player_id = room.new_player(String::from(player_name));
        println!("Player {} inserted into room {}.", room_id, player_id);
        return HttpResponse::Ok().json(player_id);
    }
    HttpResponse::BadRequest()
        .reason("Room not found")
        .body("Room not found")
}

fn format_solutions(solutions: &[String]) -> Vec<Vec<String>> {
    let n_columns: usize = 5;
    let mut sorted = solutions.to_owned();
    sorted.sort_by_key(|x| std::cmp::Reverse(x.len()));
    let mut formatted: Vec<Vec<String>> = Vec::new();
    for _ in 0..((sorted.len() / n_columns) as f64).ceil() as isize + 1 {
        let mut row: Vec<String> = Vec::new();
        for _ in 0..n_columns {
            if let Some(word) = sorted.pop() {
                row.push(word);
            } else {
                break;
            }
        }
        formatted.push(row);
    }
    formatted
}
