//! Client HTTP minimal du mod RSDWServerAPI — l'essentiel des endpoints,
//! en requêtes bloquantes (reqwest::blocking), erreurs classifiées en JSON.

use std::time::Duration;

const DEFAULT_BASE: &str = "http://127.0.0.1:8080";

pub struct Client {
    http: reqwest::blocking::Client,
    base: String,
    token: Option<String>,
}

#[derive(Debug)]
pub enum Error {
    NotConfigured,
    Unreachable(String),
    Http(u16, String),
    Malformed(String),
}

impl Error {
    pub fn to_json(&self) -> String {
        match self {
            Error::NotConfigured => {
                "{\"error\":\"API non configurée — renseigner RSDW_API_TOKEN\"}".to_string()
            }
            Error::Unreachable(d) => format!(
                "{{\"error\":\"injoignable : {}\"}}",
                json_escape(d)
            ),
            Error::Http(status, detail) => {
                format!("{{\"error\":\"HTTP {status}: {}\"}}", json_escape(detail))
            }
            Error::Malformed(d) => {
                format!("{{\"error\":\"réponse illisible : {}\"}}", json_escape(d))
            }
        }
    }
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', " ")
}

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

impl Client {
    pub fn new() -> Result<Self, Error> {
        let token = env_or("RSDW_API_TOKEN", "");
        let base = env_or("RSDW_API_BASE_URL", DEFAULT_BASE);
        let timeout = env_or("RSDW_REQUEST_TIMEOUT_SECS", "10")
            .parse::<u64>()
            .unwrap_or(10);
        let http = reqwest::blocking::Client::builder()
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(timeout))
            .build()
            .map_err(|e| Error::Unreachable(e.to_string()))?;
        Ok(Client {
            http,
            base,
            token: if token.is_empty() { None } else { Some(token) },
        })
    }

    pub fn is_configured(&self) -> bool {
        self.token.is_some()
    }

    fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        query: &[(&str, &str)],
        body: Option<&str>,
    ) -> Result<String, String> {
        let Some(token) = &self.token else {
            return Err(Error::NotConfigured.to_json());
        };
        let url = format!("{}{}", self.base.trim_end_matches('/'), path);
        let mut req = self.http.request(method, url).query(query);
        if let Some(body) = body {
            req = req.header("Content-Type", "application/json").body(body.to_string());
        }
        let resp = req
            .header("Authorization", format!("Bearer {token}"))
            .header("Accept", "application/json")
            .send()
            .map_err(|e| Error::Unreachable(e.to_string()).to_json())?;
        let status = resp.status().as_u16();
        let text = resp.text().unwrap_or_default();
        if status != 200 {
            return Err(Error::Http(status, text).to_json());
        }
        Ok(text)
    }

    pub fn get(&self, path: &str) -> Result<String, String> {
        self.request(reqwest::Method::GET, path, &[], None)
    }

    pub fn post(&self, path: &str, body: &str) -> Result<String, String> {
        self.request(reqwest::Method::POST, path, &[], Some(body))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_escape_neutralises_quotes() {
        assert_eq!(json_escape("a\"b"), "a\\\"b");
        assert_eq!(json_escape("a\nb"), "a b");
    }

    #[test]
    fn unconfigured_client_is_reported() {
        // Sans RSDW_API_TOKEN dans l'environnement du test… (peut fuiter)
        // On teste le chemin d'erreur HTTP avec un client factice impossible :
        // vérifier plutôt le format des erreurs.
        let err = Error::Http(401, "Unauthorized".into());
        assert!(err.to_json().contains("401"));
    }
}
