#![allow(dead_code, unused_imports)]
use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpRequest, HttpServer, Responder};
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

#[derive(Serialize, Deserialize)]
struct WordList{
    words: Vec<String>,
}

impl WordList{
    fn new() -> Self{
        let words: Vec<String> = Vec::new();
        WordList{words}
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

async fn index(
    tera: web::Data<Tera>, 
    trie: web::Data<bog::TrieNode>, 
    game: web::Data<Mutex<bog::BoggleBoard>>,
) -> impl Responder {
    let mut data = Context::new();
    let mut game = game.lock().unwrap();
    game._randomise();
    data.insert("title", "BogChamp");
    data.insert("rows", &game.letters);
    let solutions = game.solve(trie.get_ref());
    let sorted = format_solutions(&solutions);
    data.insert("solutions", &sorted);
    data.insert("n_solutions", &solutions.len());
    let rendered = tera.render("index.html", &data).unwrap();
    HttpResponse::Ok().body(rendered)
}

async fn eval_guess(
    req: HttpRequest,
    _trie: web::Data<bog::TrieNode>, 
    _game: web::Data<Mutex<bog::BoggleBoard>>,
    guesses: web::Data<Mutex<WordList>>,
) -> impl Responder {
    let guess = req.match_info().get("word").unwrap_or("");
    let mut guesses = guesses.lock().unwrap();
    guesses.words.push(String::from(guess.clone()));
    HttpResponse::Ok().body(serde_json::to_string(&format!("\"words\": {:?}",guesses.words)).unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let tera = Tera::new("templates/**/*.html").unwrap();
        let dictionary = bog::TrieNode::build_trie("./dict/other_word_list.txt");
        let guesses = web::Data::new(Mutex::new(WordList::new()));
        let game = web::Data::new(Mutex::new(bog::BoggleBoard::new()));
        App::new()
            .app_data(guesses.clone())
            .app_data(game.clone())
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
