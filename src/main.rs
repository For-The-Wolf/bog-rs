use actix_files as fs;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet};
use std::iter;
use std::ops::{Index, IndexMut};
use std::sync::Mutex;
use tera::{Context, Tera};

mod bog;
// mod ode_check;

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
struct Session {
    board: bog::BoggleBoard,
    solutions: WordList,
    valid_guesses: WordList,
    score: usize,
}

impl Session {
    fn new() -> Self {
        let board = bog::BoggleBoard::new();
        let solutions = WordList::new();
        let valid_guesses = WordList::new();
        Session {
            board,
            solutions,
            valid_guesses,
            score: 0,
        }
    }
}

struct GameState {
    sessions: HashMap<String, Session>,
}

impl GameState {
    fn new() -> Self {
        GameState {
            sessions: HashMap::new(),
        }
    }
    fn new_session(&mut self) -> String {
        let mut rng = thread_rng();
        let token: String = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .map(char::from)
            .take(12)
            .collect();
        self.sessions.insert(token.clone(), Session::new());
        token
    }
    fn forget(&mut self, token: String) -> () {
        self.sessions.remove(&token);
    }
}

#[derive(Debug)]
struct WordList {
    words: Vec<String>,
}

impl WordList {
    fn new() -> Self {
        let words: Vec<String> = Vec::new();
        WordList { words }
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
fn lst_to_json(words: Vec<String>, score: usize) -> String {
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
    game_state: web::Data<Mutex<GameState>>,
) -> impl Responder {
    let guess = req.match_info().get("word").unwrap_or("");
    let session_token = req.match_info().get("room").unwrap_or("");
    let mut game_state = game_state.lock().unwrap();
    if check_guess(
        String::from(guess),
        &game_state.sessions[session_token].solutions.words,
    ) && !&game_state.sessions[session_token]
        .valid_guesses
        .words
        .iter()
        .any(|word| word == guess)
    {
        game_state.sessions.get_mut(session_token).unwrap().score += score_map[&guess.len()];
        game_state
            .sessions
            .get_mut(session_token)
            .unwrap()
            .valid_guesses
            .words
            .push(String::from(guess));
    }
    let guesses = &game_state.sessions[session_token].valid_guesses;
    let score = &game_state.sessions[session_token].score;
    let json = lst_to_json(guesses.words.clone(), *score);
    println!("{:?}", guesses.words);
    HttpResponse::Ok().body(json)
}

async fn index(
    tera: web::Data<Tera>,
    trie: web::Data<bog::TrieNode>,
    game_state: web::Data<Mutex<GameState>>,
) -> impl Responder {
    let mut data = Context::new();
    let mut game_state = game_state.lock().unwrap();
    let session_token = game_state.new_session();
    data.insert("title", "BogChamp");
    data.insert("rows", &game_state.sessions[&session_token].board.letters);
    let solution_set = game_state.sessions[&session_token]
        .board
        .solve(trie.get_ref());
    let solutions: Vec<String> = solution_set.into_iter().collect();
    game_state
        .sessions
        .get_mut(&session_token)
        .unwrap()
        .solutions
        .words = solutions.clone();
    let sorted = format_solutions(&solutions);
    data.insert("solutions", &sorted);
    data.insert("n_solutions", &solutions.len());
    data.insert("session_token", &session_token);
    println!("{:?}", &session_token);
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
            .route("/eval_guess/{room}/{word}", web::post().to(eval_guess))
            .service(fs::Files::new("/letters", "./templates/letters/").show_files_listing())
            .service(fs::Files::new("/", "./templates/").show_files_listing())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
