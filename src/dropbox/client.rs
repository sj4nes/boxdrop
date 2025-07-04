use crate::Result;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, warn};

/// Dropbox API v2 client for file operations
pub struct DropboxClient {
    client: Client,
    access_token: String,
    base_url: String,
}

/// Dropbox API error response
#[derive(Debug, Deserialize)]
struct DropboxError {
    error_summary: String,
    error: DropboxErrorDetail,
}

#[derive(Debug, Deserialize)]
struct DropboxErrorDetail {
    #[serde(rename = ".tag")]
    tag: String,
    reason: Option<DropboxErrorReason>,
}

#[derive(Debug, Deserialize)]
struct DropboxErrorReason {
    #[serde(rename = ".tag")]
    tag: String,
}

/// Dropbox file metadata
#[derive(Debug, Deserialize, Serialize)]
pub struct FileMetadata {
    pub name: String,
    pub path_lower: String,
    pub path_display: String,
    pub id: String,
    pub client_modified: Option<String>,
    pub server_modified: Option<String>,
    pub rev: String,
    pub size: u64,
    pub is_downloadable: bool,
    pub content_hash: Option<String>,
    #[serde(rename = ".tag")]
    pub tag: String,
}

/// Dropbox folder metadata
#[derive(Debug, Deserialize, Serialize)]
pub struct FolderMetadata {
    pub name: String,
    pub path_lower: String,
    pub path_display: String,
    pub id: String,
    #[serde(rename = ".tag")]
    pub tag: String,
}

/// Dropbox list folder response
#[derive(Debug, Deserialize)]
struct ListFolderResponse {
    entries: Vec<serde_json::Value>,
    cursor: String,
    has_more: bool,
}

