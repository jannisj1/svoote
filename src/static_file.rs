use crate::app_error::AppError;
use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use std::sync::OnceLock;

pub static BUNDLES: OnceLock<Vec<Bundle>> = OnceLock::new();

pub struct Bundle {
    name: String,
    mime_type: String,
    path: String,
    content: String,
}

pub fn init() {
    let mut bundles = Vec::with_capacity(3);

    bundles.push(create_bundle(
        "bundle.css",
        "text/css",
        &["./src/static/output.css"],
    ));

    bundles.push(create_bundle(
        "app.js",
        "text/javascript",
        &[
            "./src/static/reconnecting-websocket.min.js",
            "./src/static/custom.js",
            //"./src/static/alpine_3_14_3_collapse.min.js",
            "./src/static/alpine_3_14_3.min.js",
        ],
    ));

    bundles.push(create_bundle(
        "tailwind_4_dev.js",
        "text/javascript",
        &["./src/static/tailwind_4_dev.js"],
    ));

    bundles.push(create_bundle(
        "qrcode.js",
        "text/javascript",
        &["./src/static/qrcode.min.js"],
    ));

    let _ = BUNDLES.set(bundles);
}

pub async fn http_get_static_file(Path(file_name): Path<String>) -> Result<Response, AppError> {
    for bundle in BUNDLES.get().unwrap().iter() {
        if file_name.contains(&bundle.name) {
            use axum::http::header;
            let mut headers = header::HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, bundle.mime_type.parse().unwrap());
            headers.insert(header::CACHE_CONTROL, "max-age=604800".parse().unwrap());

            return Ok((headers, bundle.content.clone()).into_response());
        }
    }

    return Err(AppError::NotFound);
}

fn create_bundle(name: &str, mime_type: &str, files: &[&str]) -> Bundle {
    info!("Bundling {name} ({mime_type}) from {:?}", files);

    let mut content = String::new();

    for file in files {
        match std::fs::read_to_string(&file) {
            Ok(file_content) => {
                content.push_str(&file_content);
                content.push_str("\n");
            }
            Err(e) => {
                error!("Error loading file {} ({})", &file, e);
            }
        }
    }

    let hash = md5::compute(content.as_bytes());

    let mut path = format!("/static/{:x}", hash);
    path.push_str(".");
    path.push_str(&name);

    Bundle {
        name: name.to_string(),
        mime_type: mime_type.to_string(),
        path,
        content,
    }
}

pub fn get_path(bundle_name: &str) -> String {
    for bundle in BUNDLES.get().unwrap().iter() {
        if bundle.name == bundle_name {
            return bundle.path.clone();
        }
    }

    panic!("static_files::get_path() called with invalid bundle_name ({bundle_name})");
}
