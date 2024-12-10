use clap::Parser;
use dirs::{cache_dir, config_dir, home_dir};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::value::Serializer;

use filetime::FileTime;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::result::Result;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::report::WeatherReportCurrent;

mod report;

//TODO https://open-meteo.com/en/docs
//TODO tooltip with waybar support
//TODO remove config file
//TODO integration test
//TODO readme
//TODO remove default lat lon 

#[derive(clap::ValueEnum, Clone, Debug, Deserialize, strum::Display)]
#[serde(rename_all = "snake_case")]
enum Units {
    Standard,
    Metric,
    Imperial,
}

#[derive(Deserialize)]
struct ConfigFileValues {
    config: Args,
}

struct Config {
    lat: f32,
    lon: f32,
    lang: String,
    units: Units,
    cache: u16,
}

#[derive(Parser, Deserialize, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, help = "Latitude of location")]
    lat: Option<f32>,

    #[arg(long, help = "Longitude of location")]
    lon: Option<f32>,

    #[arg(long)]
    lang: Option<String>,

    #[arg(long, value_enum)]
    units: Option<Units>,

    #[arg(long, help = "Cache lifetime in seconds")]
    cache: Option<u16>,
}

fn main() {
    let client = Client::new();

    let mut config = Config {
        lat: 52.23,
        lon: 21.01,
        lang: String::from("en"),
        units: Units::Metric,
        cache: 600,
    };

    let args = Args::parse();

    if let Some(lat_from_args) = args.lat {
        config.lat = lat_from_args
    }

    if let Some(lon_from_args) = args.lon {
        config.lon = lon_from_args
    }

    if let Some(lang_from_args) = args.lang {
        config.lang = lang_from_args
    }

    if let Some(units_from_args) = args.units {
        config.units = units_from_args
    }

    if let Some(cache_from_args) = args.cache {
        config.cache = cache_from_args
    }

    let api_key_dir = home_dir().unwrap();
    let api_key_name = ".owm-key";
    let api_key_path = format!("{}/{}", api_key_dir.display(), api_key_name);
    let api_key = fs::read_to_string(&api_key_path).unwrap_or_else(|_| {
        eprintln!("Error: no api key present in path: {}", &api_key_path);
        exit(0)
    });

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&lang={}&units={}&appid={}",
        config.lat, config.lon, config.lang, config.units, api_key
    );

    let cache_path = Path::new(&cache_dir().unwrap()).join("stormwind.cache");

    if Path::new(&cache_path).exists() {
        let cache_file_metadata = fs::metadata(&cache_path).unwrap();

        let cache_mtime =
            FileTime::from_last_modification_time(&cache_file_metadata).unix_seconds();

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let cache_age = current_time - (cache_mtime as u64);

        if cache_age < config.cache.into() {
            if let Ok(report) = read_cache_file(&cache_path, &config) {
                println!("Cached response: {}", format_output(&report));
                exit(0);
            }
        }
    }

    match client.get(url).send() {
        Ok(response) => {
            let report: WeatherReportCurrent = response.json().expect("Invalid response from API");

            write_cache_file(&report, &cache_path, &config).unwrap();

            println!("{}", format_output(&report));
        }
        Err(_) => eprintln!("Connection/api key error"),
    };
}

fn format_output(report: &WeatherReportCurrent) -> String {
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

    //TODO minus 0
    //TODO handle units per config
    let output = format!("{} {}°", &icon, &temp.round().abs());

    output
}

fn write_cache_file(
    report: &WeatherReportCurrent,
    cache_path: &PathBuf,
    config: &Config,
) -> std::io::Result<()> {
    File::create(cache_path)?;

    let mut f = File::options().append(true).open(cache_path)?;
    writeln!(&mut f, env!("CARGO_PKG_VERSION"))?;
    writeln!(&mut f, "{}", config.lat)?;
    writeln!(&mut f, "{}", config.lon)?;
    writeln!(&mut f, "{}", config.lang)?;
    writeln!(&mut f, "{}", config.units)?;

    writeln!(&mut f, "{}", report.serialize(Serializer).unwrap())?;
    Ok(())
}

fn read_cache_file(
    cache_path: &PathBuf,
    config: &Config,
) -> Result<WeatherReportCurrent, &'static str> {
    let cache_contents = fs::read_to_string(cache_path).unwrap();

    if cache_contents.lines().count() < 2
        || env!("CARGO_PKG_VERSION") != cache_contents.lines().next().unwrap()
    {
        File::create(cache_path).unwrap();
        return Err("Wrong version. Cache invalidated.");
    }

    if config.lat.to_string() != cache_contents.lines().nth(1).unwrap()
        || config.lon.to_string() != cache_contents.lines().nth(2).unwrap()
        || config.lang != cache_contents.lines().nth(3).unwrap()
        || config.units.to_string() != cache_contents.lines().nth(4).unwrap()
    {
        File::create(cache_path).unwrap();
        return Err("Different config values. Cache invalidated.");
    }

    let report: WeatherReportCurrent =
        serde_json::from_str(cache_contents.lines().nth(5).unwrap()).unwrap();
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
