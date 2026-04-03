//! Twitter API client library for Cliptions
//!
//! Provides a high-level async interface for Twitter API v2 operations
//! including posting tweets, uploading images, and retrieving user data.

use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use base64::Engine;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use rand::Rng;
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use thiserror::Error;
use urlencoding;

type HmacSha1 = Hmac<Sha1>;

#[derive(Error, Debug)]
pub enum TwitterError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Twitter API error ({status}): {message}")]
    ApiError { status: u16, message: String },

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Invalid file format: {0}")]
    FileError(String),

    #[error("Response parsing error: {0}")]
    ParseError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Media upload error: {0}")]
    MediaError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Result type alias for TwitterError
pub type Result<T> = std::result::Result<T, TwitterError>;

/// Configuration for Twitter API authentication
#[derive(Debug, Clone)]
pub struct TwitterConfig {
    pub api_key: String,
    pub api_secret: String,
    pub access_token: String,
    pub access_token_secret: String,
}

impl TwitterConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            api_key: std::env::var("TWITTER_API_KEY").map_err(|_| {
                TwitterError::AuthError("TWITTER_API_KEY environment variable not set".to_string())
            })?,
            api_secret: std::env::var("TWITTER_API_SECRET").map_err(|_| {
                TwitterError::AuthError(
                    "TWITTER_API_SECRET environment variable not set".to_string(),
                )
            })?,
            access_token: std::env::var("TWITTER_ACCESS_TOKEN").map_err(|_| {
                TwitterError::AuthError(
                    "TWITTER_ACCESS_TOKEN environment variable not set".to_string(),
                )
            })?,
            access_token_secret: std::env::var("TWITTER_ACCESS_TOKEN_SECRET").map_err(|_| {
                TwitterError::AuthError(
                    "TWITTER_ACCESS_TOKEN_SECRET environment variable not set".to_string(),
                )
            })?,
        })
    }
}

/// Represents a Twitter user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitterUser {
    pub id: String,
    pub username: String,
    pub name: String,
    pub verified: Option<bool>,
}

/// Represents a tweet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tweet {
    pub id: String,
    pub text: String,
    pub author_id: String,
    pub created_at: Option<DateTime<Utc>>,
    pub conversation_id: Option<String>,
    pub public_metrics: Option<PublicMetrics>,
    pub url: String,
}

impl Default for Tweet {
    fn default() -> Self {
        Self {
            id: String::new(),
            text: String::new(),
            author_id: String::new(),
            created_at: None,
            conversation_id: None,
            public_metrics: None,
            url: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicMetrics {
    pub retweet_count: u32,
    pub reply_count: u32,
    pub like_count: u32,
    pub quote_count: u32,
}

impl Default for PublicMetrics {
    fn default() -> Self {
        Self {
            retweet_count: 0,
            reply_count: 0,
            like_count: 0,
            quote_count: 0,
        }
    }
}

/// Result from posting a tweet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostTweetResult {
    pub tweet: Tweet,
    pub success: bool,
}

/// Result from uploading media
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaUploadResult {
    pub media_id: String,
    pub size: u64,
    pub media_type: String,
}

/// Trait for a Twitter API client, enabling mocking for tests.
#[async_trait]
pub trait TwitterApi {
    async fn post_tweet(&self, text: &str) -> Result<PostTweetResult>;
    async fn post_tweet_with_image<P: AsRef<Path> + Send + 'static>(
        &self,
        text: &str,
        image_path: P,
    ) -> Result<PostTweetResult>;
    async fn quote_tweet(&self, text: &str, quote_tweet_id: &str) -> Result<PostTweetResult>;
    async fn reply_to_tweet(&self, text: &str, reply_to_tweet_id: &str) -> Result<PostTweetResult>;
    async fn reply_to_tweet_with_image<P: AsRef<Path> + Send + 'static>(
        &self,
        text: &str,
        reply_to_tweet_id: &str,
        image_path: P,
    ) -> Result<PostTweetResult>;
    async fn get_latest_tweet(
        &self,
        username: &str,
        exclude_retweets_replies: bool,
    ) -> Result<Option<Tweet>>;
    async fn get_user_tweets(
        &self,
        username: &str,
        max_results: u32,
        exclude_retweets_replies: bool,
    ) -> Result<Vec<Tweet>>;
    async fn search_replies(&self, tweet_id: &str, max_results: u32) -> Result<Vec<Tweet>>;
}

