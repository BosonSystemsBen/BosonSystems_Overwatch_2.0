use sea_orm_migration::prelude::*;

use crate::m20240101_000002_create_serial_numbers::SerialNumber;
use crate::m20240101_000005_create_shipment_lines::ShipmentLine;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(SerialNumber::Table)
                    .add_column(ColumnDef::new(SerialNumber::ShipmentLineId).uuid().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_serial_numbers_shipment_line")
                    .from(SerialNumber::Table, SerialNumber::ShipmentLineId)
                    .to(ShipmentLine::Table, ShipmentLine::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(SerialNumber::Table)
                    .drop_column(SerialNumber::ShipmentLineId)
                    .to_owned(),
            )
            .await
    }
}
