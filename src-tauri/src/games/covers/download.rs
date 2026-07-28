use std::io;
use std::sync::Arc;
use std::time::Duration;

use reqwest::redirect::Policy;
use reqwest::{StatusCode, Url};

const MAX_REMOTE_COVER_BYTES: usize = 10 * 1024 * 1024;
const DOWNLOAD_TIMEOUT_SECONDS: u64 = 20;
const MAX_REDIRECTS: usize = 5;

type UrlValidator = Arc<dyn Fn(&Url) -> Result<(), String> + Send + Sync>;

#[derive(Clone, Copy)]
struct DownloadOptions {
    max_bytes: usize,
    timeout: Duration,
    max_redirects: usize,
    https_only: bool,
}

impl DownloadOptions {
    fn production() -> Self {
        Self {
            max_bytes: MAX_REMOTE_COVER_BYTES,
            timeout: Duration::from_secs(DOWNLOAD_TIMEOUT_SECONDS),
            max_redirects: MAX_REDIRECTS,
            https_only: true,
        }
    }
}

pub(crate) async fn download_cover(url: &str) -> Result<Vec<u8>, String> {
    download_cover_with_options(
        url,
        DownloadOptions::production(),
        Arc::new(validate_trusted_url),
    )
    .await
}

async fn download_cover_with_options(
    url: &str,
    options: DownloadOptions,
    validate_url: UrlValidator,
) -> Result<Vec<u8>, String> {
    let url = Url::parse(url).map_err(|_| "联网封面下载地址无效".to_string())?;
    validate_url(&url)?;

    let _ = rustls::crypto::ring::default_provider().install_default();
    let redirect_validator = Arc::clone(&validate_url);
    let client = reqwest::Client::builder()
        .https_only(options.https_only)
        .timeout(options.timeout)
        .redirect(Policy::custom(move |attempt| {
            if attempt.previous().len() >= options.max_redirects {
                return attempt.error(io::Error::other("联网封面重定向次数过多"));
            }
            if redirect_validator(attempt.url()).is_err() {
                return attempt.error(io::Error::other("联网封面重定向地址不受信任"));
            }
            attempt.follow()
        }))
        .user_agent(format!("Game Shift/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|_| "无法初始化联网封面下载客户端".to_string())?;

    let mut response = client.get(url).send().await.map_err(map_download_error)?;
    validate_url(response.url())?;

    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        return Err("封面下载请求过于频繁，请稍后再试".to_string());
    }
    if !response.status().is_success() {
        return Err(format!(
            "联网封面下载失败（HTTP {}）",
            response.status().as_u16()
        ));
    }
    if response
        .content_length()
        .is_some_and(|length| length > options.max_bytes as u64)
    {
        return Err("联网封面文件不能超过 10 MB".to_string());
    }

    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(map_download_error)? {
        if bytes.len().saturating_add(chunk.len()) > options.max_bytes {
            return Err("联网封面文件不能超过 10 MB".to_string());
        }
        bytes.extend_from_slice(&chunk);
    }
    if bytes.is_empty() {
        return Err("联网封面内容为空".to_string());
    }

    Ok(bytes)
}

pub(super) fn validate_trusted_url(url: &Url) -> Result<(), String> {
    if url.scheme() != "https" {
        return Err("联网封面只允许使用 HTTPS 地址".to_string());
    }
    let host = url
        .host_str()
        .ok_or_else(|| "联网封面下载地址缺少域名".to_string())?;
    if host == "steamgriddb.com" || host.ends_with(".steamgriddb.com") {
        Ok(())
    } else {
        Err("联网封面下载地址不受信任".to_string())
    }
}

fn map_download_error(error: reqwest::Error) -> String {
    if error.is_timeout() {
        "下载联网封面超时，请稍后再试".to_string()
    } else if error.is_connect() {
        "无法下载联网封面，请检查网络后重试".to_string()
    } else {
        "下载联网封面失败，请稍后再试".to_string()
    }
}

#[cfg(test)]
#[path = "download_tests.rs"]
mod tests;