/// High-level Twitter API client
#[derive(Debug, Clone)]
pub struct TwitterClient {
    config: TwitterConfig,
    client: reqwest::Client,
}

#[async_trait]
impl TwitterApi for TwitterClient {
    async fn post_tweet(&self, text: &str) -> Result<PostTweetResult> {
        let tweet_data = serde_json::json!({
            "text": text
        });

        self.post_tweet_internal(tweet_data).await
    }

    async fn post_tweet_with_image<P: AsRef<Path> + Send + 'static>(
        &self,
        text: &str,
        image_path: P,
    ) -> Result<PostTweetResult> {
        // Upload the image first
        let media_result = self.upload_media(image_path).await?;

        // Create tweet with media attachment
        let tweet_data = serde_json::json!({
            "text": text,
            "media": {
                "media_ids": [media_result.media_id]
            }
        });

        self.post_tweet_internal(tweet_data).await
    }

    async fn quote_tweet(&self, text: &str, quote_tweet_id: &str) -> Result<PostTweetResult> {
        let tweet_data = serde_json::json!({
            "text": text,
            "quote_tweet_id": quote_tweet_id
        });

        self.post_tweet_internal(tweet_data).await
    }

    async fn reply_to_tweet(&self, text: &str, reply_to_tweet_id: &str) -> Result<PostTweetResult> {
        let tweet_data = serde_json::json!({
            "text": text,
            "reply": {
                "in_reply_to_tweet_id": reply_to_tweet_id
            }
        });

        self.post_tweet_internal(tweet_data).await
    }

    async fn reply_to_tweet_with_image<P: AsRef<Path> + Send + 'static>(
        &self,
        text: &str,
        reply_to_tweet_id: &str,
        image_path: P,
    ) -> Result<PostTweetResult> {
        let media_result = self.upload_media(image_path).await?;
        let tweet_data = serde_json::json!({
            "text": text,
            "reply": {
                "in_reply_to_tweet_id": reply_to_tweet_id
            },
            "media": {
                "media_ids": [media_result.media_id]
            }
        });
        self.post_tweet_internal(tweet_data).await
    }

    async fn get_latest_tweet(
        &self,
        username: &str,
        exclude_retweets_replies: bool,
    ) -> Result<Option<Tweet>> {
        // Step 1: Get user ID from username
        let user_id = self.get_user_id(username).await?;

        // Step 2: Get the user's latest tweets
        let mut tweets_url = format!(
            "https://api.twitter.com/2/users/{}/tweets?max_results=5&tweet.fields=created_at,author_id,public_metrics,conversation_id&user.fields=username,name,verified&expansions=author_id",
            user_id
        );

        if exclude_retweets_replies {
            tweets_url.push_str("&exclude=retweets,replies");
        }

        let response = self
            .make_authenticated_request("GET", &tweets_url, None)
            .await?;
        let json: serde_json::Value = response.json().await?;

        // Extract the first tweet if available
        if let Some(tweets) = json["data"].as_array() {
            if let Some(tweet_data) = tweets.first() {
                let tweet = self.parse_tweet(tweet_data)?;
                return Ok(Some(tweet));
            }
        }

        Ok(None)
    }

    async fn get_user_tweets(
        &self,
        username: &str,
        max_results: u32,
        exclude_retweets_replies: bool,
    ) -> Result<Vec<Tweet>> {
        let user_id = self.get_user_id(username).await?;
        let clamped = max_results.clamp(5, 100);

        let mut tweets_url = format!(
            "https://api.twitter.com/2/users/{}/tweets?max_results={}&tweet.fields=created_at,author_id,public_metrics,conversation_id&user.fields=username,name,verified&expansions=author_id",
            user_id, clamped
        );

        if exclude_retweets_replies {
            tweets_url.push_str("&exclude=retweets,replies");
        }

        let response = self
            .make_authenticated_request("GET", &tweets_url, None)
            .await?;
        let json: serde_json::Value = response.json().await?;

        let mut tweets = Vec::new();
        if let Some(data) = json["data"].as_array() {
            for tweet_data in data {
                let tweet = self.parse_tweet(tweet_data)?;
                tweets.push(tweet);
            }
        }

        Ok(tweets)
    }

    async fn search_replies(&self, tweet_id: &str, max_results: u32) -> Result<Vec<Tweet>> {
        let query = format!("conversation_id:{} is:reply", tweet_id);
        let url = format!(
            "https://api.twitter.com/2/tweets/search/recent?query={}&max_results={}&tweet.fields=created_at,author_id,conversation_id,in_reply_to_user_id,referenced_tweets&user.fields=username,name&expansions=author_id",
            urlencoding::encode(&query),
            max_results
        );

        let mut all_replies = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let mut current_url = url.clone();
            if let Some(token) = &next_token {
                current_url.push_str(&format!("&pagination_token={}", token));
            }

            let response = self
                .make_authenticated_request("GET", &current_url, None)
                .await?;
            let json: serde_json::Value = response.json().await?;

            // Get tweets from this page
            if let Some(data) = json["data"].as_array() {
                for tweet_data in data {
                    let tweet = self.parse_tweet(tweet_data)?;
                    all_replies.push(tweet);
                }
            }

            // Check for next page
            if let Some(meta) = json["meta"].as_object() {
                if let Some(token) = meta.get("next_token").and_then(|t| t.as_str()) {
                    next_token = Some(token.to_string());
                } else {
                    break; // No more pages
                }
            } else {
                break;
            }
        }

        Ok(all_replies)
    }
}

