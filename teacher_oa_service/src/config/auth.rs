use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Clone, Copy, Debug)]
enum CidrRule {
    V4 { network: u32, mask: u32 },
    V6 { network: u128, mask: u128 },
}

fn v4_mask(prefix: u8) -> Option<u32> {
    if prefix == 0 {
        Some(0)
    } else if prefix <= 32 {
        Some(!0_u32 << (32 - prefix))
    } else {
        None
    }
}

fn v6_mask(prefix: u8) -> Option<u128> {
    if prefix == 0 {
        Some(0)
    } else if prefix <= 128 {
        Some(!0_u128 << (128 - prefix))
    } else {
        None
    }
}

fn parse_cidr(raw: &str) -> Option<CidrRule> {
    if raw.is_empty() || raw.trim() != raw {
        return None;
    }
    match raw.split_once('/') {
        Some((ip_part, prefix_part)) => {
            let addr: IpAddr = ip_part.parse().ok()?;
            let prefix: u8 = prefix_part.parse().ok()?;
            match addr {
                IpAddr::V4(v4) => {
                    let mask = v4_mask(prefix)?;
                    Some(CidrRule::V4 {
                        network: u32::from(v4) & mask,
                        mask,
                    })
                }
                IpAddr::V6(v6) => {
                    let mask = v6_mask(prefix)?;
                    Some(CidrRule::V6 {
                        network: u128::from(v6) & mask,
                        mask,
                    })
                }
            }
        }
        None => match raw.parse::<IpAddr>().ok()? {
            IpAddr::V4(v4) => Some(CidrRule::V4 {
                network: u32::from(v4),
                mask: u32::MAX,
            }),
            IpAddr::V6(v6) => Some(CidrRule::V6 {
                network: u128::from(v6),
                mask: u128::MAX,
            }),
        },
    }
}

impl CidrRule {
    fn contains(self, ip: IpAddr) -> bool {
        match self {
            Self::V4 { network, mask } => {
                let candidate = match ip {
                    IpAddr::V4(v4) => Some(v4),
                    IpAddr::V6(v6) => v6.to_ipv4_mapped(),
                };
                candidate.is_some_and(|v4| (u32::from(v4) & mask) == network)
            }
            Self::V6 { network, mask } => match ip {
                IpAddr::V6(v6) => {
                    if (u128::from(v6) & mask) == network {
                        return true;
                    }
                    v6.to_ipv4_mapped()
                        .is_some_and(|v4| (u128::from(v4.to_ipv6_mapped()) & mask) == network)
                }
                IpAddr::V4(v4) => (u128::from(v4.to_ipv6_mapped()) & mask) == network,
            },
        }
    }
}

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
    pub trusted_proxies: Vec<String>,
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
            trusted_proxies: Vec::new(),
            wecom_corp_id: String::new(),
            wecom_agent_id: String::new(),
            wecom_redirect_uri: String::new(),
        }
    }
}

impl AuthConfig {
    pub fn is_trusted_proxy(&self, ip: IpAddr) -> bool {
        self.trusted_proxies
            .iter()
            .any(|entry| parse_cidr(entry).is_some_and(|rule| rule.contains(ip)))
    }

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
        if self
            .trusted_proxies
            .iter()
            .any(|entry| parse_cidr(entry).is_none())
        {
            return invalid("auth.trusted_proxies entries must be valid IPs or CIDRs");
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
