use dirs::{cache_dir, home_dir};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::value::Serializer;

use std::fs;
use std::path::Path;
use std::process::exit;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, Deserialize, Debug)]
struct Coord {
    lat: f32,
    lon: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Weather {
    id: u32,
    main: String,
    description: String,
    icon: String,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
struct Wind {
    speed: f32,
    deg: f32,
    gust: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Rain {
    #[serde(rename = "1h")]
    one_h: Option<f32>,
    #[serde(rename = "3h")]
    tree_h: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Clouds {
    all: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Sys {
    #[serde(rename = "type")]
    _type: u32,
    id: u32,
    country: String,
    sunrise: u32,
    sunset: u32,
}

#[derive(Serialize, Deserialize, Debug)]
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
//TODO test
//TODO cache to file + https://crates.io/crates/dirs + version cache file + embeed timestamp +
//separate method

fn main() {
    let client = Client::new();

    let lat: f32 = 50.11;
    let lon: f32 = 19.92;

    let api_key_dir = home_dir().unwrap();
    let api_key_name: &str = "/.owm-key";
    let api_key_path = format!("{}{}", api_key_dir.display(), api_key_name);
    let api_key = fs::read_to_string(&api_key_path).unwrap_or_else(|_| {
        eprintln!(
            "{}",
            format!("Error: no api key present in path: {}", &api_key_path).to_owned()
        );
        exit(0)
    });

    let lang = String::from("pl");
    let units = String::from("metric");

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&lang={}&units={}&appid={}",
        lat, lon, lang, units, api_key
    );

    let cache_dir = cache_dir().unwrap();
    let cache_file_name = String::from("/stormwind.cache");
    let cache_path = format!("{}{}", cache_dir.display(), cache_file_name);

    if Path::new(&cache_path).exists() {
        println!("Cache exists");

        let cache_contents = fs::read_to_string(&cache_path).unwrap();

        let report: WeatherReportCurrent = serde_json::from_str(&cache_contents).unwrap();

        println!("{}", format_output(&report));
        exit(0)
    }

    match client.get(url).send() {
        Ok(response) => {
            let report: WeatherReportCurrent = response.json().unwrap();
            println!("{}", format_output(&report));

            fs::write(
                cache_path,
                report.serialize(Serializer).unwrap().to_string(),
            )
            .unwrap();
        }
        Err(_) => exit(0),
    };
}

fn format_output(report: &WeatherReportCurrent) -> String {
    let temp = report.main.feels_like;
    let wind_speed = report.wind.speed;
    format!("Temperature: {}C, Wind Speed: {}m/s", temp, wind_speed)
}
