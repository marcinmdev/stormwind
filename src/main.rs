use home::home_dir;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use std::fs;
use std::process::exit;

#[derive(Serialize, Deserialize)]
struct Coord {
    lat: f32,
    lon: f32,
}

#[derive(Serialize, Deserialize)]
struct Weather {
    id: u32,
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
    pressure: f32,
    humidity: f32,
    sea_level: f32,
    grnd_level: f32,
}

#[derive(Serialize, Deserialize)]
struct Wind {
    speed: f32,
    deg: f32,
    gust: Option<f32>,
}

#[derive(Serialize, Deserialize)]
struct Rain {
    #[serde(rename = "1h")]
    one_h: Option<f32>,
    #[serde(rename = "3h")]
    tree_h: Option<f32>,
}

#[derive(Serialize, Deserialize)]
struct Clouds {
    all: u32,
}

#[derive(Serialize, Deserialize)]
struct Sys {
    #[serde(rename = "type")]
    _type: u32,
    id: u32,
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
    rain: Option<Rain>,
    clouds: Clouds,
    dt: u64,
    sys: Sys,
    id: u64,
    name: String,
    cod: u16,
}

//NOTE https://openweathermap.org/current
//NOTE https://github.com/BroderickCarlin/openweather/blob/master/src/weather_types.rs
//TODO format output
//TODO test
//TODO cache to file + https://crates.io/crates/dirs

fn main() {
    let client = Client::new();

    let lat: f32 = 50.11;
    let lon: f32 = 19.92;

    let api_key_dir = home_dir().unwrap();
    let api_key_name: &str = "/.owm-keyz";
    let api_key_path = format!("{}{}", api_key_dir.display(), api_key_name);
    let api_key = fs::read_to_string(&api_key_path).expect(&format!("No api key present, path: {}", &api_key_path).to_string());

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&appid={}",
        lat, lon, api_key
    );

    match client.get(url).send() {
        Ok(response) => {
            let report: WeatherReportCurrent = response.json().unwrap();
            println!("body: {}", report.base)
        }
        Err(_) => exit(0),
    };
}
