use clap::Parser;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::report::WeatherReportCurrent;
use serde_json::{json, Value};

mod report;

//TODO integration test
//TODO readme
//TODO conditional tooltip - rain/snow

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

    #[arg(long, value_enum, default_value_t=Units::Metric)]
    units: Units,
}

#[derive(Serialize, Debug)]
struct WaybarOutput {
    text: String,
    tooltip: String,
}

fn main() {
    let client = Client::new();

    let args = Args::parse();

    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}\
        &current=temperature_2m,relative_humidity_2m,apparent_temperature,precipitation,\
        rain,showers,snowfall,weather_code,cloud_cover,pressure_msl,surface_pressure,\
        wind_speed_10m,wind_direction_10m,wind_gusts_10m",
        args.lat, args.lon,
    );

    match client.get(url).send() {
        Ok(response) => {
            let report: WeatherReportCurrent = response.json().expect("Invalid response from API");

            println!("{}", format_output(&report));
        }
        Err(_) => eprintln!("Connection error"),
    };
}

fn format_output(report: &WeatherReportCurrent) -> Value {
    let temp = report.current.temperature_2m;

    let icon = match &report.current.weather_code {
        0 => "",
        1 | 2 => "",
        3 => "󰖐",
        45 | 48 => "",
        51 | 53 | 55 => "",
        56 | 57 => "󰙿",
        61 | 63 | 65 => "",
        66 | 67 => "󰙿",
        71 | 73 | 75 | 77 => "",
        80..=82 => "",
        85 | 86 => "",
        95..=97 => "",

        _ => "",
    };

    let tooltip = format!(
        "󰖝 {} {}\r {}{}\r󰖐 {}{}",
        report.current.wind_speed_10m,
        report.current_units.wind_speed_10m,
        report.current.relative_humidity_2m,
        report.current_units.relative_humidity_2m,
        report.current.cloud_cover,
        report.current_units.cloud_cover
    );

    let waybar_output = WaybarOutput {
        text: format!("{} {}°", &icon, &temp.round().abs()),
        tooltip,
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
