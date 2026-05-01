/// Server configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Bind address
    pub bind: String,
    /// Port
    pub port: u16,
    /// Max number of connected clients
    pub maxclients: usize,
    /// Log level
    pub loglevel: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            bind: "127.0.0.1".to_string(),
            port: 6379,
            maxclients: 10000,
            loglevel: "info".to_string(),
        }
    }
}

impl Config {
    pub fn address(&self) -> String {
        format!("{}:{}", self.bind, self.port)
    }
}
