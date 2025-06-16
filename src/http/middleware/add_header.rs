use async_trait::async_trait;

use crate::core::{parser::Request, response::Response};

use super::{Middleware, Next};

pub struct AddHeader;
#[async_trait]
impl Middleware for AddHeader {
    async fn handle(&self, req: Request, next: Next) -> Response {
        let res = next(req).await.header("X-Example", "It works! :D");
        res
    }
}
