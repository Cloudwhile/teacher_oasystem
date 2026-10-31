use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LauncherSettings {
    pub executable: String,
    pub working_directory: String,
    pub config_path: String,
    pub base_url: String,
}

impl Default for LauncherSettings {
    fn default() -> Self {
        let executable = std::env::current_exe().unwrap_or_default();
        let executable_dir = executable.parent().unwrap_or(Path::new("."));
        let root = std::env::current_dir().unwrap_or_default();
        let service = if root.join("config.example.toml").is_file() {
            root
        } else if root.join("teacher_oa_service").is_dir() {
            root.join("teacher_oa_service")
        } else {
            executable_dir.to_path_buf()
        };
        let server = if executable_dir.join("teacher_oa_service.exe").is_file() {
            executable_dir.join("teacher_oa_service.exe")
        } else {
            service.join("target/debug/teacher_oa_service.exe")
        };
        Self {
            executable: server.to_string_lossy().into_owned(),
            working_directory: service.to_string_lossy().into_owned(),
            config_path: [
                "config.toml",
                "config.development.toml",
                "config.example.toml",
            ]
            .iter()
            .map(|name| service.join(name))
            .find(|path| path.is_file())
            .unwrap_or_else(|| service.join("config.toml"))
            .to_string_lossy()
            .into_owned(),
            base_url: "http://127.0.0.1:3000".into(),
        }
    }
}

impl LauncherSettings {
    pub fn validate(&self) -> Result<(), String> {
        for (value, label, directory) in [
            (&self.executable, "服务程序", false),
            (&self.working_directory, "工作目录", true),
            (&self.config_path, "配置文件", false),
        ] {
            let path = Path::new(value);
            if !path.is_absolute()
                || (if directory {
                    !path.is_dir()
                } else {
                    !path.is_file()
                })
            {
                return Err(format!("{label}不存在，请选择有效的绝对路径"));
            }
        }
        validate_base_url(&self.base_url)?;
        Ok(())
    }
}

pub fn validate_base_url(value: &str) -> Result<url::Url, String> {
    let url = url::Url::parse(value).map_err(|_| "本机服务地址无效")?;
    let loopback = match url.host() {
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        _ => false,
    };
    if url.scheme() != "http"
        || !loopback
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
        || url.port_or_known_default() == Some(0)
    {
        return Err("请使用本机 HTTP 地址，例如 http://127.0.0.1:3000".into());
    }
    Ok(url)
}

pub fn settings_path() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("TeacherOA/desktop-manager.json")
}

pub fn load() -> Result<LauncherSettings, String> {
    let path = settings_path();
    match std::fs::read(&path) {
        Ok(data) => {
            serde_json::from_slice(&data).map_err(|_| "启动设置格式无效，请重新保存".into())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(LauncherSettings::default())
        }
        Err(_) => Err("无法读取启动设置，请检查文件权限".into()),
    }
}

pub fn save(settings: &LauncherSettings) -> Result<(), String> {
    settings.validate()?;
    let path = settings_path();
    let parent = path.parent().ok_or("启动设置路径无效")?;
    let write = || -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(parent)?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        temporary.write_all(&serde_json::to_vec_pretty(settings)?)?;
        temporary.as_file().sync_all()?;
        temporary.persist(&path)?;
        Ok(())
    };
    write().map_err(|_| "无法保存启动设置，请检查目录权限".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capability_is_only_sent_to_literal_loopback_origins() {
        for url in ["http://127.0.0.1:3000", "http://[::1]:3000"] {
            assert!(validate_base_url(url).is_ok(), "{url}");
        }
        for url in [
            "http://localhost:3000",
            "http://127.0.0.1.evil.test",
            "https://127.0.0.1",
            "http://127.0.0.1/api",
            "http://user@127.0.0.1",
            "http://127.0.0.1?token=x",
            "http://192.168.1.2:3000",
        ] {
            assert!(validate_base_url(url).is_err(), "{url}");
        }
    }
}
