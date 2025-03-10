use axum::{
    Json, Router,
    body::Body,
    extract::{Multipart, Path},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use http::StatusCode;
use mime_guess::from_path;
use sha2::{Digest, Sha256};
use std::fs;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

const UPLOAD_PATH: &str = "/tmp/uploads";
const MAX_FILE_SIZE: usize = 1_000_000; // 1MB in bytes

pub fn create_app() -> Router {
    let cors = CorsLayer::new().allow_origin(Any);

    Router::new()
        .route("/upload", post(upload))
        .route("/download/{id}", get(download))
        .layer(cors)
}

#[tokio::main]
async fn main() {
    let app = create_app();

    let addr = SocketAddr::from(([0, 0, 0, 0], 7870));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Listening on {}", addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn upload(mut multipart: Multipart) -> Result<Json<String>, impl IntoResponse> {
    match multipart.next_field().await {
        Ok(Some(field)) => {
            let _name = field.name().unwrap().to_string();
            let file_name = field.file_name().unwrap().to_string();
            let data = field.bytes().await.unwrap();

            // Check file size
            if data.len() > MAX_FILE_SIZE {
                return Err(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from(format!(
                        "File size exceeds the maximum limit of {} bytes",
                        MAX_FILE_SIZE
                    )))
                    .unwrap());
            }

            // Calculate SHA-256 hash of the file content
            let mut hasher = Sha256::new();
            hasher.update(&data);
            let hash = hasher.finalize();
            let hash_hex = format!("{:x}", hash);

            let path = format!("{}/{}", UPLOAD_PATH, hash_hex);
            fs::create_dir_all(UPLOAD_PATH).map_err(|err| {
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(err.to_string()))
                    .unwrap()
            })?;

            // Check if file already exists
            if fs::metadata(&path).is_err() {
                fs::write(&path, data).map_err(|err| {
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from(err.to_string()))
                        .unwrap()
                })?;
            }

            // Store file name mapping
            let name_path = format!("{}/{}.name", UPLOAD_PATH, hash_hex);
            fs::write(&name_path, file_name).map_err(|err| {
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(err.to_string()))
                    .unwrap()
            })?;

            Ok(Json(hash_hex))
        }
        Ok(None) => Err(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("No file uploaded"))
            .unwrap()),
        Err(err) => Err(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(err.to_string()))
            .unwrap()),
    }
}

async fn download(Path(id): Path<String>) -> Result<impl IntoResponse, impl IntoResponse> {
    let path = format!("{}/{}", UPLOAD_PATH, id);
    let content = fs::read(&path);

    // Read original file name
    let name_path = format!("{}/{}.name", UPLOAD_PATH, id);
    let original_name = fs::read_to_string(&name_path).unwrap_or_else(|_| "unknown".to_string());

    let mime = from_path(&original_name).first_or_octet_stream();
    match content {
        Ok(data) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", mime.as_ref())
            .header(
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", original_name),
            )
            .body(Body::from(data))
            .unwrap()),
        Err(err) => Err(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from(err.to_string()))
            .unwrap()),
    }
}

#[cfg(test)]
mod tests;
