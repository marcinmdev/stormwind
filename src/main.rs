use reqwest::blocking::Client;
use std::process::exit;

fn main() {

    let client = Client::new();
    match client.get("https://www.rust-lang.org").send() {
        Ok(response) => println!("body = {response:?}"),
        Err(_) => {
            exit(0)
        }
    };
}
