//! Restricted local operations exposed to the desktop supervisor through HTTP.
use crate::{api::error::ApiError, config::AppConfig};
use management_protocol::{ConfigSnapshot, SaveConfig, ServiceConfig};
use sha2::{Digest, Sha256};
use std::{io::Write, path::PathBuf, sync::Mutex, time::Instant};
use subtle::ConstantTimeEq;

pub struct Management {
    token: String,
    config_path: PathBuf,
    started: Instant,
    write_lock: Mutex<()>,
}

impl Management {
    pub fn new(token: String, config_path: PathBuf) -> std::io::Result<Self> {
        if token.len() != 64 || !token.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(std::io::Error::other("桌面管理凭据格式无效"));
        }
        Ok(Self {
            token,
            config_path: config_path.canonicalize()?,
            started: Instant::now(),
            write_lock: Mutex::new(()),
        })
    }

    pub fn matches(&self, token: &str) -> bool {
        self.token.as_bytes().ct_eq(token.as_bytes()).into()
    }

    pub fn uptime(&self) -> u64 {
        self.started.elapsed().as_secs()
    }

    pub fn snapshot(&self, active: &AppConfig) -> Result<ConfigSnapshot, ApiError> {
        let content = std::fs::read_to_string(&self.config_path).map_err(internal)?;
        snapshot(active, &content)
    }

    pub fn save(
        &self,
        active: &AppConfig,
        request: SaveConfig,
    ) -> Result<ConfigSnapshot, ApiError> {
        request.config.validate().map_err(ApiError::Invalid)?;
        let ip: std::net::IpAddr = request
            .config
            .host
            .parse()
            .map_err(|_| ApiError::Invalid("监听 IP 无效"))?;
        if !ip.is_loopback() && !ip.is_unspecified() {
            return Err(ApiError::Invalid("桌面管理模式请监听本机地址或全部网卡"));
        }
        // Do not report success for values that a startup environment override would replace.
        for key in [
            "SERVER__HOST",
            "SERVER__PORT",
            "SERVER__WORKERS",
            "LOGGING__LEVEL",
            "LOGGING__FORMAT",
            "DATABASE__POOL_SIZE",
        ] {
            if std::env::var_os(format!("ASTER__{key}")).is_some() {
                return Err(ApiError::Conflict(
                    "服务配置由环境变量覆盖，请先移除对应覆盖后再保存",
                ));
            }
        }
        let _guard = self.write_lock.lock().map_err(internal)?;
        let content = std::fs::read_to_string(&self.config_path).map_err(internal)?;
        if revision(&content) != request.revision {
            return Err(ApiError::Conflict("配置文件已被修改，请重新读取后再保存"));
        }
        let mut document: toml::Value = toml::from_str(&content).map_err(internal)?;
        let saved: AppConfig = toml::from_str(&content).map_err(internal)?;
        let config = request.config;
        // Match the desktop's direct loopback address, including wildcard listeners.
        let direct_origin = |host: &str, port| -> Result<String, ApiError> {
            let ip: std::net::IpAddr = host.parse().map_err(internal)?;
            let loopback = if ip.is_unspecified() {
                if ip.is_ipv6() {
                    std::net::IpAddr::V6(std::net::Ipv6Addr::LOCALHOST)
                } else {
                    std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
                }
            } else {
                ip
            };
            let address = std::net::SocketAddr::new(loopback, port);
            Ok(url::Url::parse(&format!("http://{address}"))
                .map_err(internal)?
                .origin()
                .ascii_serialization())
        };
        let old_ip: std::net::IpAddr = saved.server.host.parse().map_err(internal)?;
        if (old_ip.is_loopback() || old_ip.is_unspecified())
            && saved.auth.public_origin == direct_origin(&saved.server.host, saved.server.port)?
        {
            let auth = document
                .as_table_mut()
                .ok_or(ApiError::Internal)?
                .entry("auth")
                .or_insert_with(|| toml::Value::Table(Default::default()));
            auth.as_table_mut().ok_or(ApiError::Internal)?.insert(
                "public_origin".into(),
                toml::Value::String(direct_origin(&config.host, config.port)?),
            );
        }
        // Update only the allowlist and its matching direct origin; preserve external origins.
        for (section, key, value) in [
            ("server", "host", toml::Value::String(config.host)),
            (
                "server",
                "port",
                toml::Value::Integer(i64::from(config.port)),
            ),
            (
                "server",
                "workers",
                toml::Value::Integer(i64::try_from(config.workers).map_err(internal)?),
            ),
            ("logging", "level", toml::Value::String(config.log_level)),
            ("logging", "format", toml::Value::String(config.log_format)),
            (
                "database",
                "pool_size",
                toml::Value::Integer(i64::from(config.pool_size)),
            ),
        ] {
            let table = document
                .as_table_mut()
                .ok_or(ApiError::Internal)?
                .entry(section)
                .or_insert_with(|| toml::Value::Table(Default::default()));
            table
                .as_table_mut()
                .ok_or(ApiError::Internal)?
                .insert(key.into(), value);
        }
        let updated = toml::to_string_pretty(&document).map_err(internal)?;
        let parent = self.config_path.parent().ok_or(ApiError::Internal)?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(internal)?;
        temporary
            .as_file()
            .set_permissions(
                std::fs::metadata(&self.config_path)
                    .map_err(internal)?
                    .permissions(),
            )
            .map_err(internal)?;
        temporary.write_all(updated.as_bytes()).map_err(internal)?;
        temporary.as_file().sync_all().map_err(internal)?;
        temporary.persist(&self.config_path).map_err(internal)?;
        tracing::info!("desktop management saved service configuration; restart required");
        snapshot(active, &updated)
    }
}

pub fn editable(config: &AppConfig) -> ServiceConfig {
    ServiceConfig {
        host: config.server.host.clone(),
        port: config.server.port,
        workers: config.server.workers,
        log_level: config.logging.level.clone(),
        log_format: config.logging.format.clone(),
        pool_size: config.database.pool_size,
    }
}

fn snapshot(active: &AppConfig, content: &str) -> Result<ConfigSnapshot, ApiError> {
    let config: AppConfig = toml::from_str(content).map_err(internal)?;
    let active = editable(active);
    let saved = editable(&config);
    Ok(ConfigSnapshot {
        restart_required: active != saved,
        active,
        saved,
        revision: revision(content),
    })
}

fn revision(content: &str) -> String {
    Sha256::digest(content.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn internal(error: impl std::fmt::Display) -> ApiError {
    // Error strings from TOML parsing can include source secrets, so never forward or log them.
    let _ = error;
    ApiError::Internal
}
