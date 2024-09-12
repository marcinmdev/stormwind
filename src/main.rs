use clap::Parser;
use dirs::{cache_dir, config_dir, home_dir};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::value::Serializer;

use strum::Display;

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
//TODO more elegant arg parsing
//TODO integration test
//TODO readme

#[derive(clap::ValueEnum, Clone, Debug, Deserialize, Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum Units {
    Standard,
    Metric,
    Imperial,
}

#[derive(Deserialize)]
struct Config {
    config: Args,
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

    let config_dir = config_dir().unwrap();

    let config_file_name = "stormwind.toml";
    let config_path = format!("{}/stormwind/{}", config_dir.display(), &config_file_name);

    let mut config = Args {
        lat: Some(52.23),
        lon: Some(21.01),
        lang: Some("en".to_string()),
        units: Some(Units::Standard),
        cache: Some(600),
    };

    if let Ok(config_file_content) = fs::read_to_string(&config_path) {
        let config_data: Config = match toml::from_str(&config_file_content) {
            Ok(d) => d,
            Err(_) => {
                eprintln!("Unable to load config from `{}`", &config_path);
                exit(1);
            }
        };

        if let Some(lat_from_config) = config_data.config.lat {
            config.lat = Some(lat_from_config)
        }

        if let Some(lon_from_config) = config_data.config.lon {
            config.lon = Some(lon_from_config)
        }

        if let Some(lang_from_config) = config_data.config.lang {
            config.lang = Some(lang_from_config)
        }

        if let Some(units_from_config) = config_data.config.units {
            config.units = Some(units_from_config)
        }
        if let Some(cache_from_config) = config_data.config.cache {
            config.cache = Some(cache_from_config)
        }
    }

    let args = Args::parse();

    if let Some(lat_from_args) = args.lat {
        config.lat = Some(lat_from_args)
    }

    if let Some(lon_from_args) = args.lon {
        config.lon = Some(lon_from_args)
    }

    if let Some(lang_from_args) = args.lang {
        config.lang = Some(lang_from_args)
    }

    if let Some(units_from_args) = args.units {
        config.units = Some(units_from_args)
    }

    if let Some(cache_from_args) = args.cache {
        config.cache = Some(cache_from_args)
    }

    println!(
        "config values: {:?} {:?} {:?} {:?} {:?}",
        config.lat,
        config.lon,
        config.lang,
        config.units,
        config.cache
    );

    let api_key_dir = home_dir().unwrap();
    let api_key_name = ".owm-key";
    let api_key_path = format!("{}/{}", api_key_dir.display(), api_key_name);
    let api_key = fs::read_to_string(&api_key_path).unwrap_or_else(|_| {
        eprintln!("Error: no api key present in path: {}", &api_key_path);
        exit(0)
    });

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?lat={:?}&lon={:?}&lang={:?}&units={:?}&appid={:?}",
        config.lat, config.lon, config.lang, config.units, api_key
    );

    let cache_dir = cache_dir().unwrap();
    let cache_file_name = "/stormwind.cache";
    let cache_path = format!("{}{}", cache_dir.display(), cache_file_name);

    if Path::new(&cache_path).exists() {
        let cache_file_metadata = fs::metadata(&cache_path).unwrap();

        let cache_mtime =
            FileTime::from_last_modification_time(&cache_file_metadata).unix_seconds();

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let cache_age = current_time - (cache_mtime as u64);

        if cache_age < config.cache.unwrap().into() {
            if let Ok(report) = read_cache_file(&cache_path) {
                println!("Cached response: {}", format_output(&report));
                exit(0);
            }
        }
    }

    match client.get(url).send() {
        Ok(response) => {
            let report: WeatherReportCurrent = response.json().expect("Invalid response from API");

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
    //TODO config params as struct
    // writeln!(&mut f, "{}",lat);
    // writeln!(&mut f, env!("CARGO_PKG_VERSION"))?;
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
