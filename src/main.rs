use actix_files as fs;
use actix_rt::time::Instant;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::iter;
use std::sync::Mutex;
use std::time::Duration;
use tera::{Context, Tera};

mod bog;
// mod ode_check;
static LIFETIME_SECONDS: u64 = 3 * 60;
static SCORES: [(usize, usize); 13] = [
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

#[derive(Debug)]
struct Player {
    valid_guesses: VecDeque<String>,
    score: usize,
}

struct Game {
    board: bog::BoggleBoard,
    solutions: Vec<String>,
    found_words: Vec<String>,
    players: HashMap<String, Player>,
    expiration_time: Instant,
}

impl Game {
    fn new() -> Self {
        let board = bog::BoggleBoard::new();
        let solutions = Vec::<String>::new();
        let found_words = Vec::<String>::new();
        let players = HashMap::<String, Player>::new();
        let expiration_time = Instant::now();
        Game {
            board,
            solutions,
            found_words,
            players,
            expiration_time,
        }
    }
}

struct GameState {
    games: HashMap<String, Game>,
}

impl GameState {
    fn new() -> Self {
        GameState {
            games: HashMap::<String, Game>::new(),
        }
    }
    fn new_session_single(&mut self) -> String {
        let mut rng = thread_rng();
        let token: String = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .map(char::from)
            .take(12)
            .collect();
        self.games.insert(token.clone(), Game::new());
        self.games.get_mut(&token).unwrap().players.insert(
            String::from("single_player"),
            Player {
                valid_guesses: VecDeque::<String>::new(),
                score: 0,
            },
        );
        token
    }
    fn forget(&mut self, token: &str) {
        self.games.remove(token);
    }
}

fn format_solutions(solutions: &Vec<String>) -> Vec<Vec<String>> {
    let n_columns: usize = 5;
    let mut sorted = solutions.clone();
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
fn check_guess(guess: String, solutions: &Vec<String>) -> bool {
    if solutions.iter().any(|word| word == &guess) {
        return true;
    }
    false
}
fn lst_to_json(words: VecDeque<String>, score: usize) -> String {
    let mut json = String::from(
        r#"
        {
            "words" : ["#,
    );
    for (n, word) in words.iter().enumerate() {
        if n == words.len() - 1 {
            json.push_str(&format!("\"{}\"\n", word));
        } else {
            json.push_str(&format!("\"{}\",\n", word));
        }
    }
    json.push_str(
        r#"
            ],
        "score":"#,
    );
    json.push_str(&format!("{}", score));
    json.push_str(
        r#"
        }
        "#,
    );
    json
}

async fn eval_guess(
    req: HttpRequest,
    score_map: web::Data<HashMap<usize, usize>>,
    state: web::Data<Mutex<GameState>>,
) -> impl Responder {
    let guess = &req.match_info().get("word").unwrap_or("").to_lowercase();
    let session_token = req.match_info().get("room").unwrap_or("");
    let player_name = req.match_info().get("player").unwrap_or("");
    let mut game_state = state.lock().unwrap();
    if game_state
        .games
        .get(session_token)
        .unwrap()
        .players
        .get(player_name)
        .is_some()
    {
        if check_guess(
            String::from(guess),
            &game_state.games[session_token].solutions,
        ) && !&game_state.games[session_token]
            .found_words
            .iter()
            .any(|word| word == guess)
        {
            game_state
                .games
                .get_mut(session_token)
                .unwrap()
                .players
                .get_mut(player_name)
                .unwrap()
                .score += score_map[&guess.len()];
            game_state
                .games
                .get_mut(session_token)
                .unwrap()
                .players
                .get_mut(player_name)
                .unwrap()
                .valid_guesses
                .push_front(String::from(guess));
            game_state
                .games
                .get_mut(session_token)
                .unwrap()
                .found_words
                .push(String::from(guess));
        }
        let guesses = &game_state.games[session_token].players[player_name].valid_guesses;
        let score = &game_state.games[session_token].players[player_name].score;
        let json = lst_to_json(guesses.clone(), *score);
        game_state
            .games
            .get_mut(session_token)
            .unwrap()
            .expiration_time = Instant::now() + Duration::from_secs(LIFETIME_SECONDS);
        actix_rt::spawn(check_cleanup(state.clone(), String::from(session_token)));
        return HttpResponse::Ok().body(json);
    } else {
        return HttpResponse::Ok().body(String::from(
            r#"{"error":"This session has ended, refresh the page."}"#,
        ));
    }
}

async fn check_cleanup(state: web::Data<Mutex<GameState>>, token: String) {
    actix_rt::time::delay_for(Duration::from_secs(LIFETIME_SECONDS)).await;
    let mut game_state = state.lock().unwrap();
    let expiration_time = game_state.games[&token].expiration_time;
    if Instant::now() >= expiration_time {
        let game = &game_state.games[&token];
        println!(" --Session {} dropped-- \n=====results=====\n Board:\n{}\n Solutions: \n{:#?}\n Results:\n{:#?} \n=======end=======",&token, &game.board, &game.solutions, &game.players);
        game_state.forget(&token);
    }
}

async fn single_player(
    tera: web::Data<Tera>,
    trie: web::Data<bog::TrieNode>,
    state: web::Data<Mutex<GameState>>,
) -> impl Responder {
    let mut data = Context::new();
    let mut game_state = state.lock().unwrap();
    let session_token = game_state.new_session_single();
    println!(" --Session {} created-- ", &session_token);
    data.insert("title", "BogChamp");
    data.insert("rows", &game_state.games[&session_token].board.letters);
    let solution_set = game_state.games[&session_token].board.solve(trie.get_ref());
    let solutions: Vec<String> = solution_set.into_iter().collect();
    game_state.games.get_mut(&session_token).unwrap().solutions = solutions.clone();
    let sorted = format_solutions(&solutions);
    data.insert("solutions", &sorted);
    data.insert("n_solutions", &solutions.len());
    data.insert("session_token", &session_token);
    let rendered = tera.render("single_player.html", &data).unwrap();
    game_state
        .games
        .get_mut(&session_token)
        .unwrap()
        .expiration_time = Instant::now() + Duration::from_secs(LIFETIME_SECONDS);
    actix_rt::spawn(check_cleanup(state.clone(), session_token));
    HttpResponse::Ok().body(rendered)
}
async fn index(tera: web::Data<Tera>) -> impl Responder {
    let mut data = Context::new();
    data.insert("title", "BogChamp");
    let rendered = tera.render("index.html", &data).unwrap();
    HttpResponse::Ok().body(rendered)
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let game_state = web::Data::new(Mutex::new(GameState::new()));
    HttpServer::new(move || {
        let tera = Tera::new("templates/**/*.html").unwrap();
        let dictionary = bog::TrieNode::build_trie("./dict/other_word_list.txt");
        let score_map: HashMap<usize, usize> = SCORES.iter().cloned().collect();
        App::new()
            .app_data(game_state.clone())
            .data(tera)
            .data(score_map)
            .data(dictionary)
            .route("/", web::get().to(index))
            .route("/single_player", web::get().to(single_player))
            .route(
                "/eval_guess/{room}/{player}/{word}",
                web::post().to(eval_guess),
            )
            .service(fs::Files::new("/letters", "./templates/letters/"))
            .service(fs::Files::new("/", "./templates/"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