impl DropboxClient {
    /// Create a new Dropbox client with the given access token
    pub fn new(access_token: &str) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(&format!("Bearer {}", access_token))
                .map_err(|e| anyhow::anyhow!("Invalid authorization header: {}", e))?
        );
        headers.insert("Content-Type", header::HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .default_headers(headers)
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            client,
            access_token: access_token.to_string(),
            base_url: "https://api.dropboxapi.com/2".to_string(),
        })
    }

    /// Test the connection and token validity
    pub async fn test_connection(&self) -> Result<()> {
        let response = self.client
            .post(&format!("{}/users/get_current_account", self.base_url))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to test connection: {}", e))?;

        if !response.status().is_success() {
            let error: DropboxError = response.json().await
                .map_err(|e| anyhow::anyhow!("Failed to parse error response: {}", e))?;
            return Err(anyhow::anyhow!("Dropbox API error: {}", error.error_summary));
        }

        debug!("Dropbox connection test successful");
        Ok(())
    }

    /// Get file metadata
    pub async fn get_metadata(&self, path: &str) -> Result<FileMetadata> {
        let payload = serde_json::json!({
            "path": path,
            "include_media_info": false,
            "include_deleted": false,
            "include_has_explicit_shared_members": false
        });

        let response = self.client
            .post(&format!("{}/files/get_metadata", self.base_url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get metadata for {}: {}", path, e))?;

        if !response.status().is_success() {
            let error: DropboxError = response.json().await
                .map_err(|e| anyhow::anyhow!("Failed to parse error response: {}", e))?;
            return Err(anyhow::anyhow!("Failed to get metadata: {}", error.error_summary));
        }

        let metadata: FileMetadata = response.json().await
            .map_err(|e| anyhow::anyhow!("Failed to parse metadata response: {}", e))?;

        debug!("Retrieved metadata for {}: size={}, modified={:?}", 
               path, metadata.size, metadata.server_modified);
        Ok(metadata)
    }

    /// List folder contents
    pub async fn list_folder(&self, path: &str) -> Result<Vec<FileMetadata>> {
        let mut all_entries = Vec::new();
        let mut cursor = None;

        loop {
            let payload = if let Some(ref cursor_val) = cursor {
                serde_json::json!({
                    "cursor": cursor_val
                })
            } else {
                serde_json::json!({
                    "path": path,
                    "recursive": false,
                    "include_media_info": false,
                    "include_deleted": false,
                    "include_has_explicit_shared_members": false,
                    "include_mounted_folders": true,
                    "limit": 1000
                })
            };

            let endpoint = if cursor.is_some() {
                "/files/list_folder/continue"
            } else {
                "/files/list_folder"
            };

            let response = self.client
                .post(&format!("{}{}", self.base_url, endpoint))
                .json(&payload)
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to list folder {}: {}", path, e))?;

            if !response.status().is_success() {
                let error: DropboxError = response.json().await
                    .map_err(|e| anyhow::anyhow!("Failed to parse error response: {}", e))?;
                return Err(anyhow::anyhow!("Failed to list folder: {}", error.error_summary));
            }

            let list_response: ListFolderResponse = response.json().await
                .map_err(|e| anyhow::anyhow!("Failed to parse list response: {}", e))?;

            // Parse entries into FileMetadata or FolderMetadata
            for entry in list_response.entries {
                if let Ok(file_metadata) = serde_json::from_value::<FileMetadata>(entry.clone()) {
                    all_entries.push(file_metadata);
                } else if let Ok(_folder_metadata) = serde_json::from_value::<FolderMetadata>(entry.clone()) {
                    // Skip folders for now, we'll handle them separately
                    continue;
                } else {
                    warn!("Failed to parse folder entry: {:?}", entry);
                }
            }

            if !list_response.has_more {
                break;
            }

            cursor = Some(list_response.cursor);
        }

        debug!("Listed {} files in folder {}", all_entries.len(), path);
        Ok(all_entries)
    }

    /// Get a temporary link for downloading a file
    pub async fn get_temporary_link(&self, path: &str) -> Result<String> {
        let payload = serde_json::json!({
            "path": path
        });

        #[derive(Deserialize)]
        struct TemporaryLinkResponse {
            link: String,
        }

        let response = self.client
            .post(&format!("{}/files/get_temporary_link", self.base_url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get temporary link for {}: {}", path, e))?;

        if !response.status().is_success() {
            let error: DropboxError = response.json().await
                .map_err(|e| anyhow::anyhow!("Failed to parse error response: {}", e))?;
            return Err(anyhow::anyhow!("Failed to get temporary link: {}", error.error_summary));
        }

        let temp_link: TemporaryLinkResponse = response.json().await
            .map_err(|e| anyhow::anyhow!("Failed to parse temporary link response: {}", e))?;

        debug!("Got temporary link for {}", path);
        Ok(temp_link.link)
    }

    /// Upload a file to Dropbox
    pub async fn upload_file(&self, path: &str, content: &[u8]) -> Result<FileMetadata> {
        let payload = serde_json::json!({
            "path": path,
            "mode": "overwrite",
            "autorename": false,
            "mute": false,
            "strict_conflict": false
        });

        let response = self.client
            .post(&format!("{}/files/upload", self.base_url))
            .header("Dropbox-API-Arg", serde_json::to_string(&payload)?)
            .header("Content-Type", "application/octet-stream")
            .body(content.to_vec())
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to upload file {}: {}", path, e))?;

        if !response.status().is_success() {
            let error: DropboxError = response.json().await
                .map_err(|e| anyhow::anyhow!("Failed to parse error response: {}", e))?;
            return Err(anyhow::anyhow!("Failed to upload file: {}", error.error_summary));
        }

        let metadata: FileMetadata = response.json().await
            .map_err(|e| anyhow::anyhow!("Failed to parse upload response: {}", e))?;

        debug!("Uploaded file {}: size={}", path, metadata.size);
        Ok(metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        // This test requires a valid token, so we'll just test the structure
        let token = "test_token";
        let client = DropboxClient::new(token).unwrap();
        assert_eq!(client.access_token, token);
        assert_eq!(client.base_url, "https://api.dropboxapi.com/2");
    }
} 