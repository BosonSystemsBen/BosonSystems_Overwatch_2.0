use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Shipment::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Shipment::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Shipment::PennylaneInvoiceId).big_integer().not_null())
                    .col(ColumnDef::new(Shipment::InvoiceNumber).string().not_null())
                    .col(ColumnDef::new(Shipment::PennylaneCustomerId).big_integer().not_null())
                    .col(ColumnDef::new(Shipment::CustomerName).string().not_null())
                    .col(ColumnDef::new(Shipment::DeliveryAddress).string().not_null())
                    .col(ColumnDef::new(Shipment::DeliveryPostalCode).string().not_null())
                    .col(ColumnDef::new(Shipment::DeliveryCity).string().not_null())
                    .col(ColumnDef::new(Shipment::DeliveryCountry).string().not_null())
                    .col(
                        ColumnDef::new(Shipment::Status)
                            .string()
                            .not_null()
                            .default("pending"),
                    )
                    .col(
                        ColumnDef::new(Shipment::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Shipment::UpdatedAt)
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
                    .name("idx_shipments_pennylane_invoice_id")
                    .table(Shipment::Table)
                    .col(Shipment::PennylaneInvoiceId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Shipment::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Shipment {
    #[sea_orm(iden = "shipments")]
    Table,
    Id,
    PennylaneInvoiceId,
    InvoiceNumber,
    PennylaneCustomerId,
    CustomerName,
    DeliveryAddress,
    DeliveryPostalCode,
    DeliveryCity,
    DeliveryCountry,
    Status,
    CreatedAt,
    UpdatedAt,
}
