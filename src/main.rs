use actix_files as fs;
use actix_web::{web, App, HttpServer};
use std::collections::HashMap;
use std::sync::Arc;
use tera::Tera;

mod bog;
mod game_state;
mod responses;
mod web_server;
// mod ode_check;

#[cfg(test)]
mod tests {
    use crate::bog::{BoggleBoard, TrieNode};

    #[test]
    fn test_qu() {
        let letters = Vec::from([
            Vec::from(['e', 'q', 'i', 'a']),
            Vec::from(['e', 'r', 'r', 'b']),
            Vec::from(['n', 'x', 'e', 'c']),
            Vec::from(['a', 'z', 'u', 'd']),
        ]);
        let test_board = BoggleBoard { letters };
        let trie = TrieNode::build_trie("./dict/other_word_list.txt");
        let solutions = test_board.solve(&trie);
        assert!(
            solutions.iter().any(|word| word == "require")
                && solutions.iter().any(|word| word == "queen")
        )
    }
    #[test]
    fn test_trie() {
        let trie = TrieNode::build_trie("./dict/other_word_list.txt");
        assert!(trie._find_word(String::from("require")).is_some());
        assert!(trie._find_word(String::from("robe")).is_some());
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let game_state = web::Data::new(Arc::new(game_state::GameState::new()));
    HttpServer::new(move || {
        let tera = Tera::new("templates/**/*.html").unwrap();
        let dictionary = bog::TrieNode::build_trie("./dict/other_word_list.txt");
        let score_map: HashMap<usize, usize> = game_state::SCORES.iter().cloned().collect();
        App::new()
            .app_data(game_state.clone())
            .data(tera)
            .data(score_map)
            .data(dictionary)
            .route("/", web::get().to(web_server::index))
            .route("/single_player", web::get().to(web_server::single_player))
            .route(
                "/eval_guess/{room}/{player}/{word}",
                web::post().to(web_server::eval_guess),
            )
            .route(
                "/create_room/{room_name}",
                web::post().to(web_server::create_lobby),
            )
            .route(
                "/insert_player/{room_id}/{player_name}",
                web::post().to(web_server::insert_player),
            )
            .route(
                "/lobby/{room_id}/{player_id}",
                web::get().to(web_server::lobby),
            )
            .route(
                "/poll_lobby/{button_state}/{room_id}",
                web::get().to(web_server::poll_lobby),
            )
            .route(
                "/multi_player/{room_id}/{player_id}",
                web::get().to(web_server::multi_player),
            )
            .route(
                "/poll_game/{room_id}/{player_id}",
                web::post().to(web_server::poll_game),
            )
            .service(fs::Files::new("/images", "./templates/images/"))
            .service(fs::Files::new(
                "/images/buttons",
                "./templates/images/buttons/",
            ))
            .service(fs::Files::new(
                "/images/letters",
                "./templates/images/letters/",
            ))
            .service(fs::Files::new("/", "./templates/"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
