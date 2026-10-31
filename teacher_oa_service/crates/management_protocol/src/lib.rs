//! Contract shared by the service and desktop client. No storage or database dependencies.
use serde::{Deserialize, Serialize};

pub const API_PREFIX: &str = "/api/v1/management";
pub const TOKEN_ENV: &str = "TEACHER_OA_MANAGEMENT_TOKEN";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DependencyStatus {
    pub name: String,
    pub healthy: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub service: String,
    pub version: String,
    pub pid: u32,
    pub uptime_seconds: u64,
    pub stopping: bool,
    pub dependencies: Vec<DependencyStatus>,
}

/// Deliberately excludes credentials, database URLs, and business records.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub log_level: String,
    pub log_format: String,
    pub pool_size: u32,
}

impl ServiceConfig {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.host.parse::<std::net::IpAddr>().is_err() || self.port == 0 {
            return Err("请填写有效的监听 IP 和端口（1～65535）");
        }
        if self.workers > 128 || !(1..=256).contains(&self.pool_size) {
            return Err("工作线程应为0～128，数据库连接池应为1～256");
        }
        if !["trace", "debug", "info", "warn", "error"].contains(&self.log_level.as_str())
            || !["text", "json"].contains(&self.log_format.as_str())
        {
            return Err("请选择有效的日志级别和格式");
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigSnapshot {
    pub active: ServiceConfig,
    pub saved: ServiceConfig,
    pub revision: String,
    pub restart_required: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveConfig {
    pub config: ServiceConfig,
    pub revision: String,
}
