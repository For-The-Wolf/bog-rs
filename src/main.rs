#![allow(dead_code, unused_imports)]
use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use tera::{Context, Tera};
// use std::time::Instant;

mod bog;
mod ode_check;

async fn index(tera: web::Data<Tera>, trie: web::Data<bog::TrieNode>) -> impl Responder {
    let mut data = Context::new();
    let game = bog::BoggleBoard::new();
    data.insert("title", "BogChamp");
    data.insert("rows", &game.letters);
    let solutions = game.solve(trie.get_ref());
    let mut sorted = solutions.into_iter().collect::<Vec<String>>();
    sorted.sort_by(|x, y| x.len().cmp(&y.len()));
    data.insert("solutions", &sorted);
    data.insert("n_solutions", &sorted.len());
    data.insert("test_letter", "q");
    let rendered = tera.render("index.html", &data).unwrap();
    HttpResponse::Ok().body(rendered)
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let tera = Tera::new("templates/**/*.html").unwrap();
        let dictionary = bog::TrieNode::build_trie("./dict/other_word_list.txt");
        App::new()
            .data(tera)
            .data(dictionary)
            .route("/", web::get().to(index))
            .service(fs::Files::new("/letters", "./templates/letters/").show_files_listing())
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
