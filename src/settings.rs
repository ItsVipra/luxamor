use config::Config;

pub fn get_config() -> Result<Config, config::ConfigError> {
    Config::builder()
        .set_default("link_len", 16)?
        .set_default("haas_url", "http://homeassistant.local:8123")?
        .set_default("ping_timeout", 300)?
        .set_default("port", 5892)?
        .set_default("db", "db/db.sqlite")?
        .add_source(config::File::with_name("luxamor.toml"))
        .build()
}