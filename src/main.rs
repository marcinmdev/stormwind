use clap::Parser;
use dirs::home_dir;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use std::fs;
use std::process::exit;
use serde_json::{json, Value};
use crate::report::WeatherReportCurrent;

mod report;

//TODO integration test
//TODO readme

#[derive(clap::ValueEnum, Clone, Debug, Deserialize, strum::Display)]
#[serde(rename_all = "snake_case")]
enum Units {
    Standard,
    Metric,
    Imperial,
}

#[derive(Parser, Deserialize, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, help = "Latitude of location")]
    lat: f32,

    #[arg(long, help = "Longitude of location")]
    lon: f32,

    #[arg(long, default_value = "en")]
    lang: String,

    #[arg(long, value_enum, default_value_t=Units::Metric)]
    units: Units,

    #[arg(long, default_value = "$HOME/.owm-key")]
    key_path: String,
}

#[derive(Serialize, Debug)]
struct WaybarOutput {
    text: String,
    alt: String,
    tooltip: String,
    class: String,
    percentage: i8,
}

fn main() {
    let client = Client::new();

    let args = Args::parse();

    let api_key_path_default = "$HOME/.owm-key";

    let mut api_key_path = args.key_path;

    if api_key_path == api_key_path_default {
        let api_key_dir = home_dir().unwrap();
        let api_key_name = ".owm-key";
        api_key_path = format!("{}/{}", api_key_dir.display(), api_key_name);
    }

    let api_key = fs::read_to_string(&api_key_path).unwrap_or_else(|_| {
        eprintln!("Error: no api key present in path: {}", &api_key_path);
        exit(0)
    });

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&lang={}&units={}&appid={}",
        args.lat, args.lon, args.lang, args.units, api_key
    );

    match client.get(url).send() {
        Ok(response) => {
            let report: WeatherReportCurrent = response.json().expect("Invalid response from API");

            println!("{}", format_output(&report));
        }
        Err(_) => eprintln!("Connection/api key error"),
    };
}

fn format_output(report: &WeatherReportCurrent) -> Value {
    let temp = report.main.feels_like;

    let icon = match &report.weather[0].icon as &str {
        "01d" => "",
        "01n" => "",
        "02d" => "",
        "02n" => "",
        "03d" => "󰖐",
        "03n" => "󰖐",
        "04d" => "󰖐",
        "04n" => "󰖐",
        "09d" => "",
        "09n" => "",
        "10d" => "",
        "10n" => "",
        "11d" => "",
        "11n" => "",
        "13d" => "",
        "13n" => "",
        "50d" => "",
        "50n" => "",
        _ => "",
    };

    let waybar_output = WaybarOutput{
        text: format!("{} {}°", &icon, &temp.round().abs()),
        alt: String::from("test alt"),
        class: String::from("randomClass"),
        percentage: 13,
        tooltip: String::from("test tooltip"),
    };

    json!(waybar_output)
}

#[cfg(test)]
mod tests {
    #[test]
    fn internal() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
