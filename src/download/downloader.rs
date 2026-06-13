use std::sync::Arc;

use crate::http::HttpClient;
use futures_util::StreamExt;
use tokio::{fs::File, io::AsyncWriteExt, sync::mpsc};

#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: Option<u64>,
}

pub struct DownloadManager {
    client: Arc<HttpClient>,
}

impl DownloadManager {
    pub fn create(client: Arc<HttpClient>) -> DownloadManager {
        DownloadManager { client }
    }

    pub fn download(&self, url: impl ToString) -> DownloadBuilder {
        DownloadBuilder {
            client: self.client.clone(),
            url: url.to_string(),
            path: String::default(),
        }
    }
}

pub struct DownloadBuilder {
    client: Arc<HttpClient>,
    url: String,
    path: String,
}

impl DownloadBuilder {
    pub fn destination(&mut self, path: impl ToString) -> &mut Self {
        self.path = path.to_string();
        self
    }

    pub async fn run(&self, tx: mpsc::Sender<DownloadProgress>) -> Result<(), reqwest::Error> {
        let response = self.client.get(&self.url).send().await?;

        let total = response.content_length();
        let mut downloaded = 0;

        let mut stream = response.bytes_stream();

        let mut file = File::create(&self.path).await.unwrap();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;

            file.write_all(&chunk).await.unwrap();

            downloaded += chunk.len() as u64;

            let _ = tx.send(DownloadProgress { downloaded, total }).await;
        }

        file.flush().await.unwrap();

        Ok(())
    }
}
