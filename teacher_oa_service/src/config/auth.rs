use serde::{Deserialize, Serialize};

/// Product authentication settings. Secrets come from startup .env loading or process variables.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct AuthConfig {
    pub environment: String,
    pub mock_enabled: bool,
    pub mock_seed: bool,
    pub cookie_secure: bool,
    pub public_origin: String,
    pub session_ttl_seconds: u64,
    pub store: String,
    pub wecom_corp_id: String,
    pub wecom_agent_id: String,
    pub wecom_redirect_uri: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            environment: "development".into(),
            mock_enabled: false,
            mock_seed: false,
            cookie_secure: true,
            public_origin: "http://127.0.0.1:3000".into(),
            session_ttl_seconds: 28_800,
            store: "memory".into(),
            wecom_corp_id: String::new(),
            wecom_agent_id: String::new(),
            wecom_redirect_uri: String::new(),
        }
    }
}

impl AuthConfig {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::errors::AppError;
        let invalid = |message: &str| Err(AppError::Config(message.into()));
        if !["development", "test", "production"].contains(&self.environment.as_str()) {
            return invalid("unsupported auth environment");
        }
        if !["memory", "redis"].contains(&self.store.as_str()) {
            return invalid("unsupported auth store");
        }
        if self.mock_seed && !self.mock_enabled {
            return invalid("mock seed requires mock authentication");
        }
        let origin = url::Url::parse(&self.public_origin)
            .map_err(|_| AppError::Config("auth.public_origin must be an absolute URL".into()))?;
        if origin.origin().ascii_serialization() != self.public_origin
            || !origin.username().is_empty()
            || origin.password().is_some()
            || !["http", "https"].contains(&origin.scheme())
        {
            return invalid("auth.public_origin must contain only scheme, host and optional port");
        }
        if !(60..=86_400).contains(&self.session_ttl_seconds) {
            return invalid("session lifetime must be between 60 and 86400 seconds");
        }
        if self.environment == "production"
            && (self.mock_enabled
                || self.mock_seed
                || !self.cookie_secure
                || origin.scheme() != "https")
        {
            return invalid(
                "production requires HTTPS, secure cookies and disabled mock authentication",
            );
        }
        if !self.wecom_redirect_uri.is_empty() {
            let callback = url::Url::parse(&self.wecom_redirect_uri)
                .map_err(|_| AppError::Config("invalid WeCom callback URL".into()))?;
            if callback.origin() != origin.origin()
                || callback.path() != "/api/v1/auth/wecom/callback"
                || callback.query().is_some()
                || callback.fragment().is_some()
                || !callback.username().is_empty()
                || callback.password().is_some()
            {
                return invalid(
                    "WeCom callback must use the configured public origin and fixed callback path",
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_allows_optional_redis_but_requires_secure_authentication() {
        let base = AuthConfig {
            environment: "production".into(),
            store: "redis".into(),
            public_origin: "https://oa.example.edu.cn".into(),
            ..Default::default()
        };
        assert!(base.validate().is_ok());
        assert!(
            AuthConfig {
                store: "memory".into(),
                ..base.clone()
            }
            .validate()
            .is_ok()
        );
        for config in [
            AuthConfig {
                mock_enabled: true,
                ..base.clone()
            },
            AuthConfig {
                cookie_secure: false,
                ..base.clone()
            },
            AuthConfig {
                wecom_redirect_uri: "https://other.example/api/v1/auth/wecom/callback".into(),
                ..base.clone()
            },
        ] {
            assert!(config.validate().is_err());
        }
    }
}
