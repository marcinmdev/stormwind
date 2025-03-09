use crate::report::WeatherReportCurrent;
use clap::Parser;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

mod report;

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

#[derive(clap::ValueEnum, Clone, Default, Debug, Deserialize, strum::Display)]
#[strum(serialize_all = "lowercase")]
enum AqiStandard {
    #[default]
    European,
    Us,
}

#[derive(clap::ValueEnum, Clone, Default, Debug, Deserialize, strum::Display)]
#[strum(serialize_all = "lowercase")]
enum AqiDomain {
    #[default]
    Auto,
    CamsEurope,
    CamsGlobal,
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

    #[arg(long, value_enum, default_value_t, help = "Air Quality Index standard to use (European or US)")]
    aqi_standard: AqiStandard,
    
    #[arg(long, value_enum, default_value_t, help = "AQI domain to use (auto, cams_europe, or cams_global)")]
    aqi_domain: AqiDomain,
}

#[derive(Serialize, Debug)]
struct WaybarOutput {
    text: String,
    tooltip: String,
}

fn main() {
    let client = Client::new();

    let args = Args::parse();

    // Weather API URL
    let weather_url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}\
        &current=temperature_2m,relative_humidity_2m,apparent_temperature,is_day,precipitation,\
        rain,showers,snowfall,weather_code,cloud_cover,pressure_msl,surface_pressure,\
        wind_speed_10m,wind_direction_10m,wind_gusts_10m\
        &hourly=temperature_2m,precipitation_probability\
        &forecast_hours=8\
        &temperature_unit={}&wind_speed_unit={}&precipitation_unit={}",
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

    // Get weather icon based on weather code - now using colored emojis
    let mut icon = match &report.current.weather_code {
        0 => "☀️",         // Clear sky
        1 | 2 => "🌤️",    // Partly cloudy
        3 => "☁️",         // Overcast
        45 | 48 => "🌫️",   // Fog
        51 | 53 | 55 => "🌦️", // Drizzle
        56 | 57 => "🌨️",      // Freezing drizzle
        61 | 63 | 65 => "🌧️", // Rain
        66 | 67 => "🌨️",      // Freezing rain
        71 | 73 | 75 | 77 => "❄️", // Snow
        80..=82 => "🌧️",       // Rain showers
        85 | 86 => "🌨️",       // Snow showers
        95..=97 => "⛈️",       // Thunderstorm
        _ => "🌡️",             // Default/unknown
    };

    // Night icons for clear and partly cloudy conditions
    let icon_night = match &report.current.weather_code {
        0 => "🌙",         // Clear night
        1 | 2 => "☁️",     // Partly cloudy night
        _ => icon,
    };

    // Use night icon if it's night
    if report.current.is_day == 0 {
        icon = icon_night;
    }

    // Current weather information
    let mut tooltip = format!(
        "Current Conditions\n\n\
        🌡️ Feels like: {}{}\n\
        💨 Wind: {} {}\n\
        💧 Humidity: {}{}\n\
        ☁️ Cloud cover: {}{}",
        report.current.apparent_temperature, temp_unit,
        report.current.wind_speed_10m, report.current_units.wind_speed_10m,
        report.current.relative_humidity_2m, report.current_units.relative_humidity_2m,
        report.current.cloud_cover, report.current_units.cloud_cover
    );

    // Add precipitation info if present
    if report.current.precipitation > 0.0 {
        tooltip = format!(
            "{}\n🌧️ Precipitation: {} {}",
            tooltip, 
            report.current.precipitation,
            report.current_units.precipitation
        );
    }

    // Add snowfall info if present
    if report.current.snowfall > 0.0 {
        tooltip = format!(
            "{}\n❄️ Snowfall: {} {}", 
            tooltip, 
            report.current.snowfall,
            report.current_units.snowfall
        );
    }

    let waybar_output = WaybarOutput {
        text: format!("{} {}°", &icon, &temp.round().abs()),
        tooltip,
    };

    json!(waybar_output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use compact_str::CompactString;
    use report::{Current, CurrentUnits};

    #[test]
    fn test_format_output() {
        let test_report_current_units = CurrentUnits {
            relative_humidity_2m: CompactString::from("%"),
            time: CompactString::from("iso8601"),
            interval: CompactString::from("seconds"),
            rain: CompactString::from("mm"),
            apparent_temperature: CompactString::from("°C"),
            temperature_2m: CompactString::from("°C"),
            showers: CompactString::from("mm"),
            snowfall: CompactString::from("cm"),
            cloud_cover: CompactString::from("%"),
            pressure_msl: CompactString::from("hPa"),
            surface_pressure: CompactString::from("hPa"),
            precipitation: CompactString::from("mm"),
            weather_code: CompactString::from("wmo code"),
            wind_direction_10m: CompactString::from("°"),
            wind_gusts_10m: CompactString::from("km/h"),
            wind_speed_10m: CompactString::from("km/h"),
        };

        let test_report_current_night = Current {
            relative_humidity_2m: 10.11,
            time: CompactString::from("2024-12-12-12T12:12"),
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
        assert!(output_night.to_string().contains(""));
        assert!(output_night.to_string().contains(""));
    }
}
