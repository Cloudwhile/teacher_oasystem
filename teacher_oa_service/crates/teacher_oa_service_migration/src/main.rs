use sea_orm_migration::cli;
use teacher_oa_service_migration::Migrator;

#[tokio::main]
async fn main() {
    cli::run_cli(Migrator).await;
}
