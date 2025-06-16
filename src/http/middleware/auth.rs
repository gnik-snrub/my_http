use async_trait::async_trait;

use crate::core::{parser::Request, response::{Response, StatusCode}};

use super::{Middleware, Next};

pub struct Auth;
#[async_trait]
impl Middleware for Auth {
    async fn handle(&self, req: Request, next: Next) -> Response {
        if req.headers.get("Authorization").is_none() {
            return Response::new().status(StatusCode::Unauthorized);
        }

        let res = next(req).await;

        res
    }
}

