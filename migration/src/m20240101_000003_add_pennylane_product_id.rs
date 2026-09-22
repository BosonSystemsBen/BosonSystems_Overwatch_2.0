use sea_orm_migration::prelude::*;

use crate::m20240101_000001_create_products::Product;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Product::Table)
                    .add_column(ColumnDef::new(Product::PennylaneProductId).big_integer().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_products_pennylane_product_id")
                    .table(Product::Table)
                    .col(Product::PennylaneProductId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Product::Table)
                    .drop_column(Product::PennylaneProductId)
                    .to_owned(),
            )
            .await
    }
}
