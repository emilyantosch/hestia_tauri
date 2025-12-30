use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Color {
    Table,
    _ID,
    Name,
    Hex,
}

#[derive(DeriveIden)]
enum Icon {
    Table,
    _ID,
    Name,
    Type,
    Content,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create the color table
        manager
            .create_table(
                Table::create()
                    .table(Color::Table)
                    .if_not_exists()
                    .col(pk_auto(Color::_ID))
                    .col(string(Color::Name))
                    .col(string(Color::Hex))
                    .to_owned(),
            )
            .await
            .expect("Failed to create Color table");

        // Create the icon table
        manager
            .create_table(
                Table::create()
                    .table(Icon::Table)
                    .if_not_exists()
                    .col(pk_auto(Icon::_ID))
                    .col(string(Icon::Name))
                    .col(string(Icon::Type))
                    .col(string(Icon::Content))
                    .to_owned(),
            )
            .await
            .expect("Failed to create Icon table");

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Color::Table).to_owned())
            .await
            .expect("Failed to drop Color table");

        manager
            .drop_table(Table::drop().table(Icon::Table).to_owned())
            .await
            .expect("Failed to drop Color table");

        Ok(())
    }
}
