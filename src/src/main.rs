use actix_files as fs;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use bog::BoggleBoard;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Mutex;
use tera::{Context, Tera};

mod bog;
mod ode_check;

#[derive(Debug, Deserialize)]
struct Word {
    word: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WordList {
    words: Vec<String>,
}

impl WordList {
    fn new() -> Self {
        let words: Vec<String> = Vec::new();
        WordList { words }
    }
}
fn format_solutions(solutions: &HashSet<String>) -> Vec<Vec<String>> {
    let n_columns: usize = 5;
    let mut sorted = solutions.clone().into_iter().collect::<Vec<String>>();
    sorted.sort_by(|y, x| x.len().cmp(&y.len()));
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
fn lst_to_json(words: Vec<String>) -> String {
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
            ]
        }
        "#,
    );
    json
}

async fn index(
    tera: web::Data<Tera>,
    trie: web::Data<bog::TrieNode>,
    game: web::Data<Mutex<bog::BoggleBoard>>,
    guesses: web::Data<Mutex<WordList>>,
) -> impl Responder {
    let mut data = Context::new();
    let mut game = game.lock().unwrap();
    game._randomise();
    let mut guesses = guesses.lock().unwrap();
    guesses.words = Vec::new();
    data.insert("title", "BogChamp");
    data.insert("rows", &game.letters);
    let solution_set = game.solve(trie.get_ref());
    let sorted = format_solutions(&solution_set);
    data.insert("solutions", &sorted);
    data.insert("n_solutions", &solution_set.len());
    let rendered = tera.render("index.html", &data).unwrap();
    HttpResponse::Ok().body(rendered)
}

async fn eval_guess(
    req: HttpRequest,
    game: web::Data<Mutex<bog::BoggleBoard>>,
    trie: web::Data<bog::TrieNode>,
    guesses: web::Data<Mutex<WordList>>,
) -> impl Responder {
    let guess = req.match_info().get("word").unwrap_or("");
    let game = game.lock().unwrap();
    let solutions = game
        .solve(trie.get_ref())
        .into_iter()
        .collect::<Vec<String>>();
    let mut guesses = guesses.lock().unwrap();
    if check_guess(String::from(guess), &solutions)
    //&& !&solutions.iter().any(|word| word == &guess)
    {
        guesses.words.push(String::from(guess.clone()));
    }
    let json = lst_to_json(guesses.words.clone());
    println!("{:?}", guesses.words);
    HttpResponse::Ok().body(json)
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let guesses = web::Data::new(Mutex::new(WordList::new()));
    let solutions = web::Data::new(Mutex::new(WordList::new()));
    let game = web::Data::new(Mutex::new(bog::BoggleBoard::new()));
    HttpServer::new(move || {
        let tera = Tera::new("templates/**/*.html").unwrap();
        let dictionary = bog::TrieNode::build_trie("./dict/other_word_list.txt");
        App::new()
            .app_data(game.clone())
            .app_data(guesses.clone())
            .app_data(solutions.clone())
            .data(tera)
            .data(dictionary)
            .route("/", web::get().to(index))
            .route("/eval_guess/{word}", web::get().to(eval_guess))
            .service(fs::Files::new("/letters", "./templates/letters/").show_files_listing())
            .service(fs::Files::new("/", "./templates/").show_files_listing())
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
