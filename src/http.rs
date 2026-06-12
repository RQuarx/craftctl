use std::time::Duration;

use reqwest::{Client, Method, RequestBuilder};

#[derive(Debug)]
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
        self.client.request(method, url)
    }

    pub fn get(&self, url: &str) -> RequestBuilder {
        self.request(Method::GET, url)
    }

    pub fn post(&self, url: &str) -> RequestBuilder {
        self.request(Method::POST, url)
    }

    pub async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
    ) -> Result<T, reqwest::Error> {
        self.client.get(url).send().await?.json::<T>().await
    }
}
