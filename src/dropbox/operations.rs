use crate::Result;
use super::client::DropboxClient;
use std::path::Path;
use std::fs;
use tracing::{info, warn, debug};
use chrono::{DateTime, Utc};
use std::collections::VecDeque;

/// File operations trait for Dropbox
pub trait FileOperations {
    /// Download a file from Dropbox
    async fn download_file(&self, path: &str) -> Result<Vec<u8>>;
    
    /// Upload a file to Dropbox
    async fn upload_file(&self, path: &str, content: &[u8]) -> Result<()>;
}

/// Conflict detection result
#[derive(Debug, Clone)]
pub enum ConflictResult {
    NoConflict,
    Conflict {
        local_size: u64,
        remote_size: u64,
        local_modified: Option<DateTime<Utc>>,
        remote_modified: Option<DateTime<Utc>>,
        local_hash: Option<String>,
        remote_hash: Option<String>,
    },
    Error(String),
}

#[derive(Debug, Clone)]
pub struct UploadOptions {
    pub overwrite: bool,
    pub create_backup: bool,
    pub autorename: bool,
    pub mute: bool,
}

impl Default for UploadOptions {
    fn default() -> Self {
        Self {
            overwrite: false,
            create_backup: true,
            autorename: false,
            mute: false,
        }
    }
}

/// Upload work item
#[derive(Debug, Clone)]
enum UploadTask {
    Upload {
        path: String,
        content: Vec<u8>,
        options: UploadOptions,
    },
    Backup {
        original_path: String,
        backup_path: String,
        content: Vec<u8>,
    },
}

impl FileOperations for DropboxClient {
    async fn download_file(&self, path: &str) -> Result<Vec<u8>> {
        let temp_link = self.get_temporary_link(path).await?;
        let response = reqwest::get(&temp_link).await
            .map_err(|e| anyhow::anyhow!("Failed to download file {}: {}", path, e))?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to download file {}: HTTP {}", path, response.status()));
        }
        let content = response.bytes().await
            .map_err(|e| anyhow::anyhow!("Failed to read download response: {}", e))?;
        info!("Downloaded file {}: {} bytes", path, content.len());
        Ok(content.to_vec())
    }
    async fn upload_file(&self, path: &str, content: &[u8]) -> Result<()> {
        self.upload_file_with_options(path, content, &UploadOptions::default()).await
    }
}

impl DropboxClient {
    /// Upload a file with conflict detection and backup using a work queue
    pub async fn upload_file_with_options(&self, path: &str, content: &[u8], options: &UploadOptions) -> Result<()> {
        let mut queue = VecDeque::new();
        queue.push_back(UploadTask::Upload {
            path: path.to_string(),
            content: content.to_vec(),
            options: options.clone(),
        });

        while let Some(task) = queue.pop_front() {
            match task {
                UploadTask::Upload { path, content, options } => {
                    // Conflict detection
                    if !options.overwrite && options.create_backup {
                        match self.detect_conflict(&path, &content).await? {
                            ConflictResult::Conflict { local_size, remote_size, local_modified, remote_modified, .. } => {
                                warn!("Conflict detected for {}: local={} bytes ({:?}), remote={} bytes ({:?})", 
                                    path, local_size, local_modified, remote_size, remote_modified);
                                if options.create_backup {
                                    // Enqueue backup task
                                    let backup_path = format!("{}.backup.{}", path, chrono::Utc::now().timestamp());
                                    let remote_content = self.download_file(&path).await?;
                                    queue.push_back(UploadTask::Backup {
                                        original_path: path.clone(),
                                        backup_path: backup_path.clone(),
                                        content: remote_content,
                                    });
                                }
                                if !options.autorename {
                                    return Err(anyhow::anyhow!("Conflict detected for {} and autorename is disabled", path));
                                }
                                // If autorename, continue to upload
                            }
                            ConflictResult::Error(e) => {
                                return Err(anyhow::anyhow!("Error during conflict detection: {}", e));
                            }
                            ConflictResult::NoConflict => {
                                debug!("No conflict detected for {}", path);
                            }
                        }
                    }
                    // Prepare upload payload
                    let payload = serde_json::json!({
                        "path": path,
                        "mode": if options.overwrite { "overwrite" } else { "add" },
                        "autorename": options.autorename,
                        "mute": options.mute,
                        "strict_conflict": false
                    });
                    let response = self.client
                        .post(&format!("{}/files/upload", self.base_url))
                        .header("Dropbox-API-Arg", serde_json::to_string(&payload)?)
                        .header("Content-Type", "application/octet-stream")
                        .body(content.clone())
                        .send()
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to upload file {}: {}", path, e))?;
                    if !response.status().is_success() {
                        let status = response.status();
                        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                        return Err(anyhow::anyhow!("Failed to upload file {}: HTTP {} - {}", path, status, error_text));
                    }
                    info!("Uploaded file {}: {} bytes", path, content.len());
                }
                UploadTask::Backup { original_path, backup_path, content } => {
                    // Upload backup without conflict detection or further backup
                    let payload = serde_json::json!({
                        "path": backup_path,
                        "mode": "overwrite",
                        "autorename": false,
                        "mute": true,
                        "strict_conflict": false
                    });
                    let response = self.client
                        .post(&format!("{}/files/upload", self.base_url))
                        .header("Dropbox-API-Arg", serde_json::to_string(&payload)?)
                        .header("Content-Type", "application/octet-stream")
                        .body(content.clone())
                        .send()
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to upload backup file {}: {}", backup_path, e))?;
                    if !response.status().is_success() {
                        let status = response.status();
                        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                        return Err(anyhow::anyhow!("Failed to upload backup file {}: HTTP {} - {}", backup_path, status, error_text));
                    }
                    info!("Created backup of {} as {} ({} bytes)", original_path, backup_path, content.len());
                }
            }
        }
        Ok(())
    }

