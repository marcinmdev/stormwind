use crate::report::{AirQualityReport, WeatherReport};
use clap::Parser;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

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

    #[arg(
        long,
        value_enum,
        default_value_t,
        help = "Air Quality Index standard to use (European or US)"
    )]
    aqi_standard: AqiStandard,

    #[arg(
        long,
        value_enum,
        default_value_t,
        help = "AQI domain to use (auto, cams_europe, or cams_global)"
    )]
    aqi_domain: AqiDomain,
}

#[derive(Serialize, Debug)]
struct WaybarOutput {
    text: String,
    tooltip: String,
}

fn fetch_weather(client: &Client, url: &str) -> Result<WeatherReport, String> {
    let backoffs = [0, 2, 4];

    for i in 0..backoffs.len() {
        if backoffs[i] > 0 {
            std::thread::sleep(Duration::from_secs(backoffs[i] as u64));
        }

        match client.get(url).send() {
            Ok(response) => {
                if !response.status().is_success() {
                    continue;
                }
                if let Ok(report) = response.json::<WeatherReport>() {
                    return Ok(report);
                }
            }
            Err(_) => {}
        }
    }

    Err("All attempts failed".to_string())
}

fn fetch_air_quality(client: &Client, url: &str) -> Option<AirQualityReport> {
    let backoffs = [0, 2, 4];

    for i in 0..backoffs.len() {
        if backoffs[i] > 0 {
            std::thread::sleep(Duration::from_secs(backoffs[i] as u64));
        }

        if let Ok(response) = client.get(url).send() {
            if response.status().is_success() {
                if let Ok(report) = response.json::<AirQualityReport>() {
                    return Some(report);
                }
            }
        }
    }

    None
}

fn format_fallback() -> Value {
    json!({
        "text": "⚠ --",
        "tooltip": "Unable to fetch weather data"
    })
}

fn main() {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to create HTTP client");

    let args = Args::parse();

    let weather_url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}\
        &current=temperature_2m,relative_humidity_2m,apparent_temperature,is_day,precipitation,\
        rain,showers,snowfall,weather_code,cloud_cover,pressure_msl,surface_pressure,\
        wind_speed_10m,wind_direction_10m,wind_gusts_10m\
        &hourly=temperature_2m,precipitation_probability,precipitation\
        &forecast_hours=8\
        &temperature_unit={}&wind_speed_unit={}&precipitation_unit={}&timezone=auto",
        args.lat, args.lon, args.units_temperature, args.units_wind_speed, args.units_precipitation
    );

    let domain_param = match args.aqi_domain {
        AqiDomain::Auto => "auto",
        AqiDomain::CamsEurope => "cams_europe",
        AqiDomain::CamsGlobal => "cams_global",
    };

    let air_quality_url = format!(
        "https://air-quality-api.open-meteo.com/v1/air-quality?latitude={}&longitude={}\
        &hourly=european_aqi,us_aqi\
        &forecast_hours=8\
        &domains={}",
        args.lat, args.lon, domain_param
    );

    let weather_report = match fetch_weather(&client, &weather_url) {
        Ok(report) => report,
        Err(_) => {
            println!("{}", format_fallback());
            return;
        }
    };

    let air_quality_report = fetch_air_quality(&client, &air_quality_url);

    println!(
        "{}",
        format_output(
            &weather_report,
            air_quality_report.as_ref(),
            &args.aqi_standard
        )
    );
}

