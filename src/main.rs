use clap::Parser;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::report::WeatherReportCurrent;
use serde_json::{json, Value};

mod report;

//TODO conditional tooltip - rain/snow

#[derive(clap::ValueEnum, Clone, Default, Debug, Deserialize, strum::Display)]
#[strum(serialize_all = "lowercase")]
enum UnitsTemperature {
    #[default]
    Celsius,
    Fahrenheit,
}

#[derive(clap::ValueEnum, Clone, Default, Debug, Deserialize, strum::Display)]
#[strum(serialize_all = "lowercase")]
enum UnitsWindspeed {
    #[default]
    Kmh,
    Ms,
    Mph,
    Kn,
}

#[derive(clap::ValueEnum, Clone, Default, Debug, Deserialize, strum::Display)]
#[strum(serialize_all = "lowercase")]
enum UnitsPrecipitation {
    #[default]
    Mm,
    Inch,
}

#[derive(Parser, Deserialize, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, help = "Latitude of location")]
    lat: f32,

    #[arg(long, help = "Longitude of location")]
    lon: f32,

    #[arg(long, value_enum, default_value_t)]
    units_temperature: UnitsTemperature,

    #[arg(long, value_enum, default_value_t)]
    units_wind_speed: UnitsWindspeed,

    #[arg(long, value_enum, default_value_t)]
    units_precipitation: UnitsPrecipitation,
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
        &current=temperature_2m,relative_humidity_2m,apparent_temperature,is_day,precipitation,\
        rain,showers,snowfall,weather_code,cloud_cover,pressure_msl,surface_pressure,\
        wind_speed_10m,wind_direction_10m,wind_gusts_10m&\
        temperature_unit={}&wind_speed_unit={}&precipitation_unit={}",
        args.lat, args.lon, args.units_temperature, args.units_wind_speed, args.units_precipitation
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

    let mut icon = match &report.current.weather_code {
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
        95..=97 => "󰖓",

        _ => "",
    };

    let icon_night = match &report.current.weather_code {
        0 => "",
        1 | 2 => "",
        _ => icon,
    };

    if report.current.is_day == 0 {
        icon = icon_night
    }

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
    use report::{Current, CurrentUnits};

    use super::*;

    #[test]
    fn test_format_output() {
        let test_report_current_units = CurrentUnits {
            relative_humidity_2m: String::from("%"),
            time: String::from("iso8601"),
            interval: String::from("seconds"),
            rain: String::from("mm"),
            apparent_temperature: String::from("°C"),
            temperature_2m: String::from("°C"),
            showers: String::from("mm"),
            snowfall: String::from("cm"),
            cloud_cover: String::from("%"),
            pressure_msl: String::from("hPa"),
            surface_pressure: String::from("hPa"),
            precipitation: String::from("mm"),
            weather_code: String::from("wmo code"),
            wind_direction_10m: String::from("°"),
            wind_gusts_10m: String::from("km/h"),
            wind_speed_10m: String::from("km/h"),
        };

        let test_report_current_night = Current {
            relative_humidity_2m: 10.11,
            time: String::from("2024-12-12-12T12:12"),
            interval: 900,
            rain: 10.0,
            apparent_temperature: 20.0,
            temperature_2m: 15.3,
            showers: 5.4,
            snowfall: 4.1,
            cloud_cover: 50.1,
            pressure_msl: 1009.5,
            surface_pressure: 978.2,
            precipitation: 11.3,
            weather_code: 0,
            wind_direction_10m: 334.9,
            wind_gusts_10m: 11.5,
            wind_speed_10m: 12.4,
            is_day: 0,
        };

        let test_report_night = WeatherReportCurrent {
            current_units: test_report_current_units,
            current: test_report_current_night,
        };

        let output_night = format_output(&test_report_night);
        assert!(output_night.to_string().contains(""));
    }
}
