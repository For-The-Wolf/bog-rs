use super::bog::read_lines;
use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};

#[derive(Debug)]
pub enum Reason {
    NotInDict,
    Informal,
    Regional,
    PropperNoun,
}
#[derive(Debug)]
pub enum IsValid {
    Valid,
    Invalid { reason: Reason },
    Unsure,
}

fn load_credentials() -> Option<[String; 2]> {
    let mut credentials: Vec<String> = Vec::new();
    if let Ok(mut lines) = read_lines("api_key.txt") {
        for _ in 0..2 {
            if let Some(Ok(credential)) = lines.next() {
                &credentials.push(credential);
            }
        }
        if credentials.len() == 2 {
            let app_id = credentials[0].clone();
            let app_key = credentials[1].clone();
            return Some([app_id, app_key]);
        }
    }
    return None;
}

fn retrive_word(
    word: &String,
    app_id: &String,
    app_key: &String,
) -> Result<String, reqwest::Error> {
    let language = "en-gb";
    let url = format!(
        "https://od-api.oxforddictionaries.com:443/api/v2/entries/{}/{}",
        language, word
    );
    let client = reqwest::blocking::Client::new();
    let res = client
        .get(url)
        .header("app_id", app_id)
        .header("app_key", app_key)
        .send()?
        .text()?;
    Ok(res)
}

fn is_child_word(
    word: &String,
    app_id: &String,
    app_key: &String,
) -> Result<String, reqwest::Error> {
    let language = "en-gb";
    let url = format!(
        "https://od-api.oxforddictionaries.com:443/api/v2/lemmas/{}/{}",
        language, word
    );
    let client = reqwest::blocking::Client::new();
    let res = client
        .get(url)
        .header("app_id", app_id)
        .header("app_key", app_key)
        .send()?
        .text()?;
    Ok(res)
}

pub fn check_word(word: String) -> IsValid {
    println!("{:?}", word);
    if let Some([app_id, app_key]) = load_credentials() {
        if let Ok(word_data) = retrive_word(&word, &app_id, &app_key) {
            let parsed_data: serde_json::Value =
                serde_json::from_str(&word_data).unwrap_or_else(|_| {
                    serde_json::from_str(
                        r#"
                {
                    "error": "Unwrapping failed"
                }
            "#,
                    )
                    .unwrap()
                });
            match parsed_data["error"] {
                serde_json::Value::Null => {
                    //TODO
                    return IsValid::Valid;
                }
                _ => {
                    if let Ok(child_data) = is_child_word(&word, &app_id, &app_key) {
                        let parsed_child: serde_json::Value = serde_json::from_str(&child_data)
                            .unwrap_or_else(|_| serde_json::Value::Null);
                        if let serde_json::Value::String(parent) = &parsed_child["results"][0]
                            ["lexicalEntries"][0]["inflectionOf"][0]["text"]
                        {
                            return check_word(parent.clone());
                        }
                    }
                }
            }
        }
        return IsValid::Invalid {
            reason: Reason::NotInDict,
        };
    }
    IsValid::Unsure
}

#[test]
fn gets_result() {
    if let Some([a, b]) = load_credentials() {
        if let Ok(_) = retrive_word(String::from("egg"), a, b) {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
