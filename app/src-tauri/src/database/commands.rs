use tauri::ipc;

use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction,
    EntityTrait, IntoActiveModel, QueryFilter, Set, TransactionTrait,
};

use entity::{files, prelude::*};

use crate::errors::{AppError, DbError, DbErrorKind};

#[derive(serde::Serialize)]
struct FolderContentResponse {
    name: String,
    contents: Vec<String>,
}

#[tauri::command]
pub async fn get_folder_structure() -> Result<ipc::Response, AppError> {
    let files = Files::find().all().await.map_err(|e| {
        DbError::with_source(
            DbErrorKind::QueryError,
            "Could not get file information from DB".to_string(),
            e,
        )
    })?;

    let folder_content_response: Vec<FolderContentResponse> = files
        .into_iter()
        .map(|v| FolderContentResponse { name: v.name })
        .collect();
    Ok(tauri::ipc::Response::new(folder_content_response))
}
