use std::net::Ipv4Addr;

pub struct Config {
    pub database_url: String,
    pub address: String,
}

impl Config {
    pub fn default() -> Self {
        let port = dotenvy::var("PORT").unwrap_or("8000".to_string());
        let address = Ipv4Addr::UNSPECIFIED.to_string();

        Config {
            database_url: dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            address: format!("{}:{}", address, port),
        }
    }
}
