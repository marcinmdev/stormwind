use serde::{Deserialize, Serialize};
use compact_str::CompactString;

#[derive(Serialize, Deserialize, Debug)]
pub struct CurrentUnits {
    pub time: CompactString,
    pub interval: CompactString,
    pub temperature_2m: CompactString,
    pub relative_humidity_2m: CompactString,
    pub apparent_temperature: CompactString,
    pub precipitation: CompactString,
    pub rain: CompactString,
    pub showers: CompactString,
    pub snowfall: CompactString,
    pub weather_code: CompactString,
    pub cloud_cover: CompactString,
    pub pressure_msl: CompactString,
    pub surface_pressure: CompactString,
    pub wind_speed_10m:	CompactString,
    pub wind_direction_10m: CompactString,
    pub wind_gusts_10m: CompactString,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Current {
    pub time: CompactString,
    pub interval: u32,
    pub temperature_2m: f32,
    pub relative_humidity_2m: f32,
    pub apparent_temperature: f32,
    pub is_day: u8,
    pub precipitation: f32,
    pub rain: f32,
    pub showers: f32,
    pub snowfall: f32,
    pub weather_code: u8,
    pub cloud_cover: f32,
    pub pressure_msl: f32,
    pub surface_pressure: f32,
    pub wind_speed_10m:	f32,
    pub wind_direction_10m: f32,
    pub wind_gusts_10m: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WeatherReportCurrent {
    pub current_units: CurrentUnits,
    pub current: Current,
    pub hourly_units: HourlyUnits,
    pub hourly: Hourly,
}

// Add new structs for air quality data
#[derive(Serialize, Deserialize, Debug)]
pub struct AirQualityHourlyUnits {
    pub time: CompactString,
    pub european_aqi: CompactString,
    pub us_aqi: Option<CompactString>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct AirQualityHourly {
    pub time: Vec<CompactString>,
    pub european_aqi: Vec<u8>,
    pub us_aqi: Option<Vec<u16>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AirQualityReport {
    pub hourly_units: AirQualityHourlyUnits,
    pub hourly: AirQualityHourly,
}