use std::time::SystemTime;

use async_trait::async_trait;

use crate::core::{parser::Request, response::Response};

use super::{Middleware, Next};

pub struct Timer;
#[async_trait]
impl Middleware for Timer {
    async fn handle(&self, req: Request, next: Next) -> Response {
        let now = SystemTime::now();

        let res = next(req).await;

        match now.elapsed() {
            Ok(duration) => {
                res.header("X-Duration", duration.as_nanos().to_string().as_str())
            }
            Err(_) => {
                res
            }
        }
    }
}