    pub async fn upload_local_file(&self, local_path: &Path, remote_path: &str) -> Result<()> {
        if !local_path.exists() {
            return Err(anyhow::anyhow!("Local file does not exist: {}", local_path.display()));
        }
        if !local_path.is_file() {
            return Err(anyhow::anyhow!("Path is not a file: {}", local_path.display()));
        }
        let content = fs::read(local_path)
            .map_err(|e| anyhow::anyhow!("Failed to read local file {}: {}", local_path.display(), e))?;
        self.upload_file_with_options(remote_path, &content, &UploadOptions::default()).await
    }

    async fn detect_conflict(&self, path: &str, local_content: &[u8]) -> Result<ConflictResult> {
        match self.get_metadata(path).await {
            Ok(remote_metadata) => {
                let local_size = local_content.len() as u64;
                let remote_size = remote_metadata.size;
                if local_size != remote_size {
                    return Ok(ConflictResult::Conflict {
                        local_size,
                        remote_size,
                        local_modified: None,
                        remote_modified: remote_metadata.server_modified
                            .as_ref()
                            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.with_timezone(&Utc)),
                        local_hash: None,
                        remote_hash: remote_metadata.content_hash,
                    });
                }
                Ok(ConflictResult::NoConflict)
            }
            Err(e) => {
                if e.to_string().contains("not_found") {
                    Ok(ConflictResult::NoConflict)
                } else {
                    Ok(ConflictResult::Error(e.to_string()))
                }
            }
        }
    }

    pub async fn upload_files_batch(&self, files: &[(String, Vec<u8>)]) -> Result<Vec<Result<()>>> {
        let mut results = Vec::new();
        for (path, content) in files {
            let result = self.upload_file_with_options(path, content, &UploadOptions::default()).await;
            results.push(result);
        }
        Ok(results)
    }

    pub async fn upload_directory(&self, local_dir: &Path, remote_base: &str) -> Result<()> {
        if !local_dir.exists() || !local_dir.is_dir() {
            return Err(anyhow::anyhow!("Local directory does not exist or is not a directory: {}", local_dir.display()));
        }

        let mut queue = VecDeque::new();
        queue.push_back((local_dir.to_path_buf(), remote_base.to_string()));

        while let Some((current_dir, current_remote_base)) = queue.pop_front() {
            for entry in fs::read_dir(&current_dir)
                .map_err(|e| anyhow::anyhow!("Failed to read directory {}: {}", current_dir.display(), e))? {
                
                let entry = entry
                    .map_err(|e| anyhow::anyhow!("Failed to read directory entry: {}", e))?;
                
                let entry_path = entry.path();
                let relative_path = entry_path.strip_prefix(local_dir)
                    .map_err(|e| anyhow::anyhow!("Failed to get relative path: {}", e))?;
                
                let remote_path = format!("{}/{}", current_remote_base, relative_path.display());

                if entry_path.is_file() {
                    self.upload_local_file(&entry_path, &remote_path).await?;
                } else if entry_path.is_dir() {
                    // Add subdirectory to queue instead of recursive call
                    queue.push_back((entry_path, remote_path));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    #[tokio::test]
    async fn test_upload_options_default() {
        let options = UploadOptions::default();
        assert_eq!(options.overwrite, false);
        assert_eq!(options.create_backup, true);
        assert_eq!(options.autorename, false);
        assert_eq!(options.mute, false);
    }

    #[test]
    fn test_conflict_result_debug() {
        let no_conflict = ConflictResult::NoConflict;
        let conflict = ConflictResult::Conflict {
            local_size: 100,
            remote_size: 200,
            local_modified: None,
            remote_modified: None,
            local_hash: None,
            remote_hash: None,
        };
        let error = ConflictResult::Error("test error".to_string());
        format!("{:?}", no_conflict);
        format!("{:?}", conflict);
        format!("{:?}", error);
    }
} 