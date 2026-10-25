pub fn database_url() -> String {
    if let Ok(url) = std::env::var("ASTER_TEST_DATABASE_URL") {
        return url;
    }
    let variables = dotenvy::dotenv_iter()
        .map_err(|_| "无法读取测试用 .env")
        .and_then(|values| {
            values
                .collect::<Result<std::collections::BTreeMap<_, _>, _>>()
                .map_err(|_| "测试用 .env 格式错误")
        })
        .expect("测试配置不可用");
    variables
        .get("ASTER_TEST_DATABASE_URL")
        .cloned()
        .expect("请在 .env 中配置独立的 ASTER_TEST_DATABASE_URL")
}
