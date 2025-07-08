use chrono::{DateTime, Duration, Utc};

pub struct Cookie {
    name: String,
    value: String,
    path: String,
    expires: DateTime<Utc>,
    http_only: bool,
    secure: bool,
}

impl Cookie {
    pub fn new(
        name: String,
        value: String,
        ) -> Cookie {
        Cookie {
            name,
            value,
            path: String::from("/"),
            expires: Utc::now() + Duration::days(1),
            http_only: true,
            secure: true,
        }
    }

    pub fn serialize(&self) -> String {
        let mut result = String::from(format!("{}={}; Path={}; Expires={};", self.name, self.value, self.path, self.expires));
        if self.http_only { result.push_str(" HttpOnly;"); }
        if self.secure { result.push_str(" Secure;"); }

        result
    }
}

#[cfg(test)]
#[path ="tests/cookies.rs"]
mod cookies_tests;
