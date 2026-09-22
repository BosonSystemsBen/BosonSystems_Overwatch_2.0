use sea_orm_migration::prelude::*;

use crate::m20240101_000004_create_shipments::Shipment;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SendcloudOrder::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SendcloudOrder::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SendcloudOrder::ShipmentId).uuid().not_null())
                    .col(ColumnDef::new(SendcloudOrder::SendcloudOrderId).big_integer().not_null())
                    .col(ColumnDef::new(SendcloudOrder::OrderNumber).string().not_null())
                    .col(
                        ColumnDef::new(SendcloudOrder::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(SendcloudOrder::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sendcloud_orders_shipment")
                            .from(SendcloudOrder::Table, SendcloudOrder::ShipmentId)
                            .to(Shipment::Table, Shipment::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_sendcloud_orders_shipment_id")
                    .table(SendcloudOrder::Table)
                    .col(SendcloudOrder::ShipmentId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SendcloudOrder::Table).to_owned())
            .await
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(DeriveIden)]
pub enum SendcloudOrder {
    #[sea_orm(iden = "sendcloud_orders")]
    Table,
    Id,
    ShipmentId,
    SendcloudOrderId,
    OrderNumber,
    CreatedAt,
    UpdatedAt,
}
