use std::{collections::HashMap, sync::{Arc, RwLock}};

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::{core::{parser::Request, response::Response}};

use super::{Middleware, Next};

#[derive(Debug)]
pub struct SessionData {
    data: HashMap<String, String>,
    last_accessed: DateTime<Utc>
}

#[derive(Clone, Debug)]
pub struct SessionTracker {
    sessions: Arc<RwLock<HashMap<String, SessionData>>>
}
#[async_trait]
impl Middleware for SessionTracker {
    async fn handle(&self, req: Request, next: Next) -> Response {
        let new_id = Uuid::new_v4().to_string();
        let cookies_clone = req.cookies.clone();
        if let Some(cookies) = cookies_clone {
            if let Ok(mut sessions) = self.sessions.write() {
                if let Some(session) = cookies.get("session_id") {
                    if let Some(data) = sessions.get_mut(session) {
                        data.last_accessed = Utc::now();
                    }                 
                } else {
                    let mut new_session = SessionData { data: HashMap::new(), last_accessed: Utc::now() };
                    new_session.data.insert("init".to_string(), "true".to_string());
                    sessions.insert(new_id.clone(), new_session);
                }
            }
        }
        let res = next(req).await.header("Set-Cookie", format!("session_id={}", new_id).as_str());
        res
    }
}

impl SessionTracker {
    pub fn new() -> SessionTracker {
        let tracker = SessionTracker { sessions: Arc::new(RwLock::new(HashMap::new())) };
        let session_threads = tracker.sessions.clone();
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(60 * 60)); // 1 hour
                if let Ok(mut sessions) = session_threads.write() {
                    let time_limit = Duration::minutes(15);
                    let now = Utc::now();
                    sessions.retain(|_, d| now - d.last_accessed > time_limit);
                }
            }
        });
        tracker
    }
}
