use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use reqwest::Url;

use super::{download_cover_with_options, validate_trusted_url, DownloadOptions, UrlValidator};

static HTTP_TEST_LOCK: Mutex<()> = Mutex::new(());

fn lock_http_tests() -> MutexGuard<'static, ()> {
    HTTP_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn spawn_server(
    request_count: usize,
    handler: impl Fn(usize) -> Vec<u8> + Send + 'static,
) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        listener.set_nonblocking(true).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut handled = 0;
        while handled < request_count && Instant::now() < deadline {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    stream
                        .set_read_timeout(Some(Duration::from_secs(1)))
                        .unwrap();
                    let mut request = Vec::new();
                    let mut buffer = [0_u8; 1024];
                    while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                        let read = stream.read(&mut buffer).unwrap();
                        if read == 0 {
                            break;
                        }
                        request.extend_from_slice(&buffer[..read]);
                    }
                    let response = handler(handled);
                    let _ = stream.write_all(&response);
                    handled += 1;
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("测试 HTTP 服务接收请求失败：{error}"),
            }
        }
    });
    (format!("http://{address}"), handle)
}

fn local_options(max_bytes: usize) -> DownloadOptions {
    DownloadOptions {
        max_bytes,
        timeout: Duration::from_secs(1),
        max_redirects: 5,
        https_only: false,
    }
}

fn same_origin_validator(base_url: &str) -> UrlValidator {
    let expected_origin = Url::parse(base_url).unwrap().origin();
    Arc::new(move |url| {
        if url.origin() == expected_origin {
            Ok(())
        } else {
            Err("测试下载地址不受信任".to_string())
        }
    })
}

fn run_download(
    url: &str,
    options: DownloadOptions,
    validator: UrlValidator,
) -> Result<Vec<u8>, String> {
    tauri::async_runtime::block_on(download_cover_with_options(url, options, validator))
}

#[test]
fn accepts_only_https_steamgriddb_hosts() {
    assert!(validate_trusted_url(
        &Url::parse("https://cdn2.steamgriddb.com/file/cover.png").unwrap()
    )
    .is_ok());
    assert!(validate_trusted_url(
        &Url::parse("http://cdn2.steamgriddb.com/file/cover.png").unwrap()
    )
    .is_err());
    assert!(validate_trusted_url(
        &Url::parse("https://steamgriddb.com.example.invalid/cover.png").unwrap()
    )
    .is_err());
}

#[test]
fn times_out_when_the_server_is_too_slow() {
    let _guard = lock_http_tests();
    let (base_url, server) = spawn_server(1, |_| {
        thread::sleep(Duration::from_millis(150));
        b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\nConnection: close\r\n\r\nx".to_vec()
    });
    let mut options = local_options(16);
    options.timeout = Duration::from_millis(30);

    let error = run_download(
        &format!("{base_url}/slow"),
        options,
        same_origin_validator(&base_url),
    )
    .unwrap_err();

    assert_eq!(error, "下载联网封面超时，请稍后再试");
    server.join().unwrap();
}

#[test]
fn rejects_redirects_to_an_untrusted_origin() {
    let _guard = lock_http_tests();
    let (base_url, server) = spawn_server(1, |_| {
        b"HTTP/1.1 302 Found\r\nLocation: http://example.invalid/cover.png\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec()
    });

    let result = run_download(
        &format!("{base_url}/redirect"),
        local_options(16),
        same_origin_validator(&base_url),
    );

    assert!(result.is_err());
    server.join().unwrap();
}

#[test]
fn rejects_too_many_redirects() {
    let _guard = lock_http_tests();
    let (base_url, server) = spawn_server(3, |_| {
        b"HTTP/1.1 302 Found\r\nLocation: /loop\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            .to_vec()
    });
    let mut options = local_options(16);
    options.max_redirects = 2;

    let result = run_download(
        &format!("{base_url}/loop"),
        options,
        same_origin_validator(&base_url),
    );

    assert!(result.is_err());
    server.join().unwrap();
}

#[test]
fn rejects_a_declared_body_over_the_size_limit() {
    let _guard = lock_http_tests();
    let (base_url, server) = spawn_server(1, |_| {
        b"HTTP/1.1 200 OK\r\nContent-Length: 9\r\nConnection: close\r\n\r\n123456789".to_vec()
    });

    let error = run_download(
        &format!("{base_url}/large"),
        local_options(8),
        same_origin_validator(&base_url),
    )
    .unwrap_err();

    assert_eq!(error, "联网封面文件不能超过 10 MB");
    server.join().unwrap();
}

#[test]
fn rejects_a_streamed_body_over_the_size_limit() {
    let _guard = lock_http_tests();
    let (base_url, server) = spawn_server(1, |_| {
        b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n123456789".to_vec()
    });

    let error = run_download(
        &format!("{base_url}/streamed-large"),
        local_options(8),
        same_origin_validator(&base_url),
    )
    .unwrap_err();

    assert_eq!(error, "联网封面文件不能超过 10 MB");
    server.join().unwrap();
}
