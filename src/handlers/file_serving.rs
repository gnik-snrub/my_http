use std::{collections::HashMap, fs::{self}, path::PathBuf};
use serde_json::json;

use crate::core::{parser::Request, response::{Response, StatusCode}};

pub async fn serve_file(req: &Request, res: Response) -> Response {
    let path_split: Vec<&str> = req.path.split('/').collect();
    let file_name = path_split[path_split.len() - 1];
    let rel_path = PathBuf::from("public").join(file_name);

    let metadata = match tokio::fs::metadata(&rel_path).await {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return res.status(StatusCode::NotFound).text(&"404 Not Found"),
        Err(_) => return res.status(StatusCode::InternalError).text(&"500 Internal Error")
    };

    if metadata.is_file() {
        return match tokio::fs::read(&rel_path).await {
            Ok(bytes) => {
                let mime_type = get_mime_type(file_name);
                let mut file_res = res.status(StatusCode::Ok).text(&"")
                    .header("Content-Type", mime_type)
                    .header("Content-Length", bytes.len().to_string().as_str());
                file_res.body = bytes;
                file_res
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!("{:?}", e);
                res.status(StatusCode::NotFound).text(&"404 Not Found")
            }
            Err(_) => {
                res.status(StatusCode::InternalError).text(&"500 Internal Error")
            }
        }
    }

    if metadata.is_dir() {
        let mut vec = vec![];
        if let Ok(entry) = fs::read_dir(&rel_path) {
            entry.for_each(|x| 
                if let Ok(item) = x {
                    let name = item.file_name().to_string_lossy().into_owned();
                    vec.push(name);
                }
            );
        }
        return res.status(StatusCode::Ok).json(&json!(&vec));
    }
    res.status(StatusCode::InternalError).text(&"500 Internal Error")

}

fn get_mime_type(file_name: &str) -> &str {
    let mut mime_types = HashMap::new();
    mime_types.insert("html", "text/html");
    mime_types.insert("css", "text/css");
    mime_types.insert("js", "application/javascript");
    mime_types.insert("png", "image/png");
    mime_types.insert("txt", "text/plain");

    let file_extension = file_name.rsplit_once('.').map(|(_, ext)| ext);
    return match file_extension {
        Some(ext) => {
            match mime_types.get(ext) {
                Some(mime) => {
                    *mime
                }
                None => {
                    &"application/octet-stream"
                }
            }
        }
        None => {
            &""
        }
    }
}

#[cfg(test)]
#[path ="tests/file_serving.rs"]
mod file_serving_tests;
