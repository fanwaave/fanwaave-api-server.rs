#![forbid(unsafe_code)]

use crate::config::ApiConfig;
use crate::routes;

pub fn run(config: &ApiConfig) -> Result<(), serde_json::Error> {
    println!("api bind {}", config.bind);
    let health = serde_json::to_string(&routes::health::body())?;
    println!("{health}");
    Ok(())
}
