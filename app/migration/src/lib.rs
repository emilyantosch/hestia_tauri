pub use sea_orm_migration::prelude::*;

mod m20250222_173535_create_file_tags;
mod m20250831_181914_icon_color;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250222_173535_create_file_tags::Migration),
            Box::new(m20250831_181914_icon_color::Migration),
        ]
    }
}
