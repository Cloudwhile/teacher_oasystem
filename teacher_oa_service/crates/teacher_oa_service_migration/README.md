# TeacherOA 数据库迁移

本模块维护应用所需的数据表和初始数据。应用启动时会自动执行待运行的迁移。

手动运行迁移命令前，在 `teacher_oa_service/.env` 中配置 `DATABASE_URL`。迁移工具会自动读取该文件。

查看迁移状态：

```bash
cargo +stable run -p teacher_oa_service_migration -- status
```

执行待运行的迁移：

```bash
cargo +stable run -p teacher_oa_service_migration -- up
```

数据库准备和应用启动方法见 [项目 README](../../../README.md)。
