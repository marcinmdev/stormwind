use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CurrentUnits {
    pub time: String,
    pub interval: String,
    pub temperature_2m: String,
    pub relative_humidity_2m: String,
    pub apparent_temperature: String,
    pub precipitation: String,
    pub rain: String,
    pub showers: String,
    pub snowfall: String,
    pub weather_code: String,
    pub cloud_cover: String,
    pub pressure_msl: String,
    pub surface_pressure: String,
    pub wind_speed_10m:	String,
    pub wind_direction_10m: String,
    pub wind_gusts_10m: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Current {
    pub time: String,
    pub interval: u32,
    pub temperature_2m: f32,
    pub relative_humidity_2m: f32,
    pub apparent_temperature: f32,
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
}
