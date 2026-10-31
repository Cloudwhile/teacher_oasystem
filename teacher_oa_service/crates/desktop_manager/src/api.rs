use management_protocol::{API_PREFIX, ConfigSnapshot, SaveConfig, ServiceStatus};
use reqwest::blocking::{Client, Response};
use serde::{Deserialize, de::DeserializeOwned};
use std::time::Duration;

pub struct ManagementApi {
    client: Client,
    base: String,
    token: String,
}

#[derive(Deserialize)]
struct Envelope<T> {
    data: T,
}

impl ManagementApi {
    pub fn new(base: &str, token: String) -> Result<Self, String> {
        crate::settings::validate_base_url(base)?;
        Ok(Self {
            client: Client::builder()
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
                .connect_timeout(Duration::from_secs(1))
                .timeout(Duration::from_secs(4))
                .build()
                .map_err(|_| "无法初始化管理连接")?,
            base: base.trim_end_matches('/').into(),
            token,
        })
    }
    fn endpoint(&self, path: &str) -> String {
        format!("{}{API_PREFIX}/{path}", self.base)
    }
    pub fn status(&self) -> Result<ServiceStatus, String> {
        decode(
            self.client
                .get(self.endpoint("status"))
                .bearer_auth(&self.token)
                .send(),
        )
    }
    pub fn config(&self) -> Result<ConfigSnapshot, String> {
        decode(
            self.client
                .get(self.endpoint("config"))
                .bearer_auth(&self.token)
                .send(),
        )
    }
    pub fn save_config(&self, value: &SaveConfig) -> Result<ConfigSnapshot, String> {
        decode(
            self.client
                .put(self.endpoint("config"))
                .bearer_auth(&self.token)
                .json(value)
                .send(),
        )
    }
    pub fn shutdown(&self) -> Result<(), String> {
        let _: serde_json::Value = decode(
            self.client
                .post(self.endpoint("shutdown"))
                .bearer_auth(&self.token)
                .send(),
        )?;
        Ok(())
    }
}

fn decode<T: DeserializeOwned>(response: Result<Response, reqwest::Error>) -> Result<T, String> {
    let response = response.map_err(|_| "无法连接服务，请查看运行日志")?;
    if !response.status().is_success() {
        return Err(match response.status().as_u16() {
            400 => "配置内容无效，请检查各项取值",
            401 | 403 => "管理连接未通过验证，请确认地址属于当前启动的服务",
            404 => "服务不支持桌面管理，请使用更新后的服务程序",
            409 => "配置已变更或由环境变量覆盖，请重新读取后检查",
            _ => "服务操作失败，请查看运行日志",
        }
        .into());
    }
    response
        .json::<Envelope<T>>()
        .map(|body| body.data)
        .map_err(|_| "服务返回了无法识别的数据".into())
}