impl TwitterClient {
    /// Create a new Twitter client with the given configuration
    pub fn new(config: TwitterConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Create a new Twitter client from environment variables
    pub fn from_env() -> Result<Self> {
        let config = TwitterConfig::from_env()?;
        Ok(Self::new(config))
    }

    /// Upload media file to Twitter and return media ID
    async fn upload_media<P: AsRef<Path>>(&self, image_path: P) -> Result<MediaUploadResult> {
        let path = image_path.as_ref();

        // Read the image file
        let image_data = std::fs::read(path).map_err(|e| {
            TwitterError::FileError(format!(
                "Failed to read image file {}: {}",
                path.display(),
                e
            ))
        })?;

        // Validate file size (5MB limit for images)
        if image_data.len() > 5 * 1024 * 1024 {
            return Err(TwitterError::FileError(
                "Image file too large (max 5MB)".to_string(),
            ));
        }

        // Detect media type from file extension
        let media_type = match path.extension().and_then(|ext| ext.to_str()) {
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("png") => "image/png",
            Some("gif") => "image/gif",
            Some("webp") => "image/webp",
            _ => {
                return Err(TwitterError::FileError(
                    "Unsupported image format. Supported: jpg, png, gif, webp".to_string(),
                ))
            }
        };

        // Twitter media upload endpoint
        let upload_url = "https://upload.twitter.com/1.1/media/upload.json";

        // Create OAuth header for upload request
        let auth_header = self.create_oauth_header("POST", upload_url, None)?;

        // Create multipart form
        let form = multipart::Form::new().part(
            "media",
            multipart::Part::bytes(image_data)
                .file_name(path.file_name().unwrap().to_string_lossy().to_string())
                .mime_str(media_type)
                .map_err(|e| TwitterError::FileError(format!("Invalid MIME type: {}", e)))?,
        );

        // Make upload request
        let response = self
            .client
            .post(upload_url)
            .header("Authorization", auth_header)
            .multipart(form)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let json: serde_json::Value = response.json().await?;

            let media_id = json["media_id_string"]
                .as_str()
                .ok_or_else(|| TwitterError::ApiError {
                    status: status.as_u16(),
                    message: "No media_id_string in response".to_string(),
                })?
                .to_string();

            let size = json["size"].as_u64().unwrap_or(0);

            Ok(MediaUploadResult {
                media_id,
                size,
                media_type: media_type.to_string(),
            })
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(TwitterError::ApiError {
                status: status.as_u16(),
                message: error_text,
            })
        }
    }

