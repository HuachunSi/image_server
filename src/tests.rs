use axum::body::Bytes;
use axum_test::{
    TestServer,
    multipart::{MultipartForm, Part},
};
use reqwest::header;
use std::fs;

#[tokio::test]
async fn test_upload_and_download() {
    let app = crate::create_app();
    let server = TestServer::new(app).unwrap();

    // 准备测试图片
    let file_name = "test_image.png";
    let image_data = fs::read(file_name).expect("Failed to read test image");
    let multipart =
        MultipartForm::new().add_part("file", Part::bytes(image_data.clone()).file_name(file_name));
    // 测试上传
    let response = server
        .post("/upload")
        .add_header(
            header::CONTENT_TYPE,
            "multipart/form-data; boundary=X-BOUNDARY",
        )
        .multipart(multipart)
        .await;

    response.assert_status_ok();

    // 测试下载
    let id: String = response.json();
    let response = server.get(&format!("/download/{}", id)).await;
    response.assert_status_ok();
    assert_eq!(response.as_bytes(), &Bytes::copy_from_slice(&image_data));
}

#[tokio::test]
async fn test_upload_no_file() {
    let app = crate::create_app();
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/upload")
        .add_header(
            header::CONTENT_TYPE,
            "multipart/form-data; boundary=X-BOUNDARY",
        )
        .bytes(Bytes::new())
        .await;
    response.assert_status_internal_server_error();
}

#[tokio::test]
async fn test_download_not_found() {
    let app = crate::create_app();
    let server = TestServer::new(app).unwrap();

    let response = server.get("/download/nonexistent-id").await;

    response.assert_status_not_found();
}
