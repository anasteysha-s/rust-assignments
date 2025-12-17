use anyhow::{Context, Result};
use clap::Parser;
use config::{Config as ConfigBuilder, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;

/// Command-line arguments
#[derive(Parser, Debug)]
#[command(name = "task_3_9")]
#[command(about = "Prints its configuration to STDOUT")]
struct Cli {
    /// Enables debug mode
    #[arg(short, long)]
    debug: bool,

    /// Path to configuration file
    #[arg(short, long, env = "CONF_FILE", default_value = "config.toml")]
    conf: PathBuf,
}

/// Main configuration structure
#[derive(Debug, Deserialize)]
struct Config {
    mode: ModeConfig,
    server: ServerConfig,
    db: DatabaseConfig,
    log: LogConfig,
    background: BackgroundConfig,
}

#[derive(Debug, Deserialize)]
struct ModeConfig {
    debug: bool,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    external_url: String,
    http_port: u16,
    grpc_port: u16,
    healthz_port: u16,
    metrics_port: u16,
}

#[derive(Debug, Deserialize)]
struct DatabaseConfig {
    mysql: MysqlConfig,
}

#[derive(Debug, Deserialize)]
struct MysqlConfig {
    host: String,
    port: u16,
    database: String,
    user: String,
    #[serde(default)]
    pass: String,
    connections: ConnectionsConfig,
}

#[derive(Debug, Deserialize)]
struct ConnectionsConfig {
    max_idle: u32,
    max_open: u32,
}

#[derive(Debug, Deserialize)]
struct LogConfig {
    app: AppLogConfig,
}

#[derive(Debug, Deserialize)]
struct AppLogConfig {
    level: String,
}

#[derive(Debug, Deserialize)]
struct BackgroundConfig {
    watchdog: WatchdogConfig,
}

#[derive(Debug, Deserialize)]
struct WatchdogConfig {
    #[serde(with = "humantime_serde")]
    period: Duration,
    limit: u32,
    #[serde(with = "humantime_serde")]
    lock_timeout: Duration,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration using config crate
    let config = load_config(&cli)?;

    // Print configuration
    println!("Configuration:");
    println!("  Mode:");
    println!("    debug: {}", config.mode.debug);
    println!();
    println!("  Server:");
    println!("    external_url: {}", config.server.external_url);
    println!("    http_port: {}", config.server.http_port);
    println!("    grpc_port: {}", config.server.grpc_port);
    println!("    healthz_port: {}", config.server.healthz_port);
    println!("    metrics_port: {}", config.server.metrics_port);
    println!();
    println!("  Database:");
    println!("    MySQL:");
    println!("      host: {}", config.db.mysql.host);
    println!("      port: {}", config.db.mysql.port);
    println!("      database: {}", config.db.mysql.database);
    println!("      user: {}", config.db.mysql.user);
    println!("      pass: {}", mask_password(&config.db.mysql.pass));
    println!("      connections:");
    println!("        max_idle: {}", config.db.mysql.connections.max_idle);
    println!("        max_open: {}", config.db.mysql.connections.max_open);
    println!();
    println!("  Log:");
    println!("    app:");
    println!("      level: {}", config.log.app.level);
    println!();
    println!("  Background:");
    println!("    watchdog:");
    println!("      period: {}", humantime::format_duration(config.background.watchdog.period));
    println!("      limit: {}", config.background.watchdog.limit);
    println!(
        "      lock_timeout: {}",
        humantime::format_duration(config.background.watchdog.lock_timeout)
    );

    Ok(())
}

/// Load configuration with proper precedence using config crate
fn load_config(cli: &Cli) -> Result<Config> {
    let config = ConfigBuilder::builder()
        // 1. Start with defaults
        .set_default("mode.debug", false)?
        .set_default("server.external_url", "http://127.0.0.1:8081")?
        .set_default("server.http_port", 8080)?
        .set_default("server.grpc_port", 8082)?
        .set_default("server.healthz_port", 10025)?
        .set_default("server.metrics_port", 9199)?
        .set_default("db.mysql.host", "localhost")?
        .set_default("db.mysql.port", 3306)?
        .set_default("db.mysql.database", "myapp")?
        .set_default("db.mysql.user", "root")?
        .set_default("db.mysql.pass", "")?
        .set_default("db.mysql.connections.max_idle", 10)?
        .set_default("db.mysql.connections.max_open", 100)?
        .set_default("log.app.level", "info")?
        .set_default("background.watchdog.period", "5s")?
        .set_default("background.watchdog.limit", 10)?
        .set_default("background.watchdog.lock_timeout", "1s")?
        // 2. Merge from TOML file if it exists
        .add_source(File::from(cli.conf.clone()).required(false))
        // 3. Merge from environment variables with CONF_ prefix
        .add_source(
            Environment::with_prefix("CONF")
                .separator("_")
                .try_parsing(true)
        )
        // 4. Override debug mode from CLI flag
        .set_override("mode.debug", cli.debug)?
        .build()
        .context("Failed to build configuration")?;

    // Deserialize into Config struct
    config
        .try_deserialize()
        .context("Failed to deserialize configuration")
}

/// Mask password for display
fn mask_password(password: &str) -> String {
    if password.is_empty() {
        String::from("<empty>")
    } else {
        String::from("******")
    }
}