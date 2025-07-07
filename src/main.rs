use crate::report::{WeatherReport, AirQualityReport};
use clap::Parser;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use compact_str::CompactString;

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

    let aqi_param = match args.aqi_standard {
        AqiStandard::European => "european_aqi",
        AqiStandard::Us => "us_aqi",
    };

    let domain_param = match args.aqi_domain {
        AqiDomain::Auto => "auto",
        AqiDomain::CamsEurope => "cams_europe",
        AqiDomain::CamsGlobal => "cams_global",
    };

    let air_quality_url = format!(
        "https://air-quality-api.open-meteo.com/v1/air-quality?latitude={}&longitude={}\
        &hourly={}\
        &forecast_hours=8\
        &domains={}",
        args.lat, args.lon,
        aqi_param,
        domain_param
    ); 

    let weather_report = match client.get(&weather_url).send() {
        Ok(response) => {
            response.json::<WeatherReport>().expect("Invalid response from weather API")
        }
        Err(_) => {
            eprintln!("Connection error to weather API");
            std::process::exit(1);
        }
    };

    let air_quality_report = match client.get(&air_quality_url).send() {
        Ok(response) => {
            response.json::<AirQualityReport>().expect("Invalid response from air quality API")
        }
        Err(e) => {
            eprintln!("Connection error to air quality API: {}", e);
            // Create a default empty report so the app can continue with just weather data
            AirQualityReport {
                hourly_units: report::AirQualityHourlyUnits {
                    time: CompactString::from("iso8601"),
                    european_aqi: CompactString::from(""),
                    us_aqi: None,
                },
                hourly: report::AirQualityHourly {
                    time: Vec::new(),
                    european_aqi: Vec::new(),
                    us_aqi: None,
                },
            }
        }
    };

    println!("{}", format_output(&weather_report, &air_quality_report, &args.aqi_standard));
}

fn format_output(report: &WeatherReport, air_quality: &AirQualityReport, aqi_standard: &AqiStandard) -> Value {
    let temp = report.current.temperature_2m;
    let temp_unit = &report.current_units.temperature_2m;

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
        "🌡️ Feels like: {}{}\n\
        💨 Wind: {} {}\n\
        💧 Humidity: {}{}\n\
        ☁️ Cloud cover: {}{}",
        report.current.apparent_temperature, temp_unit,
        report.current.wind_speed_10m, report.current_units.wind_speed_10m,
        report.current.relative_humidity_2m, report.current_units.relative_humidity_2m,
        report.current.cloud_cover, report.current_units.cloud_cover
    );

    if report.current.precipitation > 0.0 {
        tooltip = format!(
            "{}\n🌧️ Precipitation: {} {}",
            tooltip, 
            report.current.precipitation,
            report.current_units.precipitation
        );
    }

    if report.current.snowfall > 0.0 {
        tooltip = format!(
            "{}\n❄️ Snowfall: {} {}", 
            tooltip, 
            report.current.snowfall,
            report.current_units.snowfall
        );
    }

    // Get current AQI (first hour value)
    if !air_quality.hourly.time.is_empty() {
        let current_aqi_info = match aqi_standard {
            AqiStandard::European => {
                if !air_quality.hourly.european_aqi.is_empty() {
                    let aqi = air_quality.hourly.european_aqi[0];
                    let emoji = get_european_aqi_emoji(aqi);
                    format!("😷 Air Quality: {} {}", aqi, emoji)
                } else {
                    String::from("😷 Air Quality: N/A ❓")
                }
            },
            AqiStandard::Us => {
                if let Some(us_aqi) = &air_quality.hourly.us_aqi {
                    if !us_aqi.is_empty() {
                        let aqi = us_aqi[0];
                        let emoji = get_us_aqi_emoji(aqi);
                        format!("😷 Air Quality: {} {}", aqi, emoji)
                    } else {
                        String::from("😷 Air Quality: N/A ❓")
                    }
                } else {
                    String::from("😷 Air Quality: N/A ❓")
                }
            }
        };

        tooltip = format!("{}\n{}", tooltip, current_aqi_info);
    }

    // Add hourly forecast information
    tooltip = format!("{}\n\n--------- Hourly Forecast ---------", tooltip);

    // Get emoji for European AQI
    fn get_european_aqi_emoji(aqi: u8) -> &'static str {
        match aqi {
            0..=20 => "🟢",    // Good
            21..=40 => "🟡",   // Fair
            41..=60 => "🟠",   // Moderate
            61..=80 => "🔴",   // Poor
            81..=100 => "🟣",  // Very Poor
            _ => "⚫",         // Extremely Poor
        }
    }

    // Get emoji for US AQI
    fn get_us_aqi_emoji(aqi: u16) -> &'static str {
        match aqi {
            0..=50 => "🟢",     // Good
            51..=100 => "🟡",   // Moderate
            101..=150 => "🟠",  // Unhealthy for Sensitive Groups
            151..=200 => "🔴",  // Unhealthy
            201..=300 => "🟣",  // Very Unhealthy
            _ => "⚫",          // Hazardous
        }
    }

    // Loop through the hourly data (up to 8 hours)
    let max_hours = report.hourly.time.len().min(8);
    for i in 0..max_hours {
        let time = &report.hourly.time[i];
        let hour_temp = report.hourly.temperature_2m[i];
        let precip_prob = report.hourly.precipitation_probability[i];

        // Format time to show just HH:MM
        let time_parts: Vec<&str> = time.split('T').collect();
        let hour_str = if time_parts.len() > 1 {
            time_parts[1].to_string()
        } else {
            time.to_string()
        };

        // Get air quality data if available based on selected standard
        let aqi_info = match aqi_standard {
            AqiStandard::European => {
                if !air_quality.hourly.european_aqi.is_empty() && i < air_quality.hourly.european_aqi.len() {
                    let aqi = air_quality.hourly.european_aqi[i];
                    let emoji = get_european_aqi_emoji(aqi);
                    format!("{:>3} {}", aqi, emoji)
                } else {
                    String::from("N/A ❓")
                }
            },
            AqiStandard::Us => {
                if let Some(us_aqi) = &air_quality.hourly.us_aqi {
                    if !us_aqi.is_empty() && i < us_aqi.len() {
                        let aqi = us_aqi[i];
                        let emoji = get_us_aqi_emoji(aqi);
                        format!("{:>3} {}", aqi, emoji)
                    } else {
                        String::from("N/A ❓")
                    }
                } else {
                    String::from("N/A ❓")
                }
            }
        };

        tooltip = format!(
            "{}\n{:<5} | {:>3}{}° | 🌧️{:>3}% | AQI: {:>5}",
            tooltip,
            hour_str,
            hour_temp.round() as i32,
            if temp_unit.starts_with('°') { "" } else { temp_unit },
            precip_prob.round() as i32,
            aqi_info
        );
    }

    // Changing this in format_output
    let waybar_output = WaybarOutput {
        text: format!("{} {}°", &icon, &temp.round().abs()),
        tooltip,
    };

    json!(waybar_output)
}