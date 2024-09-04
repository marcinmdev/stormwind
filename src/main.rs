use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::process::exit;

const URL: &str = "https://jsonplaceholder.typicode.com/todos/1";

#[derive(Serialize, Deserialize)]
struct Coord {
    lat: f32,
    lon: f32,
}

#[derive(Serialize, Deserialize)]
struct Weather {
    id: u8,
    main: String,
    description: String,
    icon: String,
}

#[derive(Serialize, Deserialize)]
struct MainWeather {
    temp: f32,
    feels_like: f32,
    temp_min: f32,
    temp_max: f32,
    pressure: u8,
    humidity: u8,
    sea_level: u8,
    grnd_level: u8,
}

#[derive(Serialize, Deserialize)]
struct Wind {
    speed: f32,
    deg: u8,
    gust: f32,
}

#[derive(Serialize, Deserialize)]
struct Rain {
    #[serde(rename = "1h")]
    one_h: f32,
}

#[derive(Serialize, Deserialize)]
struct Clouds {
    all: u8,
}

#[derive(Serialize, Deserialize)]
struct Sys {
    #[serde(rename = "type")]
    _type: u8,
    id: u8,
    country: String,
    sunrise: u32,
    sunset: u32,
}

#[derive(Serialize, Deserialize)]
pub struct WeatherReportCurrent {
    coord: Coord,
    weather: Vec<Weather>,
    base: String,
    main: MainWeather,
    visibility: u32,
    wind: Wind,
    rain: Rain,
    clouds: Clouds,
    dt: u64,
    sys: Sys,
    id: u64,
    name: String,
    cod: u16,
}

//TODO https://github.com/serde-rs/json?tab=readme-ov-file#parsing-json-as-strongly-typed-data-structures
//NOTE https://openweathermap.org/current
//NOTE https://github.com/BroderickCarlin/openweather/blob/master/src/weather_types.rs
//TODO parse api key

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
