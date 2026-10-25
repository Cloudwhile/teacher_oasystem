use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct StorageConfig {
    pub backend: String,
    pub root: String,
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
    pub max_upload_bytes: usize,
}
impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: "local".into(),
            root: "private-files".into(),
            endpoint: "http://127.0.0.1:59000".into(),
            bucket: "teacheroa-private".into(),
            region: "us-east-1".into(),
            max_upload_bytes: 10 * 1024 * 1024,
        }
    }
}