fn format_output(
    report: &WeatherReport,
    air_quality: Option<&AirQualityReport>,
    aqi_standard: &AqiStandard,
) -> Value {
    let temp = report.current.temperature_2m;
    let temp_unit = &report.current_units.temperature_2m;

    let mut icon = match &report.current.weather_code {
        0 => "☀️",                 // Clear sky
        1 | 2 => "🌤️",             // Partly cloudy
        3 => "☁️",                 // Overcast
        45 | 48 => "🌫️",           // Fog
        51 | 53 | 55 => "🌦️",      // Drizzle
        56 | 57 => "🌨️",           // Freezing drizzle
        61 | 63 | 65 => "🌧️",      // Rain
        66 | 67 => "🌨️",           // Freezing rain
        71 | 73 | 75 | 77 => "❄️", // Snow
        80..=82 => "🌧️",           // Rain showers
        85 | 86 => "🌨️",           // Snow showers
        95..=97 => "⛈️",           // Thunderstorm
        _ => "❓",                 // Default/unknown
    };

    let icon_night = match &report.current.weather_code {
        0 => "🌙",     // Clear night
        1 | 2 => "☁️", // Partly cloudy night
        _ => icon,
    };

    if report.current.is_day == 0 {
        icon = icon_night;
    }

    // Current weather information
    let mut tooltip = format!(
        "🌡️ Feels like: {}{}\n\
        💨 Wind: {} {}\n\
        💧 Humidity: {}{}\n\
        ☁️ Cloud cover: {}{}",
        report.current.apparent_temperature,
        temp_unit,
        report.current.wind_speed_10m,
        report.current_units.wind_speed_10m,
        report.current.relative_humidity_2m,
        report.current_units.relative_humidity_2m,
        report.current.cloud_cover,
        report.current_units.cloud_cover
    );

    if report.current.precipitation > 0.0 {
        tooltip = format!(
            "{}\n🌧️ Precipitation: {} {}",
            tooltip, report.current.precipitation, report.current_units.precipitation
        );
    }

    if report.current.snowfall > 0.0 {
        tooltip = format!(
            "{}\n❄️ Snowfall: {} {}",
            tooltip, report.current.snowfall, report.current_units.snowfall
        );
    }

    // Get current AQI (first hour value)
    if let Some(aq) = air_quality {
        if !aq.hourly.time.is_empty() {
            let current_aqi_info = match aqi_standard {
                AqiStandard::European => {
                    if !aq.hourly.european_aqi.is_empty() {
                        let aqi = aq.hourly.european_aqi[0];
                        let emoji = get_european_aqi_emoji(aqi);
                        format!("😷 Air Quality: {} {}", aqi, emoji)
                    } else {
                        String::from("😷 Air Quality: N/A ❓")
                    }
                }
                AqiStandard::Us => {
                    if !aq.hourly.us_aqi.is_empty() {
                        let aqi = aq.hourly.us_aqi[0];
                        let emoji = get_us_aqi_emoji(aqi);
                        format!("😷 Air Quality: {} {}", aqi, emoji)
                    } else {
                        String::from("😷 Air Quality: N/A ❓")
                    }
                }
            };

            tooltip = format!("{}\n{}", tooltip, current_aqi_info);
        }
    }

    fn get_european_aqi_emoji(aqi: u8) -> &'static str {
        match aqi {
            0..=20 => "🟢",   // Good
            21..=40 => "🟡",  // Fair
            41..=60 => "🟠",  // Moderate
            61..=80 => "🔴",  // Poor
            81..=100 => "🟣", // Very Poor
            _ => "⚫",        // Extremely Poor
        }
    }

    fn get_us_aqi_emoji(aqi: u16) -> &'static str {
        match aqi {
            0..=50 => "🟢",    // Good
            51..=100 => "🟡",  // Moderate
            101..=150 => "🟠", // Unhealthy for Sensitive Groups
            151..=200 => "🔴", // Unhealthy
            201..=300 => "🟣", // Very Unhealthy
            _ => "⚫",         // Hazardous
        }
    }

    tooltip = format!("{}\n", tooltip);

    let max_hours = report.hourly.time.len().min(8);
    for i in 0..max_hours {
        let time = &report.hourly.time[i];
        let hour_temp = report.hourly.temperature_2m[i];
        let precip_prob = report.hourly.precipitation_probability[i];
        let precip = report.hourly.precipitation[i];

        // Format time to show just HH:MM
        let time_parts: Vec<&str> = time.split('T').collect();
        let hour_str = if time_parts.len() > 1 {
            time_parts[1].to_string()
        } else {
            time.to_string()
        };

        // Get air quality data if available based on selected standard
        let aqi_info = if let Some(aq) = air_quality {
            match aqi_standard {
                AqiStandard::European => {
                    if !aq.hourly.european_aqi.is_empty() && i < aq.hourly.european_aqi.len() {
                        let aqi = aq.hourly.european_aqi[i];
                        let emoji = get_european_aqi_emoji(aqi);
                        format!("{} {}", aqi, emoji)
                    } else {
                        String::from("N/A ❓")
                    }
                }
                AqiStandard::Us => {
                    if !aq.hourly.us_aqi.is_empty() && i < aq.hourly.us_aqi.len() {
                        let aqi = aq.hourly.us_aqi[i];
                        let emoji = get_us_aqi_emoji(aqi);
                        format!("{} {}", aqi, emoji)
                    } else {
                        String::from("N/A ❓")
                    }
                }
            }
        } else {
            String::from("N/A ❓")
        };

        tooltip = format!(
            "{}\n{:<5} | {:>2}{}° | 🌧️{:>3}% {:.2}{} | AQI: {:>4}",
            tooltip,
            hour_str,
            hour_temp.round() as i32,
            if temp_unit.starts_with('°') {
                ""
            } else {
                temp_unit
            },
            precip_prob.round() as i32,
            precip,
            report.current_units.precipitation,
            aqi_info
        );
    }

    let waybar_output = WaybarOutput {
        text: format!("{} {}°", &icon, temp.round() as i32),
        tooltip,
    };

    json!(waybar_output)
}

