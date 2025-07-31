use crate::m20250222_173535_create_file_tags::Folders;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(
                Table::alter()
                    .table(Folders::Table)
                    .drop_column(Alias::new("parent_folder_id"))
                    .to_owned(),
            )
            .await
            .expect("Failed to alter Column ParentFolderId");

        manager
            .alter_table(
                Table::alter()
                    .table(Folders::Table)
                    .add_column(
                        ColumnDef::new(Alias::new("parent_folder_id"))
                            .integer()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
            .expect("Failed to alter Column ParentFolderId");
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .alter_table(
                Table::alter()
                    .table(Folders::Table)
                    .drop_column(Alias::new("parent_folder_id"))
                    .to_owned(),
            )
            .await
            .expect("Failed to alter Column ParentFolderId");

        manager
            .alter_table(
                Table::alter()
                    .table(Folders::Table)
                    .add_column(
                        ColumnDef::new(Alias::new("parent_folder_id"))
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
            .expect("Failed to alter Column ParentFolderId");
        Ok(())
    }
}
