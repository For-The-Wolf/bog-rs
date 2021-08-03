use actix_web::{dev, Error, FromRequest, HttpRequest};
use futures::future::{ok, Ready};
use rand::distributions::WeightedIndex;
// use actix_web::error::ErrorBadRequest;
use rand::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

const WEIGHTS: [f64; 26] = [
    0.1115994349,
    0.0849748862,
    0.07581227437,
    0.07543949145,
    0.0716331816,
    0.06951420499,
    0.06655156176,
    0.05734970962,
    0.05489719039,
    0.04538141579,
    0.03631690472,
    0.03384476534,
    0.03166692827,
    0.03013655627,
    0.0300384555,
    0.02470177366,
    0.02071888244,
    0.01812902213,
    0.01777585936,
    0.01289044106,
    0.01100690629,
    0.01006513891,
    0.002903782766,
    0.002727201381,
    0.001962015382,
    0.001962015382,
];
const LETTERS: [char; 26] = [
    'e', 'a', 'r', 'i', 'o', 't', 'n', 's', 'l', 'c', 'u', 'd', 'p', 'm', 'h', 'g', 'b', 'f', 'y',
    'w', 'k', 'v', 'x', 'z', 'j', 'q',
];
#[derive(Clone)]
pub struct BoggleBoard {
    pub letters: Vec<Vec<char>>,
}

impl FromRequest for BoggleBoard {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(_req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        ok(BoggleBoard::new())
    }
}

#[derive(Clone, Debug)]
pub struct TrieNode {
    value: Option<String>,
    children: HashMap<char, TrieNode>,
}

impl FromRequest for TrieNode {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(_req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        ok(TrieNode::build_trie("./dict/new_word_list.txt"))
    }
}

impl BoggleBoard {
    pub fn new() -> BoggleBoard {
        BoggleBoard {
            letters: rand_chars(),
        }
    }
    pub fn randomise(&mut self) {
        self.letters = rand_chars();
    }

    pub fn solve(&self, trie: &TrieNode) -> HashSet<String> {
        let mut words: Vec<String> = Vec::new();
        for x in 0..4 {
            for y in 0..4 {
                self.dfs2(trie, (x,y).clone(), Vec::new(), Vec::new(),&mut words);
            }
        }
        words.into_iter().collect()
    }

    fn dfs2(
        &self,
        trie: &TrieNode,
        current_location: (usize, usize),
        visited_locations: Vec<(usize, usize)>,
        found_letters: Vec<char>,
        found_words: &mut Vec<String>,
    ) {
        let current_letter = self.extract(&current_location);
        let mut found_letters = found_letters.clone();
        let trie = if let Some(trie) = trie.children.get(&current_letter){
            found_letters.push(current_letter.clone());
            trie
        } else {
            return;
        };
        let trie = {
            if current_letter == 'q'{
                let trie = if let Some(trie) = trie.children.get(&'u'){
                    found_letters.push('u');
                    trie
                }else{
                    return;
                };
                trie
            }else{
                trie
            }
        };
        if let Some(word) = &trie.value{
            found_words.push(word.clone());
        }
        let mut visited_locations = visited_locations.clone();
        visited_locations.push(current_location.clone());
        let visited_set: HashSet<(usize, usize)> = visited_locations.iter().cloned().collect();
        let neighbour_set: HashSet<(usize, usize)> = BoggleBoard::adjacent(&current_location)
            .iter()
            .cloned()
            .collect();

        for coord in neighbour_set.difference(&visited_set) {
            self.dfs2(
                &trie,
                coord.clone(),
                visited_locations.clone(),
                found_letters.clone(),
                found_words
            )
        }
    }


    fn adjacent(coord: &(usize, usize)) -> Vec<(usize, usize)> {
        let (x, y) = coord.clone();
        let mut neighbours: Vec<(usize, usize)> = Vec::new();
        for xn in 0..=2 {
            for yn in 0..=2 {
                let x_new = (x + xn) as isize - 1;
                let y_new = (y + yn) as isize - 1;
                if ((x_new, y_new) != (x as isize, y as isize))
                    && x_new < 4
                    && x_new >= 0
                    && y_new < 4
                    && y_new >= 0
                {
                    neighbours.push((x_new as usize, y_new as usize));
                }
            }
        }
        neighbours
    }
    fn extract(&self, coord: &(usize, usize)) -> char {
        let (x, y) = coord.clone();
        self.letters[x][y].clone()
    }
}
impl TrieNode {
    fn new() -> TrieNode {
        let value = None;
        let children = HashMap::new();
        TrieNode { value, children }
    }

    pub fn build_trie(filename: &str) -> TrieNode {
        let mut root = TrieNode::new();
        if let Ok(lines) = read_lines(filename) {
            for line in lines {
                if let Ok(ip) = line {
                    if ip.len() > 3 {
                        root.insert_word(ip);
                    }
                }
            }
        }
        root
    }

    pub fn insert_word(&mut self, word: String) {
        let letters: VecDeque<char> = word.chars().collect();
        self.insert(letters, word);
    }

    pub fn _find_word(&self, word: String) -> Option<String> {
        let letters: VecDeque<char> = word.chars().collect();
        self._find(letters)
    }

    fn _find(&self, mut letters: VecDeque<char>) -> Option<String> {
        if let Some(letter) = letters.pop_front() {
            if self.children.keys().any(|key| key == &letter) {
                self.children.get(&letter).unwrap()._find(letters)
            } else {
                None
            }
        } else {
            self.value.clone()
        }
    }

    fn insert(&mut self, mut letters: VecDeque<char>, word: String) {
        if let Some(letter) = letters.pop_front() {
            if !self.children.keys().any(|key| key == &letter) {
                self.children.insert(letter, TrieNode::new());
            }
            self.children
                .get_mut(&letter)
                .unwrap()
                .insert(letters, word);
        } else {
            self.value = Some(word);
        }
    }
}

impl std::fmt::Display for BoggleBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut to_print = String::new();
        for row in &self.letters {
            for letter in row {
                to_print.push(' ');
                to_print.push(*letter);
                if letter == &'Q' {
                    to_print.push('u');
                } else {
                    to_print.push(' ');
                }
            }
            to_print.push('\n');
        }
        to_print.pop();
        write!(f, "{}", to_print)
    }
}

fn rand_chars() -> Vec<Vec<char>> {
    let dist = WeightedIndex::new(&WEIGHTS).unwrap();
    let mut rng = thread_rng();
    let mut letters: Vec<Vec<char>> = Vec::new();
    for _ in 0..4 {
        let mut row: Vec<char> = Vec::new();
        for _ in 0..4 {
            row.push(LETTERS[dist.sample(&mut rng)]);
        }
        letters.push(row);
    }
    letters
}

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
