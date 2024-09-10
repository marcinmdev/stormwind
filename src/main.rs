use clap::Parser;
use dirs::{cache_dir, config_dir, home_dir};
use reqwest::blocking::Client;
use serde::Serialize;
use serde_json::value::Serializer;

use filetime::FileTime;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::exit;
use std::result::Result;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::report::WeatherReportCurrent;

mod report;

//NOTE https://openweathermap.org/current
//TODO integration test
//TODO config params
//TODO readme
//
//

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    lat: Option<f32>,

    #[arg(short, long)]
    lon: Option<f32>,

    #[arg(short, long)]
    lang: Option<String>,

    #[arg(short, long)]
    units: Option<String>,
}

fn main() {
    let client = Client::new();

    let lat: f32 = 50.11;
    let lon: f32 = 19.92;

    let config_dir = config_dir().unwrap();

    let config_path = format!("{}{}", config_dir.display(), "stormwind.cfg");

    if let Ok(config_file) = fs::read_to_string(&config_path) {
        //TODO parse config values
        //https://github.com/mehcode/config-rs or serde with toml support
    }

    let api_key_dir = home_dir().unwrap();
    let api_key_name = "/.owm-key";
    let api_key_path = format!("{}{}", api_key_dir.display(), api_key_name);
    let api_key = fs::read_to_string(&api_key_path).unwrap_or_else(|_| {
        eprintln!("Error: no api key present in path: {}", &api_key_path);
        exit(0)
    });

    let lang = "pl";
    let units = "metric";

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&lang={}&units={}&appid={}",
        lat, lon, lang, units, api_key
    );

    let cache_dir = cache_dir().unwrap();
    let cache_file_name = "/stormwind.cache";
    let cache_path = format!("{}{}", cache_dir.display(), cache_file_name);

    if Path::new(&cache_path).exists() {
        let cache_file_metadata = fs::metadata(&cache_path).unwrap();

        let cache_mtime =
            FileTime::from_last_modification_time(&cache_file_metadata).unix_seconds();

        let cache_lifetime = 600;

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let cache_age = current_time - (cache_mtime as u64);

        if cache_age < cache_lifetime {
            if let Ok(report) = read_cache_file(&cache_path) {
                println!("Cached response: {}", format_output(&report));
                exit(0);
            }
        }
    }

    match client.get(url).send() {
        Ok(response) => {
            let report: WeatherReportCurrent = response.json().unwrap();

            write_cache_file(&report, &cache_path).unwrap();

            println!("{}", format_output(&report));
        }
        Err(_) => eprintln!("Connection/api key error"),
    };
}

fn format_output(report: &WeatherReportCurrent) -> String {
    let temp = report.main.feels_like;
    let wind_speed = report.wind.speed;
    format!("Temperature: {}C, Wind Speed: {}m/s", temp, wind_speed)
}

fn write_cache_file(report: &WeatherReportCurrent, cache_path: &String) -> std::io::Result<()> {
    File::create(cache_path)?;

    let mut f = File::options().append(true).open(cache_path)?;
    writeln!(&mut f, env!("CARGO_PKG_VERSION"))?;
    writeln!(&mut f, "{}", report.serialize(Serializer).unwrap())?;
    Ok(())
}

fn read_cache_file(cache_path: &String) -> Result<WeatherReportCurrent, &'static str> {
    let cache_contents = fs::read_to_string(cache_path).unwrap();

    if cache_contents.lines().count() < 2
        || env!("CARGO_PKG_VERSION") != cache_contents.lines().next().unwrap()
    {
        File::create(cache_path).unwrap();
        return Err("Wrong version");
    }

    let report: WeatherReportCurrent =
        serde_json::from_str(cache_contents.lines().nth(1).unwrap()).unwrap();
    Ok(report)
}

#[cfg(test)]
mod tests {
    #[test]
    fn internal() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
