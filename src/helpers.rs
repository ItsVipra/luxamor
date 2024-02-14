use colors_transform::Color;
use rand::Rng;
use reqwest::header;

pub fn new_link(size: usize) -> Box<str> {
    const BASE59: &[u8] = b"0123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghiklmnopqrstuvwxyz";

    let mut id = String::with_capacity(size);
    let mut rng = rand::thread_rng();
    for _ in 0..size {
        id.push(BASE59[rng.gen::<usize>() % 59] as char);
    }

    Box::from(id)
}

// async fn peel(query: QueryResult<Vec<Ping>>) -> Option<Ping> {
//     query.unwrap().first().cloned()
// }

pub async fn haas_api(color: String, origin: String) -> Result<(), colors_transform::ParseError> {
    let config = super::settings::get_config().expect("config should have passed checks before");

    let mut headers = header::HeaderMap::new();
    headers.insert("Authorization", format!("Bearer {}", config.get_string("haas_key").expect("Home Assistant API Key not provided")).parse().unwrap());
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let rgb = hex_to_rgb(color);

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let rgb = rgb.await?;

    match client.post(format!("{}/api/states/sensor.thought_ping", config.get_string("haas_url").unwrap_or("http://homeassistant.local:8123".to_string())))
        .headers(headers)
        .body(format!("{{\"state\":\"ping\", \"attributes\":{{\"red\":\"{}\", \"green\":\"{}\", \"blue\":\"{}\", \"origin\":\"{}\", \"timestamp\":\"{}\"}}}}", rgb.0, rgb.1, rgb.2, origin, chrono::Utc::now().naive_utc()))
        .send().await {
        Ok(_) => Ok(()),
        Err(_) => Err(colors_transform::ParseError { message: "haas api error".parse().unwrap() })
    }
}

async fn hex_to_rgb(hex: String) -> Result<(String, String, String), colors_transform::ParseError> {
    let rgb = colors_transform::Rgb::from_hex_str(&hex);
    match rgb {
        Ok(rgb) => {
            Ok((rgb.get_red().to_string(), rgb.get_green().to_string(), rgb.get_blue().to_string()))
        }
        Err(e) => {
            Err(e)
        }
    }
}

pub fn valid_hex(hex: &str) -> bool {
    colors_transform::Rgb::from_hex_str(hex).is_ok()
}