    /// Internal method to handle all tweet posting logic
    async fn post_tweet_internal(&self, tweet_data: serde_json::Value) -> Result<PostTweetResult> {
        let url = "https://api.twitter.com/2/tweets";

        if std::env::var("CLIPTIONS_DEBUG").is_ok() {
            println!("[DEBUG] post_tweet_internal: url = {}", url);
            println!("[DEBUG] post_tweet_internal: tweet_data = {}", tweet_data);
        }

        let response = self
            .make_authenticated_request("POST", url, Some(tweet_data))
            .await?;
        let json: serde_json::Value = response.json().await?;

        // Parse the tweet from the response
        if let Some(tweet_data) = json["data"].as_object() {
            // For posted tweets, we need to create a minimal Tweet struct since the API
            // doesn't return all fields. We'll use the authenticated user's ID as author_id.
            let id = tweet_data["id"]
                .as_str()
                .ok_or_else(|| TwitterError::ParseError("Missing tweet id".to_string()))?
                .to_string();

            let text = tweet_data["text"]
                .as_str()
                .ok_or_else(|| TwitterError::ParseError("Missing tweet text".to_string()))?
                .to_string();

            // For posted tweets, we know the author_id is the authenticated user
            // We can derive this from the access token, but for now we'll use "self"
            let author_id = "self".to_string();

            let url = format!("https://twitter.com/i/status/{}", id);

            let tweet = Tweet {
                id,
                text,
                author_id,
                created_at: Some(Utc::now()), // Use current time for posted tweets
                conversation_id: tweet_data
                    .get("conversation_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                public_metrics: None, // Not available immediately after posting
                url,
            };

            Ok(PostTweetResult {
                tweet,
                success: true,
            })
        } else {
            Err(TwitterError::ApiError {
                status: 200,
                message: "Invalid response format".to_string(),
            })
        }
    }

    /// Get user ID from username
    async fn get_user_id(&self, username: &str) -> Result<String> {
        let user_lookup_url = format!("https://api.twitter.com/2/users/by/username/{}", username);

        let response = self
            .make_authenticated_request("GET", &user_lookup_url, None)
            .await?;
        let json: serde_json::Value = response.json().await?;

        json["data"]["id"]
            .as_str()
            .ok_or_else(|| TwitterError::ApiError {
                status: 404,
                message: format!("User not found: {}", username),
            })
            .map(|s| s.to_string())
    }

    /// Parse tweet data from JSON response
    fn parse_tweet(&self, tweet_data: &serde_json::Value) -> Result<Tweet> {
        let id = tweet_data["id"]
            .as_str()
            .ok_or_else(|| TwitterError::ParseError("Missing tweet id".to_string()))?
            .to_string();

        let text = tweet_data["text"]
            .as_str()
            .ok_or_else(|| TwitterError::ParseError("Missing tweet text".to_string()))?
            .to_string();

        let author_id = tweet_data["author_id"]
            .as_str()
            .ok_or_else(|| TwitterError::ParseError("Missing author_id".to_string()))?
            .to_string();

        let created_at = tweet_data["created_at"]
            .as_str()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let conversation_id = tweet_data["conversation_id"]
            .as_str()
            .map(|s| s.to_string());

        let public_metrics =
            tweet_data["public_metrics"]
                .as_object()
                .map(|metrics| PublicMetrics {
                    retweet_count: metrics["retweet_count"].as_u64().unwrap_or(0) as u32,
                    reply_count: metrics["reply_count"].as_u64().unwrap_or(0) as u32,
                    like_count: metrics["like_count"].as_u64().unwrap_or(0) as u32,
                    quote_count: metrics["quote_count"].as_u64().unwrap_or(0) as u32,
                });

        let url = format!("https://twitter.com/i/status/{}", id);

        Ok(Tweet {
            id,
            text,
            author_id,
            created_at,
            conversation_id,
            public_metrics,
            url,
        })
    }

    /// Make an authenticated HTTP request to the Twitter API
    async fn make_authenticated_request(
        &self,
        method: &str,
        url: &str,
        body: Option<serde_json::Value>,
    ) -> Result<reqwest::Response> {
        // Parse URL to separate base URL from query parameters for OAuth signature
        let (base_url, query_params) = if let Some(pos) = url.find('?') {
            let base = &url[..pos];
            let params = &url[pos + 1..];
            (base, Some(params))
        } else {
            (url, None)
        };

        let auth_header = self.create_oauth_header(method, base_url, query_params)?;
        if std::env::var("CLIPTIONS_DEBUG").is_ok() {
            println!("[DEBUG] make_authenticated_request: method = {}", method);
            println!("[DEBUG] make_authenticated_request: url = {}", url);
            println!(
                "[DEBUG] make_authenticated_request: Authorization = {}",
                auth_header
            );
            if let Some(ref json_body) = body {
                println!("[DEBUG] make_authenticated_request: body = {}", json_body);
            }
        }

        let mut request_builder = match method {
            "GET" => self.client.get(url),
            "POST" => self.client.post(url),
            _ => {
                return Err(TwitterError::AuthError(format!(
                    "Unsupported HTTP method: {}",
                    method
                )))
            }
        };

        request_builder = request_builder.header("Authorization", auth_header);

        if let Some(json_body) = body {
            request_builder = request_builder
                .header("Content-Type", "application/json")
                .json(&json_body);
        }

        let response = request_builder.send().await?;
        let status = response.status();

        if status.is_success() {
            Ok(response)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(TwitterError::ApiError {
                status: status.as_u16(),
                message: error_text,
            })
        }
    }

    /// Create OAuth 1.0a authorization header
    fn create_oauth_header(
        &self,
        method: &str,
        base_url: &str,
        query_params: Option<&str>,
    ) -> Result<String> {
        // Generate OAuth parameters
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        let nonce: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        // OAuth parameters
        let mut oauth_params = HashMap::new();
        oauth_params.insert("oauth_consumer_key", self.config.api_key.as_str());
        oauth_params.insert("oauth_token", self.config.access_token.as_str());
        oauth_params.insert("oauth_signature_method", "HMAC-SHA1");
        oauth_params.insert("oauth_timestamp", &timestamp);
        oauth_params.insert("oauth_nonce", &nonce);
        oauth_params.insert("oauth_version", "1.0");

        // Combine OAuth parameters with query parameters for signature
        let mut all_params: HashMap<String, String> = HashMap::new();

        // Add OAuth parameters
        for (k, v) in oauth_params.iter() {
            all_params.insert(k.to_string(), v.to_string());
        }

        // Parse query parameters if they exist
        if let Some(query_str) = query_params {
            for param in query_str.split('&') {
                if let Some(pos) = param.find('=') {
                    let key = &param[..pos];
                    let value = &param[pos + 1..];
                    // URL decode the parameters for signature
                    let decoded_key = urlencoding::decode(key).unwrap_or_else(|_| key.into());
                    let decoded_value = urlencoding::decode(value).unwrap_or_else(|_| value.into());
                    all_params.insert(decoded_key.into_owned(), decoded_value.into_owned());
                }
            }
        }

        // Create parameter string for signature (all parameters must be included)
        let mut sorted_params: Vec<_> = all_params.iter().collect();
        sorted_params.sort_by_key(|(k, _)| k.as_str());

        let param_string = sorted_params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        // Create signature base string
        let base_string = format!(
            "{}&{}&{}",
            method.to_uppercase(),
            urlencoding::encode(base_url),
            urlencoding::encode(&param_string)
        );

        // Create signing key
        let signing_key = format!(
            "{}&{}",
            urlencoding::encode(&self.config.api_secret),
            urlencoding::encode(&self.config.access_token_secret)
        );

        // Generate HMAC-SHA1 signature
        let mut mac = HmacSha1::new_from_slice(signing_key.as_bytes())
            .map_err(|e| TwitterError::AuthError(format!("HMAC key error: {}", e)))?;
        mac.update(base_string.as_bytes());
        let signature =
            base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());

        // Add signature to OAuth parameters
        oauth_params.insert("oauth_signature", &signature);

        // Create authorization header (only OAuth parameters)
        let auth_params: Vec<String> = oauth_params
            .iter()
            .filter(|(k, _)| k.starts_with("oauth_"))
            .map(|(k, v)| format!("{}=\"{}\"", urlencoding::encode(k), urlencoding::encode(v)))
            .collect();

        Ok(format!("OAuth {}", auth_params.join(", ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        pub TwitterApiClient {
            // This mirrors the TwitterApi trait
        }

        #[async_trait]
        impl TwitterApi for TwitterApiClient {
            async fn post_tweet(&self, text: &str) -> Result<PostTweetResult>;
            async fn post_tweet_with_image<P: AsRef<Path> + Send + 'static>(
                &self,
                text: &str,
                image_path: P,
            ) -> Result<PostTweetResult>;
            async fn quote_tweet(&self, text: &str, quote_tweet_id: &str) -> Result<PostTweetResult>;
            async fn reply_to_tweet(&self, text: &str, reply_to_tweet_id: &str) -> Result<PostTweetResult>;
            async fn reply_to_tweet_with_image<P: AsRef<Path> + Send + 'static>(
                &self,
                text: &str,
                reply_to_tweet_id: &str,
                image_path: P,
            ) -> Result<PostTweetResult>;
            async fn get_latest_tweet(
                &self,
                username: &str,
                exclude_retweets_replies: bool,
            ) -> Result<Option<Tweet>>;
            async fn get_user_tweets(
                &self,
                username: &str,
                max_results: u32,
                exclude_retweets_replies: bool,
            ) -> Result<Vec<Tweet>>;
            async fn search_replies(&self, tweet_id: &str, max_results: u32) -> Result<Vec<Tweet>>;
        }
    }

    #[tokio::test]
    async fn test_mock_post_tweet() {
        let mut mock_client = MockTwitterApiClient::new();

        let tweet_text = "Hello from Cliptions!";
        let expected_tweet = Tweet {
            id: "12345".to_string(),
            text: tweet_text.to_string(),
            author_id: "test_user".to_string(),
            created_at: Some(Utc::now()),
            conversation_id: None,
            public_metrics: None,
            url: "https://twitter.com/i/status/12345".to_string(),
        };

        let expected_result = PostTweetResult {
            tweet: expected_tweet.clone(),
            success: true,
        };

        mock_client
            .expect_post_tweet()
            .withf(move |text| text == tweet_text)
            .times(1)
            .returning(move |_| Ok(expected_result.clone()));

        let result = mock_client.post_tweet(tweet_text).await.unwrap();

        assert!(result.success);
        assert_eq!(result.tweet.id, "12345");
    }
}
