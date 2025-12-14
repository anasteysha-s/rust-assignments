use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

/// Prints its configuration to STDOUT
#[derive(Parser, Debug)]
#[command(name = "task_3_9")]
#[command(version = "0.1.0")]
#[command(about = "Prints its configuration to STDOUT", long_about = None)]
struct Cli {
    /// Enables debug mode
    #[arg(short, long)]
    debug: bool,

    /// Path to configuration file
    #[arg(short, long, env = "CONF_FILE", default_value = "config.toml")]
    conf: PathBuf,
}

// ============================================================================
// CONFIGURATION STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    #[serde(default)]
    mode: ModeConfig,
    #[serde(default)]
    server: ServerConfig,
    #[serde(default)]
    db: DbConfig,
    #[serde(default)]
    log: LogConfig,
    #[serde(default)]
    background: BackgroundConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: ModeConfig::default(),
            server: ServerConfig::default(),
            db: DbConfig::default(),
            log: LogConfig::default(),
            background: BackgroundConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModeConfig {
    debug: bool,
}

impl Default for ModeConfig {
    fn default() -> Self {
        Self { debug: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServerConfig {
    external_url: String,
    http_port: u16,
    grpc_port: u16,
    healthz_port: u16,
    metrics_port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            external_url: "http://127.0.0.1".to_string(),
            http_port: 8081,
            grpc_port: 8082,
            healthz_port: 10025,
            metrics_port: 9199,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DbConfig {
    mysql: MysqlConfig,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            mysql: MysqlConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MysqlConfig {
    host: String,
    port: u16,
    #[serde(rename = "dating")]
    database: String,
    user: String,
    pass: String,
    connections: ConnectionsConfig,
}

impl Default for MysqlConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3306,
            database: "default".to_string(),
            user: "root".to_string(),
            pass: String::new(),
            connections: ConnectionsConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConnectionsConfig {
    max_idle: u32,
    max_open: u32,
}

impl Default for ConnectionsConfig {
    fn default() -> Self {
        Self {
            max_idle: 30,
            max_open: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LogConfig {
    app: AppLogConfig,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            app: AppLogConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppLogConfig {
    level: String,
}

impl Default for AppLogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackgroundConfig {
    watchdog: WatchdogConfig,
}

impl Default for BackgroundConfig {
    fn default() -> Self {
        Self {
            watchdog: WatchdogConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WatchdogConfig {
    #[serde(with = "humantime_serde")]
    period: Duration,
    limit: u32,
    #[serde(with = "humantime_serde")]
    lock_timeout: Duration,
}

impl Default for WatchdogConfig {
    fn default() -> Self {
        Self {
            period: Duration::from_secs(5),
            limit: 10,
            lock_timeout: Duration::from_secs(4),
        }
    }
}

// ============================================================================
// CONFIGURATION LOADING & MERGING
// ============================================================================

impl Config {
    /// Load configuration with proper precedence:
    /// 1. Default values (lowest priority)
    /// 2. TOML file values
    /// 3. Environment variables with CONF_ prefix (highest priority)
    fn load(config_path: &PathBuf, debug_flag: bool) -> Result<Self> {
        // Step 1: Start with defaults
        let mut config = Config::default();

        // Step 2: Merge TOML file if it exists
        if config_path.exists() {
            let toml_content = fs::read_to_string(config_path)
                .context(format!("Failed to read config file: {}", config_path.display()))?;

            let toml_config: Config = toml::from_str(&toml_content)
                .context("Failed to parse TOML configuration")?;

            config = Self::merge(config, toml_config);
        }

        // Step 3: Override with environment variables (CONF_ prefix)
        config = Self::merge_from_env(config)?;

        // Step 4: Apply debug flag from CLI if set
        if debug_flag {
            config.mode.debug = true;
        }

        Ok(config)
    }

    /// Merge two configurations (b overrides a where values are present)
    fn merge(a: Config, b: Config) -> Config {
        // Simply return b since we're doing a complete override
        // In a more sophisticated implementation, you'd merge field by field
        Config {
            mode: b.mode,
            server: b.server,
            db: DbConfig {
                mysql: b.db.mysql,
            },
            log: LogConfig {
                app: b.log.app,
            },
            background: BackgroundConfig {
                watchdog: b.background.watchdog,
            },
        }
    }

    /// Merge configuration from environment variables with CONF_ prefix
    fn merge_from_env(mut config: Config) -> Result<Self> {
        // Mode
        if let Ok(val) = std::env::var("CONF_MODE_DEBUG") {
            config.mode.debug = val.parse()
                .context("CONF_MODE_DEBUG must be 'true' or 'false'")?;
        }

        // Server
        if let Ok(val) = std::env::var("CONF_SERVER_EXTERNAL_URL") {
            config.server.external_url = val;
        }
        if let Ok(val) = std::env::var("CONF_SERVER_HTTP_PORT") {
            config.server.http_port = val.parse()
                .context("CONF_SERVER_HTTP_PORT must be a valid port number")?;
        }
        if let Ok(val) = std::env::var("CONF_SERVER_GRPC_PORT") {
            config.server.grpc_port = val.parse()
                .context("CONF_SERVER_GRPC_PORT must be a valid port number")?;
        }
        if let Ok(val) = std::env::var("CONF_SERVER_HEALTHZ_PORT") {
            config.server.healthz_port = val.parse()
                .context("CONF_SERVER_HEALTHZ_PORT must be a valid port number")?;
        }
        if let Ok(val) = std::env::var("CONF_SERVER_METRICS_PORT") {
            config.server.metrics_port = val.parse()
                .context("CONF_SERVER_METRICS_PORT must be a valid port number")?;
        }

        // Database - MySQL
        if let Ok(val) = std::env::var("CONF_DB_MYSQL_HOST") {
            config.db.mysql.host = val;
        }
        if let Ok(val) = std::env::var("CONF_DB_MYSQL_PORT") {
            config.db.mysql.port = val.parse()
                .context("CONF_DB_MYSQL_PORT must be a valid port number")?;
        }
        if let Ok(val) = std::env::var("CONF_DB_MYSQL_DATING") {
            config.db.mysql.database = val;
        }
        if let Ok(val) = std::env::var("CONF_DB_MYSQL_USER") {
            config.db.mysql.user = val;
        }
        if let Ok(val) = std::env::var("CONF_DB_MYSQL_PASS") {
            config.db.mysql.pass = val;
        }
        if let Ok(val) = std::env::var("CONF_DB_MYSQL_CONNECTIONS_MAX_IDLE") {
            config.db.mysql.connections.max_idle = val.parse()
                .context("CONF_DB_MYSQL_CONNECTIONS_MAX_IDLE must be a number")?;
        }
        if let Ok(val) = std::env::var("CONF_DB_MYSQL_CONNECTIONS_MAX_OPEN") {
            config.db.mysql.connections.max_open = val.parse()
                .context("CONF_DB_MYSQL_CONNECTIONS_MAX_OPEN must be a number")?;
        }

        // Log
        if let Ok(val) = std::env::var("CONF_LOG_APP_LEVEL") {
            config.log.app.level = val;
        }

        // Background - Watchdog
        if let Ok(val) = std::env::var("CONF_BACKGROUND_WATCHDOG_PERIOD") {
            config.background.watchdog.period = humantime::parse_duration(&val)
                .context("CONF_BACKGROUND_WATCHDOG_PERIOD must be a valid duration (e.g., '5s', '1m')")?;
        }
        if let Ok(val) = std::env::var("CONF_BACKGROUND_WATCHDOG_LIMIT") {
            config.background.watchdog.limit = val.parse()
                .context("CONF_BACKGROUND_WATCHDOG_LIMIT must be a number")?;
        }
        if let Ok(val) = std::env::var("CONF_BACKGROUND_WATCHDOG_LOCK_TIMEOUT") {
            config.background.watchdog.lock_timeout = humantime::parse_duration(&val)
                .context("CONF_BACKGROUND_WATCHDOG_LOCK_TIMEOUT must be a valid duration")?;
        }

        Ok(config)
    }

    /// Pretty-print the configuration
    fn print(&self) {
        println!("Configuration:");
        println!("==============");
        println!();
        
        println!("[mode]");
        println!("  debug = {}", self.mode.debug);
        println!();

        println!("[server]");
        println!("  external_url = \"{}\"", self.server.external_url);
        println!("  http_port = {}", self.server.http_port);
        println!("  grpc_port = {}", self.server.grpc_port);
        println!("  healthz_port = {}", self.server.healthz_port);
        println!("  metrics_port = {}", self.server.metrics_port);
        println!();

        println!("[db.mysql]");
        println!("  host = \"{}\"", self.db.mysql.host);
        println!("  port = {}", self.db.mysql.port);
        println!("  dating = \"{}\"", self.db.mysql.database);
        println!("  user = \"{}\"", self.db.mysql.user);
        println!("  pass = \"{}\"", if self.db.mysql.pass.is_empty() { "" } else { "***" });
        println!();

        println!("[db.mysql.connections]");
        println!("  max_idle = {}", self.db.mysql.connections.max_idle);
        println!("  max_open = {}", self.db.mysql.connections.max_open);
        println!();

        println!("[log.app]");
        println!("  level = \"{}\"", self.log.app.level);
        println!();

        println!("[background.watchdog]");
        println!("  period = \"{}\"", humantime::format_duration(self.background.watchdog.period));
        println!("  limit = {}", self.background.watchdog.limit);
        println!("  lock_timeout = \"{}\"", humantime::format_duration(self.background.watchdog.lock_timeout));
    }
}

// ============================================================================
// MAIN
// ============================================================================

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration with proper precedence
    let config = Config::load(&cli.conf, cli.debug)?;

    // Print the final merged configuration
    config.print();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(!config.mode.debug);
        assert_eq!(config.server.http_port, 8081);
        assert_eq!(config.db.mysql.host, "127.0.0.1");
    }

    #[test]
    fn test_env_override() {
        unsafe {
            env::set_var("CONF_MODE_DEBUG", "true");
            env::set_var("CONF_SERVER_HTTP_PORT", "9000");
        }

        let config = Config::merge_from_env(Config::default()).unwrap();
        assert!(config.mode.debug);
        assert_eq!(config.server.http_port, 9000);

        unsafe {
            env::remove_var("CONF_MODE_DEBUG");
            env::remove_var("CONF_SERVER_HTTP_PORT");
        }
    }

    #[test]
    fn test_merge_configs() {
        let config1 = Config::default();
        let mut config2 = Config::default();
        
        config2.mode.debug = true;
        config2.server.http_port = 9090;

        let merged = Config::merge(config1, config2);
        assert!(merged.mode.debug);
        assert_eq!(merged.server.http_port, 9090);
    }
}