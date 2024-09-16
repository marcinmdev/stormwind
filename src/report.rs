use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Coord {
   pub lat: f32,
   pub lon: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Weather {
   pub id: u32,
   pub main: String,
   pub description: String,
   pub icon: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MainWeather {
   pub temp: f32,
   pub feels_like: f32,
   pub temp_min: f32,
   pub temp_max: f32,
   pub pressure: f32,
   pub humidity: f32,
   pub sea_level: f32,
   pub grnd_level: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Wind {
   pub speed: f32,
   pub deg: f32,
   pub gust: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Rain {
    #[serde(rename = "1h")]
    pub one_h: Option<f32>,
    #[serde(rename = "3h")]
    pub tree_h: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Snow {
    #[serde(rename = "1h")]
    pub one_h: Option<f32>,
    #[serde(rename = "3h")]
    pub tree_h: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Clouds {
    pub all: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sys {
    #[serde(rename = "type")]
    pub _type: Option<u32>,
    pub id: Option<u32>,
    pub country: Option<String>,
    pub sunrise: u32,
    pub sunset: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WeatherReportCurrent {
    pub coord: Coord,
    pub weather: Vec<Weather>,
    pub base: String,
    pub main: MainWeather,
    pub visibility: u32,
    pub wind: Wind,
    pub rain: Option<Rain>,
    pub snow: Option<Snow>,
    pub clouds: Clouds,
    pub dt: u64,
    pub sys: Sys,
    pub id: u64,
    pub name: String,
    pub cod: u16,
}
