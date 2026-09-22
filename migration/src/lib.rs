pub use sea_orm_migration::prelude::*;

pub mod m20240101_000001_create_products;
pub mod m20240101_000002_create_serial_numbers;
mod m20240101_000003_add_pennylane_product_id;
pub mod m20240101_000004_create_shipments;
pub mod m20240101_000005_create_shipment_lines;
mod m20240101_000006_add_shipment_line_to_serial_numbers;
mod m20240101_000007_create_sendcloud_labels;
mod m20240101_000008_add_amount_to_shipment_lines;
mod m20240101_000009_create_sendcloud_orders;
mod m20240101_000010_drop_sendcloud_labels;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_products::Migration),
            Box::new(m20240101_000002_create_serial_numbers::Migration),
            Box::new(m20240101_000003_add_pennylane_product_id::Migration),
            Box::new(m20240101_000004_create_shipments::Migration),
            Box::new(m20240101_000005_create_shipment_lines::Migration),
            Box::new(m20240101_000006_add_shipment_line_to_serial_numbers::Migration),
            Box::new(m20240101_000007_create_sendcloud_labels::Migration),
            Box::new(m20240101_000008_add_amount_to_shipment_lines::Migration),
            Box::new(m20240101_000009_create_sendcloud_orders::Migration),
            Box::new(m20240101_000010_drop_sendcloud_labels::Migration),
        ]
    }
}
