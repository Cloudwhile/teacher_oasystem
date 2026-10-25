use crate::schema_helpers::*;
use sea_orm_migration::prelude::*;
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_table(
            Table::create()
                .table(n("files"))
                .col(id())
                .col(text("object_key", 200).unique_key())
                .col(text("filename", 255))
                .col(bigint("size_bytes"))
                .col(text("mime_type", 100))
                .col(text("sha256", 64))
                .col(bigint("uploaded_by"))
                .col(text("upload_status", 20))
                .col(time("created_at"))
                .check(Expr::cust(
                    "size_bytes > 0 AND upload_status IN ('uploading', 'ready', 'failed')",
                ))
                .foreign_key(&mut fk("files", "uploaded_by", "users", "id"))
                .to_owned(),
        )
        .await?;
        m.create_table(Table::create().table(n("achievements"))
            .col(id()).col(bigint("teacher_id")).col(text("type_code",64)).col(integer("schema_version"))
            .col(text("title",500)).col(ColumnDef::new(n("subject_code")).string_len(64))
            .col(ColumnDef::new(n("academic_term_id")).big_integer())
            .col(ColumnDef::new(n("occurred_at")).date()).col(ColumnDef::new(n("level_code")).string_len(64))
            .col(ColumnDef::new(n("award_grade_code")).string_len(64))
            .col(ColumnDef::new(n("funding_amount")).decimal_len(18,2))
            .col(ColumnDef::new(n("attributes")).json_binary().not_null())
            .col(ColumnDef::new(n("remarks")).text().not_null()).col(text("status",20).default("draft"))
            .col(integer("version").default(1)).col(bigint("created_by"))
            .col(ColumnDef::new(n("term_mismatch_confirmed")).boolean().not_null().default(false))
            .col(time("created_at")).col(time("updated_at"))
            .check(Expr::cust("status IN ('draft', 'pending', 'returned', 'approved', 'archived') AND version > 0 AND schema_version > 0 AND (funding_amount IS NULL OR funding_amount >= 0)"))
            .foreign_key(&mut fk("achievements","teacher_id","teachers","id"))
            .foreign_key(&mut fk("achievements","created_by","users","id"))
            .foreign_key(&mut fk("achievements","academic_term_id","academic_terms","id"))
            .foreign_key(ForeignKey::create().name("fk_achievement_frozen_schema")
                .from(n("achievements"),(n("type_code"),n("schema_version")))
                .to(n("achievement_type_versions"),(n("type_code"),n("version"))).on_delete(ForeignKeyAction::Restrict))
            .to_owned()).await?;
        m.create_table(
            Table::create()
                .table(n("achievement_participants"))
                .col(id())
                .col(bigint("achievement_id"))
                .col(ColumnDef::new(n("teacher_id")).big_integer())
                .col(text("external_name", 100))
                .col(text("role_code", 32))
                .col(integer("sort_order"))
                .check(Expr::cust("sort_order >= 0"))
                .index(
                    Index::create()
                        .name("uq_participant_order")
                        .unique()
                        .col(n("achievement_id"))
                        .col(n("sort_order")),
                )
                .foreign_key(&mut fk(
                    "achievement_participants",
                    "achievement_id",
                    "achievements",
                    "id",
                ))
                .foreign_key(&mut fk(
                    "achievement_participants",
                    "teacher_id",
                    "teachers",
                    "id",
                ))
                .to_owned(),
        )
        .await?;
        m.create_table(
            Table::create()
                .table(n("achievement_files"))
                .col(bigint("achievement_id"))
                .col(bigint("file_id"))
                .col(text("purpose", 32))
                .col(integer("sort_order"))
                .primary_key(Index::create().col(n("achievement_id")).col(n("file_id")))
                .foreign_key(&mut fk(
                    "achievement_files",
                    "achievement_id",
                    "achievements",
                    "id",
                ))
                .foreign_key(&mut fk("achievement_files", "file_id", "files", "id"))
                .to_owned(),
        )
        .await?;
        m.create_table(
            Table::create()
                .table(n("achievement_revisions"))
                .col(id())
                .col(bigint("achievement_id"))
                .col(integer("version"))
                .col(integer("schema_version"))
                .col(ColumnDef::new(n("snapshot")).json_binary().not_null())
                .col(bigint("modified_by"))
                .col(time("created_at"))
                .index(
                    Index::create()
                        .name("uq_achievement_revision")
                        .unique()
                        .col(n("achievement_id"))
                        .col(n("version")),
                )
                .foreign_key(&mut fk(
                    "achievement_revisions",
                    "achievement_id",
                    "achievements",
                    "id",
                ))
                .foreign_key(&mut fk(
                    "achievement_revisions",
                    "modified_by",
                    "users",
                    "id",
                ))
                .to_owned(),
        )
        .await?;
        m.create_table(
            Table::create()
                .table(n("achievement_revision_files"))
                .col(bigint("revision_id"))
                .col(bigint("achievement_id"))
                .col(bigint("file_id"))
                .primary_key(Index::create().col(n("revision_id")).col(n("file_id")))
                .foreign_key(&mut fk(
                    "achievement_revision_files",
                    "revision_id",
                    "achievement_revisions",
                    "id",
                ))
                .foreign_key(&mut fk(
                    "achievement_revision_files",
                    "achievement_id",
                    "achievements",
                    "id",
                ))
                .foreign_key(&mut fk(
                    "achievement_revision_files",
                    "file_id",
                    "files",
                    "id",
                ))
                .to_owned(),
        )
        .await?;
        m.create_table(
            Table::create()
                .table(n("review_events"))
                .col(id())
                .col(bigint("achievement_id"))
                .col(bigint("reviewer_id"))
                .col(text("action", 32))
                .col(text("old_status", 20))
                .col(text("new_status", 20))
                .col(ColumnDef::new(n("opinion")).text().not_null())
                .col(time("created_at"))
                .foreign_key(&mut fk(
                    "review_events",
                    "achievement_id",
                    "achievements",
                    "id",
                ))
                .foreign_key(&mut fk("review_events", "reviewer_id", "users", "id"))
                .to_owned(),
        )
        .await?;
        for (table, columns, index) in [
            (
                "achievements",
                vec!["teacher_id"],
                "idx_achievements_teacher",
            ),
            (
                "achievements",
                vec!["type_code", "academic_term_id"],
                "idx_achievements_type_term",
            ),
            ("achievements", vec!["status"], "idx_achievements_status"),
            (
                "review_events",
                vec!["achievement_id", "created_at"],
                "idx_review_events_achievement_time",
            ),
            (
                "files",
                vec!["uploaded_by", "created_at"],
                "idx_files_uploader_time",
            ),
            (
                "achievement_files",
                vec!["file_id"],
                "idx_achievement_files_file",
            ),
            (
                "achievement_revision_files",
                vec!["achievement_id", "file_id"],
                "idx_revision_files_achievement_file",
            ),
            (
                "achievement_participants",
                vec!["teacher_id"],
                "idx_participants_teacher",
            ),
        ] {
            let mut statement = Index::create();
            statement.name(index).table(n(table));
            for column in columns {
                statement.col(n(column));
            }
            m.create_index(statement.to_owned()).await?;
        }
        Ok(())
    }
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        for table in [
            "review_events",
            "achievement_revision_files",
            "achievement_revisions",
            "achievement_files",
            "achievement_participants",
            "achievements",
            "files",
        ] {
            m.drop_table(Table::drop().table(n(table)).to_owned())
                .await?;
        }
        Ok(())
    }
}
