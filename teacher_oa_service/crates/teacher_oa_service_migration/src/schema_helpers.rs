use sea_orm_migration::prelude::*;
pub fn n(value: &str) -> Alias {
    Alias::new(value)
}
pub fn id() -> ColumnDef {
    let mut c = ColumnDef::new(n("id"));
    c.big_integer().not_null().auto_increment().primary_key();
    c
}
pub fn text(value: &str, length: u32) -> ColumnDef {
    let mut c = ColumnDef::new(n(value));
    c.string_len(length).not_null();
    c
}
pub fn bigint(value: &str) -> ColumnDef {
    let mut c = ColumnDef::new(n(value));
    c.big_integer().not_null();
    c
}
pub fn integer(value: &str) -> ColumnDef {
    let mut c = ColumnDef::new(n(value));
    c.integer().not_null();
    c
}
pub fn time(value: &str) -> ColumnDef {
    let mut c = ColumnDef::new(n(value));
    c.timestamp_with_time_zone()
        .not_null()
        .default(Expr::current_timestamp());
    c
}
pub fn fk(table: &str, column: &str, target: &str, key: &str) -> ForeignKeyCreateStatement {
    ForeignKey::create()
        .name(format!("fk_{table}_{column}"))
        .from(n(table), n(column))
        .to(n(target), n(key))
        .on_delete(ForeignKeyAction::Restrict)
        .to_owned()
}
pub async fn insert(
    manager: &SchemaManager<'_>,
    table: &str,
    columns: &[&str],
    values: Vec<SimpleExpr>,
) -> Result<(), DbErr> {
    let statement = Query::insert()
        .into_table(n(table))
        .columns(columns.iter().map(|v| n(v)))
        .values(values)
        .map_err(|e| DbErr::Custom(e.to_string()))?
        .to_owned();
    manager.get_connection().execute(&statement).await?;
    Ok(())
}
