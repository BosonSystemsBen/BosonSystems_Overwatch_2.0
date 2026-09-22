use sea_orm_migration::prelude::*;

use crate::m20240101_000001_create_products::Product;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SerialNumber::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SerialNumber::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SerialNumber::ProductId).uuid().not_null())
                    .col(ColumnDef::new(SerialNumber::Value).string().not_null())
                    .col(
                        ColumnDef::new(SerialNumber::Status)
                            .string()
                            .not_null()
                            .default("in_stock"),
                    )
                    .col(ColumnDef::new(SerialNumber::AssignedTo).string().null())
                    .col(
                        ColumnDef::new(SerialNumber::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(SerialNumber::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_serial_numbers_product")
                            .from(SerialNumber::Table, SerialNumber::ProductId)
                            .to(Product::Table, Product::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_serial_numbers_value")
                    .table(SerialNumber::Table)
                    .col(SerialNumber::Value)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SerialNumber::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum SerialNumber {
    #[sea_orm(iden = "serial_numbers")]
    Table,
    Id,
    ProductId,
    Value,
    Status,
    AssignedTo,
    CreatedAt,
    UpdatedAt,
    ShipmentLineId,
}
