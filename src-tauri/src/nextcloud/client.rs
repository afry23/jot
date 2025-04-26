use chrono::DateTime;
use log::{debug, info, warn};
use reqwest::{Client, Method, StatusCode};
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

use crate::nextcloud::error::SyncError;
use crate::nextcloud::types::{NextcloudConfig, NoteStatus, SyncStatus};

// Custom method for WebDAV operations
fn webdav_method(name: &str) -> Method {
    Method::from_bytes(name.as_bytes()).unwrap_or(Method::GET)
}

// Format timestamp for logging purposes
fn format_timestamp(timestamp: u64) -> String {
    use std::time::{Duration, UNIX_EPOCH};

    let datetime = UNIX_EPOCH + Duration::from_secs(timestamp);

    match datetime.duration_since(UNIX_EPOCH) {
        Ok(_duration) => {
            if let Ok(datetime) = datetime.duration_since(UNIX_EPOCH) {
                let secs = datetime.as_secs();
                let mins = (secs / 60) % 60;
                let hours = (secs / 3600) % 24;
                let days = secs / 86400;

                format!(
                    "{} days, {:02}:{:02} (timestamp: {})",
                    days, hours, mins, timestamp
                )
            } else {
                format!("Invalid time (timestamp: {})", timestamp)
            }
        }
        Err(_) => format!("Invalid time (timestamp: {})", timestamp),
    }
}

// Parse HTTP date
fn parse_http_date(date_str: &str) -> Option<u64> {
    match DateTime::parse_from_rfc2822(date_str) {
        Ok(datetime) => Some(datetime.timestamp() as u64),
        Err(e) => {
            debug!("Error parsing date '{}': {}", date_str, e);
            None
        }
    }
}

// Try to extract timestamp from ETag
fn extract_timestamp_from_etag(etag: &str) -> Option<u64> {
    let digits: String = etag.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.len() >= 10 {
        if let Ok(timestamp) = digits[..10].parse::<u64>() {
            return Some(timestamp);
        }
    }

    None
}

// Find last modified in XML response
fn find_last_modified_in_xml(xml: &str) -> Option<u64> {
    let possible_tags = [
        "<d:getlastmodified>",
        "<getlastmodified>",
        "<lastmodified>",
        "<ns0:getlastmodified>",
        "<DAV:getlastmodified>",
    ];

    for start_tag in possible_tags.iter() {
        let end_tag = start_tag.replace("<", "</");

        if let Some(pos) = xml.find(start_tag) {
            let start = pos + start_tag.len();
            if let Some(end) = xml[start..].find(&end_tag) {
                let date_str = &xml[start..start + end];
                debug!("Found date string: {}", date_str);

                return parse_http_date(date_str);
            }
        }
    }

    // Look for getetag
    if let Some(pos) = xml.find("<d:getetag>") {
        let start = pos + "<d:getetag>".len();
        if let Some(end) = xml[start..].find("</d:getetag>") {
            let etag = &xml[start..start + end];
            debug!("Found ETag: {}", etag);

            if let Some(timestamp) = extract_timestamp_from_etag(etag) {
                return Some(timestamp);
            }
        }
    }

    // Fallback: Get current time
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    debug!("Using current time as fallback: {}", now);
    Some(now)
}

pub struct NextcloudClient {
    config: NextcloudConfig,
    client: Client,
}

