use config::Config;

pub fn get_config() -> Result<Config, config::ConfigError> {
    Config::builder()
        .set_default("link_len", 16)?
        .set_default("haas_url", "http://homeassistant.local:8123")?
        .set_default("ping_timeout", 300)?
        .set_default("port", 5892)?
        .set_default("db", "db/db.sqlite")?
        .set_default("PRE_SHARED_KEY", format!("RUNTIME_GENERATED-{}", gen_random_key()))?
        .add_source(config::File::with_name("luxamor.toml"))
        .build()
}

fn gen_random_key() -> String {
    use rand::seq::SliceRandom;

    let mut rng= rand::thread_rng();
    (0..64).map(|_| *b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz".choose(&mut rng).unwrap() as char).collect::<String>()
}