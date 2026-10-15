mod report;

use crate::report::{AirQualityReport, WeatherReport};
use clap::Parser;
use reqwest::blocking::Client;
use reqwest::Url;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Write;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

const REQUEST_TIMEOUT_SECS: u64 = 10;
const MAX_ATTEMPTS: u32 = 3;
const RETRY_DELAY_MS: u64 = 1500;
const FORECAST_HOURS: usize = 8;
const OFFLINE_RETRY_SECS: u64 = 5;

const WEATHER_API: &str = "https://api.open-meteo.com/v1/forecast";
const AIR_QUALITY_API: &str = "https://air-quality-api.open-meteo.com/v1/air-quality";

#[derive(clap::ValueEnum, Clone, Copy, Default, Debug, strum::Display)]
#[strum(serialize_all = "lowercase")]
enum UnitsTemperature {
    #[default]
    Celsius,
    Fahrenheit,
}

#[derive(clap::ValueEnum, Clone, Copy, Default, Debug, strum::Display)]
#[strum(serialize_all = "lowercase")]
enum UnitsWindspeed {
    #[default]
    Kmh,
    Ms,
    Mph,
    Kn,
}

#[derive(clap::ValueEnum, Clone, Copy, Default, Debug, strum::Display)]
#[strum(serialize_all = "lowercase")]
enum UnitsPrecipitation {
    #[default]
    Mm,
    Inch,
}

#[derive(clap::ValueEnum, Clone, Copy, Default, Debug, strum::Display)]
#[strum(serialize_all = "lowercase")]
enum AqiStandard {
    #[default]
    European,
    Us,
}

impl AqiStandard {
    fn values(self, aq: &AirQualityReport) -> &[Option<u16>] {
        match self {
            Self::European => &aq.hourly.european_aqi,
            Self::Us => &aq.hourly.us_aqi,
        }
    }

    fn emoji(self, aqi: u16) -> &'static str {
        match self {
            Self::European => match aqi {
                0..=20 => "🟢",   // Good
                21..=40 => "🟡",  // Fair
                41..=60 => "🟠",  // Moderate
                61..=80 => "🔴",  // Poor
                81..=100 => "🟣", // Very Poor
                _ => "⚫",        // Extremely Poor
            },
            Self::Us => match aqi {
                0..=50 => "🟢",    // Good
                51..=100 => "🟡",  // Moderate
                101..=150 => "🟠", // Unhealthy for Sensitive Groups
                151..=200 => "🔴", // Unhealthy
                201..=300 => "🟣", // Very Unhealthy
                _ => "⚫",         // Hazardous
            },
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Default, Debug, strum::Display)]
#[strum(serialize_all = "snake_case")]
enum AqiDomain {
    #[default]
    Auto,
    CamsEurope,
    CamsGlobal,
}

#[derive(Parser, Debug)]
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
        help = "Air Quality Index standard to use"
    )]
    aqi_standard: AqiStandard,

    #[arg(
        long,
        value_enum,
        default_value_t,
        help = "Air quality model domain to use"
    )]
    aqi_domain: AqiDomain,

    #[arg(
        long,
        help = "Run forever, refreshing every INTERVAL seconds. While no \
                successful fetch has happened yet (e.g. no internet at \
                boot), retries quickly instead of waiting a full interval."
    )]
    watch_interval: Option<u64>,
}

#[derive(Serialize, Debug)]
struct WaybarOutput {
    text: String,
    tooltip: String,
}

fn fetch<T: DeserializeOwned>(client: &Client, url: &Url) -> Result<T, String> {
    let mut last_err = String::new();

    for attempt in 1..=MAX_ATTEMPTS {
        match client.get(url.clone()).send() {
            Ok(response) if response.status().is_success() => {
                return response.json().map_err(|e| e.to_string());
            }
            Ok(response) => last_err = format!("HTTP {}", response.status()),
            Err(e) => last_err = e.to_string(),
        }

        if attempt < MAX_ATTEMPTS {
            thread::sleep(Duration::from_millis(RETRY_DELAY_MS));
        }
    }

    Err(last_err)
}


fn weather_url(args: &Args) -> Url {
    Url::parse_with_params(
        WEATHER_API,
        &[
            ("latitude", args.lat.to_string()),
            ("longitude", args.lon.to_string()),
            (
                "current",
                "temperature_2m,relative_humidity_2m,apparent_temperature,is_day,\
                 precipitation,snowfall,weather_code,cloud_cover,wind_speed_10m"
                    .to_string(),
            ),
            (
                "hourly",
                "temperature_2m,precipitation_probability,precipitation".to_string(),
            ),
            ("forecast_hours", FORECAST_HOURS.to_string()),
            ("temperature_unit", args.units_temperature.to_string()),
            ("wind_speed_unit", args.units_wind_speed.to_string()),
            ("precipitation_unit", args.units_precipitation.to_string()),
            ("timezone", "auto".to_string()),
        ],
    )
    .expect("WEATHER_API is a valid URL")
}

