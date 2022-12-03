use crate::domain::interface::*;
use crate::error::*;
use async_trait::async_trait;

#[derive(Debug)]
pub enum HttpClientError {
    #[allow(dead_code)]
    InvalidBody,
    HttpError,
}

impl IServiceError for HttpClientError {
    fn error_type(&self) -> String {
        use HttpClientError::*;

        match self {
            InvalidBody => "invalid_body",
            HttpError => "http_error",
        }
        .to_string()
    }

    fn status_code(&self) -> http::StatusCode {
        use HttpClientError::*;

        match self {
            InvalidBody => http::StatusCode::INTERNAL_SERVER_ERROR,
            HttpError => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<reqwest::Error> for ServiceError {
    fn from(err: reqwest::Error) -> ServiceError {
        ServiceError::new(HttpClientError::HttpError, err)
    }
}

#[derive(Clone)]
pub struct HttpClient {
    client: reqwest::Client,
}

impl HttpClient {
    pub fn new() -> HttpClient {
        HttpClient {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl IHttpClient for HttpClient {
    async fn get(
        &self,
        url: &str,
        header: Option<reqwest::header::HeaderMap>,
    ) -> Result<reqwest::Response> {
        let mut req = self.client.get(url);
        if let Some(h) = header {
            req = req.headers(h);
        }
        let resp = req.send().await?;

        Ok(resp)
    }

    async fn post(
        &self,
        url: &str,
        header: Option<reqwest::header::HeaderMap>,
        body: std::option::Option<std::string::String>,
    ) -> Result<reqwest::Response> {
        let mut req = self.client.post(url).body(body.unwrap_or_default());
        if let Some(h) = header {
            req = req.headers(h);
        }

        let resp = req.send().await?;

        Ok(resp)
    }

    async fn put(
        &self,
        url: &str,
        header: Option<reqwest::header::HeaderMap>,
        body: String,
    ) -> Result<reqwest::Response> {
        let mut req = self.client.put(url).body(body);
        if let Some(h) = header {
            req = req.headers(h);
        }

        let resp = req.send().await?;

        Ok(resp)
    }

    async fn delete(
        &self,
        url: &str,
        header: Option<reqwest::header::HeaderMap>,
    ) -> Result<reqwest::Response> {
        let mut req = self.client.delete(url);
        if let Some(h) = header {
            req = req.headers(h);
        }

        let resp = req.send().await?;

        Ok(resp)
    }
}
