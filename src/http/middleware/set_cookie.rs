use async_trait::async_trait;
use uuid::Uuid;

use crate::{core::{parser::Request, response::Response}, http::cookies::Cookie};

use super::{Middleware, Next};


pub struct SetCookie;
#[async_trait]
impl Middleware for SetCookie {
    async fn handle(&self, req: Request, next: Next) -> Response {
        let cookie = Cookie::new("session_id".to_string(), Uuid::new_v4().to_string());
        let res = next(req).await.header("Set-Cookie", cookie.serialize().as_str());
        res
    }
}
