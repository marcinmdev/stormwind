use reqwest::blocking::Client;
use serde_json::Value;
use std::process::exit;

const URL: &str = "https://jsonplaceholder.typicode.com/todos/1";

fn main() {
    let client = Client::new();
    match client.get(self::URL).send() {
        Ok(response) => {
            let json: Value = response.json().unwrap();
            println!(
                "userId: {}, id: {}, title: {}, completed: {}",
                json["userId"], json["id"], json["title"], json["completed"]
            )
        }
        Err(_) => exit(0),
    };
}
