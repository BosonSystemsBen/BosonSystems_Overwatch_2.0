use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Product::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Product::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Product::Sku).string().not_null())
                    .col(ColumnDef::new(Product::Name).string().not_null())
                    .col(
                        ColumnDef::new(Product::RequiresSerial)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Product::DetectionPattern).string().null())
                    .col(
                        ColumnDef::new(Product::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Product::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_products_sku")
                    .table(Product::Table)
                    .col(Product::Sku)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Product::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Product {
    #[sea_orm(iden = "products")]
    Table,
    Id,
    Sku,
    Name,
    RequiresSerial,
    DetectionPattern,
    CreatedAt,
    UpdatedAt,
}
