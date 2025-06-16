use async_trait::async_trait;

use crate::core::{parser::Request, response::Response};

use super::{Middleware, Next};

use chrono::NaiveDate;
use flate2::{write::GzEncoder, Compression};
use std::{fs::{self, File}, io::{BufReader, BufWriter}, path::Path};

pub struct Logger;
#[async_trait]
impl Middleware for Logger {
    async fn handle(&self, req: Request, next: Next) -> Response {
        tracing::info!(
            method  = ?req.method,
            path    = %req.path,
            query   = ?req.query,
            "request_received"
        );

        next(req).await
    }
}

impl Logger {
    pub fn new() -> Self {
        std::thread::spawn(|| {
            loop {
                // Runs once per 24h
                std::thread::sleep(std::time::Duration::from_secs(60 * 60 * 24));
                compress_old_logs();
            }
        });

        Logger
    }
}

fn compress_old_logs() {
    let log_path = Path::new("logs");

    if !log_path.exists() {
        eprintln!("Log directory not found: {:?}", log_path);
        return;
    }

    let today = chrono::Local::now().date_naive();

    let entries = match fs::read_dir(log_path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Failed to read log directory: {:?}", e);
            return;
        }
    };

    for entry_result in entries {
        if let Ok(entry) = entry_result {
            let file_name = entry.file_name();
            let fn_str = file_name.to_string_lossy();
            let path = Path::new("logs").join(fn_str.as_ref());

            if fn_str.ends_with(".gz") {
                if let Some(date_part) = fn_str.strip_prefix("server.log.").and_then(|s| s.strip_suffix(".gz")) {
                    if let Ok(log_date) = NaiveDate::parse_from_str(date_part, "%Y-%m-%d") {
                        if today.signed_duration_since(log_date) > chrono::Duration::days(30) {
                            match std::fs::remove_file(&path) {
                                Ok(_) => println!("Deleted old log: {:?}", path),
                                Err(_) => println!("Could not delete old log: {:?}", path),
                            }
                        }
                    }
                }

                continue;
            }

            if let Some(date_part) = fn_str.strip_prefix("server.log.") {
                if let Ok(log_date) = chrono::NaiveDate::parse_from_str(date_part, "%Y-%m-%d") {
                    if log_date == today {
                        continue;
                    }
                }
            }


            match compress_file(&path) {
                Ok(_) => {
                    println!("Compressed: {:?}", path);
                },
                Err(_) => {
                    eprintln!("Error compressing file: {:?}", path);
                }
            }
        }
    }
}

fn compress_file(input_path: &Path) -> std::io::Result<()> {
    let output_path = input_path.with_file_name(format!(
        "{}.gz",
        input_path.file_name().unwrap().to_string_lossy()
    ));

    let input_file = File::open(input_path)?;
    let output_file = File::create(&output_path)?;

    let mut reader = BufReader::new(input_file);
    let writer = BufWriter::new(output_file);

    let mut encoder = GzEncoder::new(writer, Compression::default());

    std::io::copy(&mut reader, &mut encoder)?;

    encoder.finish()?;

    std::fs::remove_file(input_path)?;

    Ok(())
}
