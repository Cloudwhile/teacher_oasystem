# TeacherOA 应用

本目录包含 TeacherOA 服务端、界面、数据库迁移及测试。

环境准备、数据库配置和完整启动步骤见仓库根目录的 [README](../README.md)。

已经构建界面并准备数据库后，首次复制 `.env.example` 为 `.env` 并按需编辑，随后直接启动：

```bash
cargo +stable run --features openapi,metrics
```

访问 <http://127.0.0.1:3000>。
