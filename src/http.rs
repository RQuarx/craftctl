use std::time::Duration;

use reqwest::{Client, Method, RequestBuilder, Response};

pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> Result<HttpClient, reqwest::Error> {
        Ok(HttpClient {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .user_agent(format!(
                    "{}/{}",
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_VERSION")
                ))
                .build()?,
        })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn request(&self, method: Method, url: &str) -> RequestBuilder {
        self.request(method, url)
    }

    pub async fn get(&self, url: &str) -> Result<Response, reqwest::Error> {
        self.client.get(url).send().await
    }

    pub async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
    ) -> Result<T, reqwest::Error> {
        self.client.get(url).send().await?.json::<T>().await
    }
}