fn air_quality_url(args: &Args) -> Url {
    Url::parse_with_params(
        AIR_QUALITY_API,
        &[
            ("latitude", args.lat.to_string()),
            ("longitude", args.lon.to_string()),
            ("hourly", "european_aqi,us_aqi".to_string()),
            ("forecast_hours", FORECAST_HOURS.to_string()),
            ("domains", args.aqi_domain.to_string()),
        ],
    )
    .expect("AIR_QUALITY_API is a valid URL")
}

fn cache_path(lat: f32, lon: f32) -> PathBuf {
    let dir = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
        .unwrap_or_else(std::env::temp_dir);

    dir.join(env!("CARGO_PKG_NAME"))
        .join(format!("{lat:.4}-{lon:.4}.json"))
}

fn save_cached_output(path: &Path, output: &str) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, output);
}

fn run_once(args: &Args, client: &Client, cache_file: &Path) -> bool {
    let weather_report: WeatherReport = match fetch(client, &weather_url(args)) {
        Ok(report) => report,
        Err(e) => {
            eprintln!("Weather API error: {e}");
            if let Ok(cached) = std::fs::read_to_string(cache_file) {
                println!("{cached}");
                let _ = std::io::stdout().flush();
            }
            return false;
        }
    };

    let air_quality_report = fetch::<AirQualityReport>(client, &air_quality_url(args))
        .inspect_err(|e| eprintln!("Air quality API error: {e}"))
        .ok();

    let output = format_output(
        &weather_report,
        air_quality_report.as_ref(),
        args.aqi_standard,
    );
    let json = serde_json::to_string(&output).expect("WaybarOutput is serializable");

    save_cached_output(cache_file, &json);
    println!("{json}");
    let _ = std::io::stdout().flush();
    true
}

fn main() {
    let args = Args::parse();
    let client = Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .build()
        .expect("Failed to build HTTP client");
    let cache_file = cache_path(args.lat, args.lon);

    match args.watch_interval {
        None => {
            if !run_once(&args, &client, &cache_file) {
                std::process::exit(1);
            }
        }
        Some(interval) => loop {
            let success = run_once(&args, &client, &cache_file);
            let sleep_secs = if success { interval } else { OFFLINE_RETRY_SECS };
            thread::sleep(Duration::from_secs(sleep_secs));
        },
    }
}

fn weather_icon(code: u8, is_day: bool) -> &'static str {
    match (code, is_day) {
        (0, true) => "☀️",                        // Clear sky
        (0, false) => "🌙",                       // Clear night
        (1 | 2, true) => "🌤️",                    // Partly cloudy
        (1 | 2, false) | (3, _) => "☁️",          // Partly cloudy night / overcast
        (45 | 48, _) => "🌫️",                     // Fog
        (51 | 53 | 55, _) => "🌦️",                // Drizzle
        (61 | 63 | 65 | 80..=82, _) => "🌧️",      // Rain / rain showers
        (56 | 57 | 66 | 67 | 85 | 86, _) => "🌨️", // Freezing drizzle/rain, snow showers
        (71 | 73 | 75 | 77, _) => "❄️",           // Snow
        (95 | 96 | 99, _) => "⛈️",                // Thunderstorm (with hail)
        _ => "❓",                                // Unknown
    }
}

fn aqi_label(air_quality: Option<&AirQualityReport>, standard: AqiStandard, hour: usize) -> String {
    air_quality
        .and_then(|aq| standard.values(aq).get(hour).copied().flatten())
        .map_or_else(
            || String::from("N/A ❓"),
            |aqi| format!("{aqi} {}", standard.emoji(aqi)),
        )
}

