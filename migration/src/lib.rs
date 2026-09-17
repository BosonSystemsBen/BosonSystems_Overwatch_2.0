pub use sea_orm_migration::prelude::*;

mod m20240101_000001_create_products;
mod m20240101_000002_create_serial_numbers;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_products::Migration),
            Box::new(m20240101_000002_create_serial_numbers::Migration),
        ]
    }
}
