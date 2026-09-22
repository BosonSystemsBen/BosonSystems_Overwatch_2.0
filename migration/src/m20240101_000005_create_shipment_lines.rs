use sea_orm_migration::prelude::*;

use crate::m20240101_000001_create_products::Product;
use crate::m20240101_000004_create_shipments::Shipment;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ShipmentLine::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ShipmentLine::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ShipmentLine::ShipmentId).uuid().not_null())
                    .col(ColumnDef::new(ShipmentLine::PennylaneLineId).big_integer().not_null())
                    .col(ColumnDef::new(ShipmentLine::ProductId).uuid().null())
                    .col(ColumnDef::new(ShipmentLine::Label).string().not_null())
                    .col(ColumnDef::new(ShipmentLine::Quantity).string().not_null())
                    .col(
                        ColumnDef::new(ShipmentLine::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ShipmentLine::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_shipment_lines_shipment")
                            .from(ShipmentLine::Table, ShipmentLine::ShipmentId)
                            .to(Shipment::Table, Shipment::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_shipment_lines_product")
                            .from(ShipmentLine::Table, ShipmentLine::ProductId)
                            .to(Product::Table, Product::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_shipment_lines_pennylane_line_id")
                    .table(ShipmentLine::Table)
                    .col(ShipmentLine::PennylaneLineId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ShipmentLine::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum ShipmentLine {
    #[sea_orm(iden = "shipment_lines")]
    Table,
    Id,
    ShipmentId,
    PennylaneLineId,
    ProductId,
    Label,
    Quantity,
    CreatedAt,
    UpdatedAt,
}
