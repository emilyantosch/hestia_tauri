use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Folders {
    Table,
    Id,
    ContentHash,
    IdentityHash,
    StructureHash,
    FileSystemId,
    ParentFolderId,
    Name,
    Path,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum FileSystemIdentifier {
    Table,
    Id,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create Folders table
        manager
            .create_table(
                Table::create()
                    .table(Folders::Table)
                    .if_not_exists()
                    .col(pk_auto(Folders::Id))
                    .col(string(Folders::ContentHash))
                    .col(string(Folders::IdentityHash))
                    .col(string(Folders::StructureHash))
                    .col(integer(Folders::FileSystemId))
                    .col(integer_null(Folders::ParentFolderId))
                    .col(string(Folders::Name))
                    .col(string(Folders::Path))
                    .col(date_time(Folders::CreatedAt))
                    .col(date_time(Folders::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_folders_file_system_identifier")
                            .from(Folders::Table, Folders::FileSystemId)
                            .to(FileSystemIdentifier::Table, FileSystemIdentifier::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_folders_parent_folder")
                            .from(Folders::Table, Folders::ParentFolderId)
                            .to(Folders::Table, Folders::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index for Folders
        manager
            .create_index(
                Index::create()
                    .table(Folders::Table)
                    .if_not_exists()
                    .col(Folders::Id)
                    .col(Folders::FileSystemId)
                    .col(Folders::ParentFolderId)
                    .name("idx_folders_id_filesystem_parent")
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop index first
        manager
            .drop_index(
                Index::drop()
                    .name("idx_folders_id_filesystem_parent")
                    .to_owned(),
            )
            .await?;

        // Drop Folders table (foreign keys will be dropped automatically)
        manager
            .drop_table(Table::drop().table(Folders::Table).to_owned())
            .await?;

        Ok(())
    }
}
