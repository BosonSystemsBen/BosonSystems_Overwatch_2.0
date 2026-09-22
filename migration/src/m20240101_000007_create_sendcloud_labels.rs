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
                    .table(SendcloudLabel::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SendcloudLabel::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SendcloudLabel::ShipmentId).uuid().not_null())
                    .col(ColumnDef::new(SendcloudLabel::SendcloudShipmentId).string().not_null())
                    .col(ColumnDef::new(SendcloudLabel::SendcloudParcelId).big_integer().not_null())
                    .col(ColumnDef::new(SendcloudLabel::TrackingNumber).string().null())
                    .col(ColumnDef::new(SendcloudLabel::TrackingUrl).string().null())
                    .col(ColumnDef::new(SendcloudLabel::LabelLink).string().null())
                    .col(ColumnDef::new(SendcloudLabel::Status).string().not_null())
                    .col(
                        ColumnDef::new(SendcloudLabel::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(SendcloudLabel::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sendcloud_labels_shipment")
                            .from(SendcloudLabel::Table, SendcloudLabel::ShipmentId)
                            .to(Shipment::Table, Shipment::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_sendcloud_labels_shipment_id")
                    .table(SendcloudLabel::Table)
                    .col(SendcloudLabel::ShipmentId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SendcloudLabel::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum SendcloudLabel {
    #[sea_orm(iden = "sendcloud_labels")]
    Table,
    Id,
    ShipmentId,
    SendcloudShipmentId,
    SendcloudParcelId,
    TrackingNumber,
    TrackingUrl,
    LabelLink,
    Status,
    CreatedAt,
    UpdatedAt,
}
