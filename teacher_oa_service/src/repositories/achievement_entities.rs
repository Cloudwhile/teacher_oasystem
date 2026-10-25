pub mod term {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "academic_terms")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub academic_year: String,
        pub term_number: i32,
        pub name: String,
        pub starts_on: Date,
        pub ends_on: Date,
        pub enabled: bool,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}
pub mod dictionary {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "dictionary_items")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub group_code: String,
        #[sea_orm(primary_key, auto_increment = false)]
        pub code: String,
        pub name: String,
        pub sort_order: i32,
        pub enabled: bool,
        pub parent_code: Option<String>,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}
pub mod achievement_type {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "achievement_types")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub code: String,
        pub name: String,
        pub category: String,
        pub current_version: i32,
        pub published: bool,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}
pub mod schema_version {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "achievement_type_versions")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub type_code: String,
        #[sea_orm(primary_key, auto_increment = false)]
        pub version: i32,
        pub schema_document: Json,
        pub published_at: DateTimeUtc,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}
pub mod achievement {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "achievements")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub teacher_id: i64,
        pub type_code: String,
        pub schema_version: i32,
        pub title: String,
        pub subject_code: Option<String>,
        pub academic_term_id: Option<i64>,
        pub occurred_at: Option<Date>,
        pub level_code: Option<String>,
        pub award_grade_code: Option<String>,
        pub funding_amount: Option<Decimal>,
        pub attributes: Json,
        pub remarks: String,
        pub status: String,
        pub version: i32,
        pub created_by: i64,
        pub term_mismatch_confirmed: bool,
        pub created_at: DateTimeUtc,
        pub updated_at: DateTimeUtc,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}
pub mod participant {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "achievement_participants")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub achievement_id: i64,
        pub teacher_id: Option<i64>,
        pub external_name: String,
        pub role_code: String,
        pub sort_order: i32,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}
pub mod file {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "files")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub object_key: String,
        pub filename: String,
        pub size_bytes: i64,
        pub mime_type: String,
        pub sha256: String,
        pub uploaded_by: i64,
        pub upload_status: String,
        pub created_at: DateTimeUtc,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}
pub mod achievement_file {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "achievement_files")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub achievement_id: i64,
        #[sea_orm(primary_key, auto_increment = false)]
        pub file_id: i64,
        pub purpose: String,
        pub sort_order: i32,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}
pub mod revision {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "achievement_revisions")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub achievement_id: i64,
        pub version: i32,
        pub schema_version: i32,
        pub snapshot: Json,
        pub modified_by: i64,
        pub created_at: DateTimeUtc,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}
pub mod review_event {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "review_events")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub achievement_id: i64,
        pub reviewer_id: i64,
        pub action: String,
        pub old_status: String,
        pub new_status: String,
        pub opinion: String,
        pub created_at: DateTimeUtc,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

pub mod revision_file {
    use sea_orm::entity::prelude::*;
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "achievement_revision_files")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub revision_id: i64,
        #[sea_orm(primary_key, auto_increment = false)]
        pub file_id: i64,
        pub achievement_id: i64,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}