fn format_output(
    report: &WeatherReport,
    air_quality: Option<&AirQualityReport>,
    aqi_standard: AqiStandard,
) -> WaybarOutput {
    let current = &report.current;
    let units = &report.current_units;

    let mut tooltip = format!(
        "🌡️ Feels like: {}{}\n\
         💨 Wind: {} {}\n\
         💧 Humidity: {}{}\n\
         ☁️ Cloud cover: {}{}",
        current.apparent_temperature,
        units.temperature_2m,
        current.wind_speed_10m,
        units.wind_speed_10m,
        current.relative_humidity_2m,
        units.relative_humidity_2m,
        current.cloud_cover,
        units.cloud_cover,
    );

    if current.precipitation > 0.0 {
        let _ = write!(
            tooltip,
            "\n🌧️ Precipitation: {} {}",
            current.precipitation, units.precipitation
        );
    }

    if current.snowfall > 0.0 {
        let _ = write!(
            tooltip,
            "\n❄️ Snowfall: {} {}",
            current.snowfall, units.snowfall
        );
    }

    if air_quality.is_some() {
        let _ = write!(
            tooltip,
            "\n😷 Air Quality: {}",
            aqi_label(air_quality, aqi_standard, 0)
        );
    }

    tooltip.push('\n');

    let hourly = &report.hourly;
    let hours = hourly
        .time
        .iter()
        .zip(&hourly.temperature_2m)
        .zip(&hourly.precipitation_probability)
        .zip(&hourly.precipitation)
        .take(FORECAST_HOURS)
        .enumerate();

    for (i, (((time, temp), precip_prob), precip)) in hours {
        let hour = time.split_once('T').map_or(time.as_str(), |(_, t)| t);

        let _ = write!(
            tooltip,
            "\n{hour:<5} | {:>2}° | 🌧️{:>3}% {precip:.2}{} | AQI: {:>4}",
            temp.round() as i32,
            precip_prob.round() as i32,
            units.precipitation,
            aqi_label(air_quality, aqi_standard, i),
        );
    }

    WaybarOutput {
        text: format!(
            "{} {}°",
            weather_icon(current.weather_code, current.is_day != 0),
            current.temperature_2m.round() as i32
        ),
        tooltip,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn weather(code: u8, is_day: u8) -> WeatherReport {
        serde_json::from_value(json!({
            "current_units": {
                "temperature_2m": "°C",
                "relative_humidity_2m": "%",
                "precipitation": "mm",
                "snowfall": "cm",
                "cloud_cover": "%",
                "wind_speed_10m": "km/h"
            },
            "current": {
                "temperature_2m": 12.6,
                "relative_humidity_2m": 80.0,
                "apparent_temperature": 11.0,
                "is_day": is_day,
                "precipitation": 0.0,
                "snowfall": 0.0,
                "weather_code": code,
                "cloud_cover": 50.0,
                "wind_speed_10m": 10.0
            },
            "hourly": {
                "time": ["2026-10-14T08:00", "2026-10-14T09:00"],
                "temperature_2m": [12.6, 13.4],
                "precipitation_probability": [5.0, 40.0],
                "precipitation": [0.0, 0.3]
            }
        }))
        .unwrap()
    }

    fn air_quality() -> AirQualityReport {
        serde_json::from_value(json!({
            "hourly": { "european_aqi": [15, null], "us_aqi": [120] }
        }))
        .unwrap()
    }

    #[test]
    fn thunderstorm_codes_have_icon() {
        for code in [95, 96, 99] {
            assert_eq!(weather_icon(code, true), "⛈️");
        }
    }

    #[test]
    fn night_icons() {
        assert_eq!(weather_icon(0, false), "🌙");
        assert_eq!(weather_icon(2, false), "☁️");
        assert_eq!(weather_icon(61, false), "🌧️");
    }

    #[test]
    fn text_shows_icon_and_rounded_temperature() {
        let output = format_output(&weather(0, 1), None, AqiStandard::European);
        assert_eq!(output.text, "☀️ 13°");
    }

    #[test]
    fn tooltip_lists_hourly_rows() {
        let output = format_output(&weather(0, 1), Some(&air_quality()), AqiStandard::European);
        assert!(output.tooltip.contains("😷 Air Quality: 15 🟢"));
        assert!(output
            .tooltip
            .contains("\n08:00 | 13° | 🌧️  5% 0.00mm | AQI: 15 🟢"));
        assert!(output
            .tooltip
            .contains("\n09:00 | 13° | 🌧️ 40% 0.30mm | AQI: N/A ❓"));
    }

    #[test]
    fn mismatched_hourly_lengths_do_not_panic() {
        let mut report = weather(0, 1);
        report.hourly.precipitation.pop();
        let output = format_output(&report, None, AqiStandard::European);
        assert_eq!(output.tooltip.matches(" | AQI: ").count(), 1);
    }

    #[test]
    fn aqi_label_uses_selected_standard() {
        let aq = air_quality();
        assert_eq!(aqi_label(Some(&aq), AqiStandard::European, 0), "15 🟢");
        assert_eq!(aqi_label(Some(&aq), AqiStandard::European, 1), "N/A ❓");
        assert_eq!(aqi_label(Some(&aq), AqiStandard::Us, 0), "120 🟠");
        assert_eq!(aqi_label(Some(&aq), AqiStandard::Us, 5), "N/A ❓");
        assert_eq!(aqi_label(None, AqiStandard::Us, 0), "N/A ❓");
    }

    #[test]
    fn cache_path_uses_app_subdirectory() {
        let path = cache_path(50.11, 19.92);
        assert!(path.ends_with("stormwind/50.1100-19.9200.json"));
    }

    #[test]
    fn aqi_domain_matches_api_names() {
        assert_eq!(AqiDomain::CamsEurope.to_string(), "cams_europe");
        assert_eq!(AqiDomain::CamsGlobal.to_string(), "cams_global");
        assert_eq!(AqiDomain::Auto.to_string(), "auto");
    }
}
