use std::path::PathBuf;

use crate::{parser::Request, response::{Response, StatusCode}};

pub async fn serve_file(req: &Request, res: Response) -> Response {
    let path_split: Vec<&str> = req.path.split('/').collect();
    let file_name = path_split[path_split.len() - 1];
    let rel_path = PathBuf::from("public").join(file_name);

    return match tokio::fs::read(&rel_path).await {
        Ok(bytes) => {
            res.status(StatusCode::Ok).text(&"")
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
