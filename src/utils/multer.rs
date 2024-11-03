use axum::{extract::{Multipart, State}, response::IntoResponse};
use std::{fs::File, io::Write, path::Path};

use crate::AppState;

const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024;

const ALLOWED_EXTENSIONS: [&str; 3] = ["pdf", "docx", "txt"];

pub async fn upload_file_handler(
    State(_state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let file_name = field.file_name().unwrap_or("file").to_string();

        let extension = Path::new(&file_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !ALLOWED_EXTENSIONS.contains(&extension.as_str()) {
            return "File type not allowed".to_string();
        }

        let path = Path::new("uploads").join(&file_name);

        let mut file = File::create(&path).expect("Failed to create file");
        let mut total_size: u64 = 0;

        while let Some(chunk) = field.chunk().await.unwrap() {
            total_size += chunk.len() as u64;

            if total_size > MAX_FILE_SIZE {
                return "File size exceeds the limit".to_string();
            }

            file.write_all(&chunk).expect("Failed to write to file");
        }

        println!("Uploaded file saved to {:?}", path);
    }

    "File upload successful".to_string()
}