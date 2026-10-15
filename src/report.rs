use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct WeatherReport {
    pub current_units: CurrentUnits,
    pub current: Current,
    pub hourly: Hourly,
}

#[derive(Deserialize, Debug)]
pub struct CurrentUnits {
    pub temperature_2m: String,
    pub relative_humidity_2m: String,
    pub precipitation: String,
    pub snowfall: String,
    pub cloud_cover: String,
    pub wind_speed_10m: String,
}

#[derive(Deserialize, Debug)]
pub struct Current {
    pub temperature_2m: f32,
    pub relative_humidity_2m: f32,
    pub apparent_temperature: f32,
    pub is_day: u8,
    pub precipitation: f32,
    pub snowfall: f32,
    pub weather_code: u8,
    pub cloud_cover: f32,
    pub wind_speed_10m: f32,
}

#[derive(Deserialize, Debug)]
pub struct Hourly {
    pub time: Vec<String>,
    pub temperature_2m: Vec<f32>,
    pub precipitation_probability: Vec<f32>,
    pub precipitation: Vec<f32>,
}

#[derive(Deserialize, Debug)]
pub struct AirQualityReport {
    pub hourly: AirQualityHourly,
}

#[derive(Deserialize, Debug)]
pub struct AirQualityHourly {
    #[serde(default)]
    pub european_aqi: Vec<Option<u16>>,
    #[serde(default)]
    pub us_aqi: Vec<Option<u16>>,
}