impl NextcloudClient {
    // Create a new Nextcloud client
    pub fn new(config: NextcloudConfig) -> Result<Self, SyncError> {
        // Validate configuration
        if config.server_url.is_empty() || config.username.is_empty() || config.password.is_empty()
        {
            return Err(SyncError::Configuration(
                "Nextcloud configuration is incomplete".into(),
            ));
        }

        // Create HTTP client
        let client = Client::builder()
            .build()
            .map_err(|e| SyncError::Request(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    // Get WebDAV URL for a note
    pub fn get_note_webdav_url(&self, tab_index: usize) -> Result<String, SyncError> {
        let server_url = self.config.server_url.trim_end_matches('/');

        // Ensure server URL is valid
        let _parsed_url = Url::parse(server_url)?;

        // Build WebDAV path
        let mut webdav_path = format!(
            "{}/remote.php/dav/files/{}",
            server_url, self.config.username
        );

        // Add sync folder
        let sync_folder = if self.config.sync_folder.starts_with('/') {
            self.config.sync_folder.clone()
        } else {
            format!("/{}", self.config.sync_folder)
        };

        webdav_path.push_str(&sync_folder);

        // Ensure the path ends with a slash for directory access
        if !webdav_path.ends_with('/') {
            webdav_path.push('/');
        }

        // Add note filename
        webdav_path.push_str(&format!("note_{}.md", tab_index));

        Ok(webdav_path)
    }

    // Test connection to Nextcloud server
    pub async fn test_connection(&self) -> Result<bool, SyncError> {
        let server_url = self.config.server_url.trim_end_matches('/');
        let webdav_path = format!(
            "{}/remote.php/dav/files/{}",
            server_url, self.config.username
        );

        // Test connection with a simple PROPFIND request
        let request = self
            .client
            .request(webdav_method("PROPFIND"), &webdav_path)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .header("Depth", "0")
            .build()
            .map_err(|e| SyncError::Request(format!("Failed to build request: {}", e)))?;

        let response = self
            .client
            .execute(request)
            .await
            .map_err(|e| SyncError::Request(format!("Connection test failed: {}", e)))?;

        Ok(response.status().is_success())
    }

    // Ensure the remote directory exists
    pub async fn ensure_remote_directory(&self) -> Result<(), SyncError> {
        let server_url = self.config.server_url.trim_end_matches('/');

        // Build WebDAV path for the directory
        let mut webdav_path = format!(
            "{}/remote.php/dav/files/{}",
            server_url, self.config.username
        );

        // Add sync folder
        let sync_folder = if self.config.sync_folder.starts_with('/') {
            self.config.sync_folder.clone()
        } else {
            format!("/{}", self.config.sync_folder)
        };

        webdav_path.push_str(&sync_folder);

        // Check if directory exists
        let request = self
            .client
            .request(webdav_method("PROPFIND"), &webdav_path)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .header("Depth", "0")
            .build()
            .map_err(|e| SyncError::Request(format!("Failed to build request: {}", e)))?;

        let response = self
            .client
            .execute(request)
            .await
            .map_err(|e| SyncError::Request(format!("Failed to check directory: {}", e)))?;

        if response.status() == StatusCode::NOT_FOUND {
            // Directory doesn't exist, create it
            let request = self
                .client
                .request(webdav_method("MKCOL"), &webdav_path)
                .basic_auth(&self.config.username, Some(&self.config.password))
                .build()
                .map_err(|e| SyncError::Request(format!("Failed to build request: {}", e)))?;

            let response =
                self.client.execute(request).await.map_err(|e| {
                    SyncError::Request(format!("Failed to create directory: {}", e))
                })?;

            if !response.status().is_success() {
                return Err(SyncError::WebDav(format!(
                    "Failed to create directory, status: {}",
                    response.status()
                )));
            }
        } else if !response.status().is_success() {
            return Err(SyncError::WebDav(format!(
                "Failed to check directory, status: {}",
                response.status()
            )));
        }

        Ok(())
    }

    // Get modified time of remote note
    pub async fn get_remote_note_modified_time(
        &self,
        tab_index: usize,
    ) -> Result<Option<u64>, SyncError> {
        let webdav_url = self.get_note_webdav_url(tab_index)?;

        debug!(
            "Checking remote modified time for tab {} at URL: {}",
            tab_index, webdav_url
        );

        let request = self
            .client
            .request(webdav_method("PROPFIND"), &webdav_url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .header("Depth", "0")
            .header("Content-Type", "application/xml")
            .body(
                r#"<?xml version="1.0" encoding="utf-8"?>
                <propfind xmlns="DAV:">
                    <prop>
                    <getlastmodified/>
                    </prop>
                </propfind>"#,
            )
            .build()
            .map_err(|e| SyncError::Request(format!("Failed to build request: {}", e)))?;

        let response =
            self.client.execute(request).await.map_err(|e| {
                SyncError::Request(format!("Failed to get remote note info: {}", e))
            })?;

        let status = response.status();
        debug!("PROPFIND status for tab {}: {}", tab_index, status);

        if status == StatusCode::NOT_FOUND {
            debug!("Remote note {} does not exist", tab_index);
            return Ok(None);
        }

        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(SyncError::Response(format!(
                "Failed to get remote note info, status: {}, body: {}",
                status, error_body
            )));
        }

        // Parse response XML to extract last modified time
        let body = response
            .text()
            .await
            .map_err(|e| SyncError::Response(format!("Failed to read response: {}", e)))?;

        debug!(
            "PROPFIND response for tab {} (truncated): {}",
            tab_index,
            if body.len() > 500 {
                &body[0..500]
            } else {
                &body
            }
        );

        // Extract the last modified date
        let last_modified = find_last_modified_in_xml(&body);

        debug!(
            "Extracted last modified for tab {}: {:?}",
            tab_index, last_modified
        );

        Ok(last_modified)
    }

    // Get remote modified time from HEAD request as a fallback
    pub async fn get_remote_mod_time_from_head(
        &self,
        tab_index: usize,
    ) -> Result<Option<u64>, SyncError> {
        let webdav_url = self.get_note_webdav_url(tab_index)?;

        debug!(
            "Checking remote modified time via HEAD for tab {} at URL: {}",
            tab_index, webdav_url
        );

        let response = self
            .client
            .head(&webdav_url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .send()
            .await
            .map_err(|e| {
                SyncError::Request(format!("Failed to get remote note HEAD info: {}", e))
            })?;

        let status = response.status();
        debug!("HEAD status for tab {}: {}", tab_index, status);

        if status == StatusCode::NOT_FOUND {
            debug!("Remote note {} does not exist", tab_index);
            return Ok(None);
        }

        if !status.is_success() {
            return Err(SyncError::Response(format!(
                "Failed to get remote note HEAD info, status: {}",
                status
            )));
        }

        // Extract Last-Modified header
        if let Some(last_modified) = response.headers().get(reqwest::header::LAST_MODIFIED) {
            debug!("HEAD Last-Modified header: {:?}", last_modified);

            if let Ok(last_modified_str) = last_modified.to_str() {
                debug!("Last-Modified string: {}", last_modified_str);

                // Try to parse the HTTP date
                if let Some(timestamp) = parse_http_date(last_modified_str) {
                    debug!(
                        "Parsed timestamp from header: {} ({})",
                        timestamp,
                        format_timestamp(timestamp)
                    );
                    return Ok(Some(timestamp));
                }
            }
        }

        // Extract ETag as fallback
        if let Some(etag) = response.headers().get(reqwest::header::ETAG) {
            debug!("HEAD ETag header: {:?}", etag);

            if let Ok(etag_str) = etag.to_str() {
                debug!("ETag string: {}", etag_str);

                // Try to extract timestamp from ETag
                if let Some(timestamp) = extract_timestamp_from_etag(etag_str) {
                    debug!(
                        "Parsed timestamp from ETag: {} ({})",
                        timestamp,
                        format_timestamp(timestamp)
                    );
                    return Ok(Some(timestamp));
                }
            }
        }

        debug!("Could not extract timestamp from HEAD response");
        Ok(None)
    }

    // Upload a note to Nextcloud
    pub async fn upload_note(&self, tab_index: usize, content: &str) -> Result<(), SyncError> {
        let webdav_url = self.get_note_webdav_url(tab_index)?;

        debug!("Uploading note {} to URL: {}", tab_index, webdav_url);

        // Upload note to Nextcloud
        let response = self
            .client
            .put(&webdav_url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .body(content.to_string())
            .send()
            .await
            .map_err(|e| SyncError::Request(format!("Failed to upload note: {}", e)))?;

        if !response.status().is_success() {
            return Err(SyncError::Response(format!(
                "Failed to upload note, status: {}",
                response.status()
            )));
        }

        debug!("Note {} uploaded successfully", tab_index);
        Ok(())
    }

    // Download a note from Nextcloud
    pub async fn download_note(&self, tab_index: usize) -> Result<String, SyncError> {
        let webdav_url = self.get_note_webdav_url(tab_index)?;

        debug!("Downloading note {} from URL: {}", tab_index, webdav_url);

        // Download note from Nextcloud
        let response = self
            .client
            .get(&webdav_url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .send()
            .await
            .map_err(|e| SyncError::Request(format!("Failed to download note: {}", e)))?;

        let status = response.status();
        debug!("Download status for note {}: {}", tab_index, status);

        if status == StatusCode::NOT_FOUND {
            return Err(SyncError::WebDav(format!(
                "Remote note {} does not exist",
                tab_index
            )));
        }

        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(SyncError::Response(format!(
                "Failed to download note, status: {}, body: {}",
                status, error_body
            )));
        }

        let content = response
            .text()
            .await
            .map_err(|e| SyncError::Response(format!("Failed to read response: {}", e)))?;

        debug!(
            "Downloaded note {} successfully ({} bytes)",
            tab_index,
            content.len()
        );

        Ok(content)
    }

    // Sync a single note
    pub async fn sync_note(
        &self,
        tab_index: usize,
        local_path: &Path,
        emit_event: bool,
    ) -> Result<NoteStatus, SyncError> {
        let local_exists = local_path.exists();

        info!("\n===== SYNC NOTE {} START =====", tab_index);
        debug!("Note {}: Local file exists: {}", tab_index, local_exists);

        // Try PROPFIND first, then HEAD as fallback
        let remote_modified_propfind = self.get_remote_note_modified_time(tab_index).await;
        let remote_modified = match remote_modified_propfind {
            Ok(time) => time,
            Err(e) => {
                debug!("PROPFIND error, trying HEAD: {}", e);
                match self.get_remote_mod_time_from_head(tab_index).await {
                    Ok(time) => time,
                    Err(e2) => {
                        debug!("HEAD error too: {}", e2);
                        None
                    }
                }
            }
        };

        // Get local modified time
        let local_modified = if local_exists {
            match std::fs::metadata(local_path) {
                Ok(metadata) => {
                    match metadata.modified() {
                        Ok(time) => {
                            match time.duration_since(UNIX_EPOCH) {
                                Ok(duration) => {
                                    let timestamp = duration.as_secs();
                                    debug!(
                                        "Local file timestamp: {} ({})",
                                        timestamp,
                                        format_timestamp(timestamp)
                                    );
                                    timestamp
                                }
                                Err(e) => {
                                    warn!("Error calculating local timestamp: {}", e);
                                    // Fallback to current time
                                    let now = SystemTime::now()
                                        .duration_since(UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs();
                                    debug!(
                                        "Using current time as fallback: {} ({})",
                                        now,
                                        format_timestamp(now)
                                    );
                                    now
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Error getting local modified time: {}", e);
                            // Fallback to current time
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            debug!(
                                "Using current time as fallback: {} ({})",
                                now,
                                format_timestamp(now)
                            );
                            now
                        }
                    }
                }
                Err(e) => {
                    warn!("Error getting local file metadata: {}", e);
                    0
                }
            }
        } else {
            0
        };

        info!(
            "Note {}: Local timestamp: {}, Remote timestamp: {:?}",
            tab_index, local_modified, remote_modified
        );

        if let Some(remote) = remote_modified {
            debug!("  Remote: {} ({})", remote, format_timestamp(remote));

            // Show the time difference
            let diff_secs = if remote > local_modified {
                remote - local_modified
            } else {
                local_modified - remote
            };

            debug!("  Time difference: {} seconds", diff_secs);
            debug!(
                "  Which is newer: {}",
                match remote.cmp(&local_modified) {
                    Ordering::Greater => "REMOTE", // remote > local_modified
                    Ordering::Less => "LOCAL",     // remote < local_modified
                    Ordering::Equal => "SAME",     // remote == local_modified
                }
            );
        } else {
            debug!("  Remote: DOES NOT EXIST");
        }

        let mut note_status = NoteStatus {
            tab_index,
            local_modified,
            remote_modified,
            synced: false,
            conflict: false,
        };

        // Logic for sync decisions
        if !local_exists && remote_modified.is_none() {
            // Nothing to sync, both sides are empty
            debug!("Note {}: Nothing to sync (both empty)", tab_index);
            note_status.synced = true;
            return Ok(note_status);
        }

        if !local_exists && remote_modified.is_some() {
            // Download remote note
            match self.download_note(tab_index).await {
                Ok(content) => {
                    // Save to local
                    std::fs::write(local_path, &content).map_err(|e| {
                        SyncError::FileSystem(format!("Failed to save downloaded note: {}", e))
                    })?;

                    note_status.synced = true;

                    // Return the content for event emission in the command layer
                    if emit_event {
                        // This needs to be handled by the calling code
                    }
                }
                Err(e) => {
                    warn!("Error downloading note {}: {}", tab_index, e);
                    note_status.synced = false;
                }
            }
            return Ok(note_status);
        }

        if local_exists && remote_modified.is_none() {
            debug!("Note {}: Uploading to remote (remote missing)", tab_index);
            // Upload local note
            let content = std::fs::read_to_string(local_path)
                .map_err(|e| SyncError::FileSystem(format!("Failed to read local note: {}", e)))?;

            match self.upload_note(tab_index, &content).await {
                Ok(_) => {
                    note_status.synced = true;
                }
                Err(e) => {
                    warn!("Error uploading note {}: {}", tab_index, e);
                    note_status.synced = false;
                }
            }
            return Ok(note_status);
        }

        // Both exist, check timestamps with an adjustable tolerance
        // Add a time tolerance to reduce unnecessary syncs (e.g., 2 seconds)
        const TIME_TOLERANCE_SECS: u64 = 2;

        if let Some(remote_time) = remote_modified {
            // Check if local is newer by more than the tolerance
            if local_modified > remote_time && local_modified - remote_time > TIME_TOLERANCE_SECS {
                debug!("Note {}: Local is newer, uploading to remote", tab_index);
                // Local is newer, upload
                let content = std::fs::read_to_string(local_path).map_err(|e| {
                    SyncError::FileSystem(format!("Failed to read local note: {}", e))
                })?;

                match self.upload_note(tab_index, &content).await {
                    Ok(_) => {
                        note_status.synced = true;
                    }
                    Err(e) => {
                        warn!("Error uploading note {}: {}", tab_index, e);
                        note_status.synced = false;
                    }
                }
            }
            // Check if remote is newer by more than the tolerance
            else if remote_time > local_modified
                && remote_time - local_modified > TIME_TOLERANCE_SECS
            {
                debug!(
                    "Note {}: Remote is newer, downloading from remote",
                    tab_index
                );
                // Remote is newer, download
                match self.download_note(tab_index).await {
                    Ok(content) => {
                        // Compare content to detect if there are actual changes
                        let local_content = match std::fs::read_to_string(local_path) {
                            Ok(content) => content,
                            Err(e) => {
                                warn!("Error reading local note: {}", e);
                                "".to_string()
                            }
                        };

                        if content != local_content {
                            debug!("Note {}: Content differs, updating local file", tab_index);
                            // Save to local only if content actually differs
                            std::fs::write(local_path, &content).map_err(|e| {
                                SyncError::FileSystem(format!(
                                    "Failed to save downloaded note: {}",
                                    e
                                ))
                            })?;

                            if emit_event {
                                // This needs to be handled by the calling code
                            }
                        } else {
                            debug!(
                                "Note {}: Content identical despite timestamp difference",
                                tab_index
                            );
                        }
                        note_status.synced = true;
                    }
                    Err(e) => {
                        warn!("Error downloading note {}: {}", tab_index, e);
                        note_status.synced = false;
                    }
                }
            } else {
                // Timestamps are close enough, consider synced
                debug!("Note {}: Timestamps close, considering synced", tab_index);
                note_status.synced = true;
            }
        }

        info!("===== SYNC NOTE {} END =====\n", tab_index);
        Ok(note_status)
    }

    // Sync all notes
    pub async fn sync_all_notes(
        &self,
        get_note_path: impl Fn(usize) -> PathBuf,
        emit_events: bool,
    ) -> Result<SyncStatus, SyncError> {
        info!("Syncing all notes...");

        // Ensure remote directory exists
        self.ensure_remote_directory().await?;

        // Sync all 7 tabs
        let mut notes_status = std::collections::HashMap::new();
        for tab_index in 0..7 {
            let note_path = get_note_path(tab_index);
            match self.sync_note(tab_index, &note_path, emit_events).await {
                Ok(status) => {
                    notes_status.insert(tab_index, status);
                }
                Err(e) => {
                    warn!("Error syncing note {}: {}", tab_index, e);
                    // Continue with other notes even if one fails
                }
            }
        }

        // Create sync status
        let sync_status = SyncStatus {
            last_sync: Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            ),
            syncing: false,
            error: None,
            notes_status,
        };

        Ok(sync_status)
    }
}
