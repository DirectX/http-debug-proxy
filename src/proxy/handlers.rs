use actix_web::{
    HttpRequest, HttpResponse,
    http::Method,
    route,
    web::{self, Bytes, Data},
};
use colored::Colorize;
use reqwest::Client;
use serde_json::{Value, to_string_pretty};
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};

use crate::config::Config;

#[route(
    "/{url_path:.*}",
    method = "GET",
    method = "POST",
    method = "PUT",
    method = "PATCH",
    method = "DELETE"
)]
pub async fn index(
    req: HttpRequest,
    config: Data<Config>,
    url_path: web::Path<String>,
    request_body: Bytes,
    client: Data<Client>,
    counter: Data<AtomicUsize>,
) -> HttpResponse {
    let start = Instant::now();

    let connection_id = counter.fetch_add(1, Ordering::SeqCst) + 1;

    let method = req.method().clone();
    let mut request_headers_vec = req
        .headers()
        .iter()
        .map(|(k, v)| format!("\"{}: {}\"", k, v.to_str().unwrap_or("N/A")))
        .collect::<Vec<String>>();
    request_headers_vec.sort();
    let request_headers_string = request_headers_vec.join(", ");

    let request_body_string = if let Ok(utf8_str) = std::str::from_utf8(&request_body) {
        if utf8_str.len() == 0 {
            format!("None")
        } else if let Ok(json) = serde_json::from_str::<Value>(utf8_str) {
            match to_string_pretty(&json) {
                Ok(pretty_json) => format!("JSON:\n{}", pretty_json),
                Err(_) => format!("JSON (unformatted): {}", json),
            }
        } else {
            format!("UTF-8 string: {}", utf8_str)
        }
    } else {
        format!("Binary: {:?}", &request_body)
    };

    let (upstream_name, upstream_url_path) = if url_path.len() == 0 {
        if config.upstreams.len() > 1 {
            (config.default_upstream.clone(), "/".to_string())
        } else {
            (
                Some(config.upstreams.iter().next().unwrap().0.to_string()),
                "/".to_string(),
            )
        }
    } else {
        let mut parts = url_path.splitn(2, '/');
        let prefix = parts.next().unwrap_or("").to_string();
        let suffix = parts.next().unwrap_or("").to_string();

        if config.upstreams.contains_key(&prefix) {
            (Some(prefix), format!("/{suffix}"))
        } else {
            if config.upstreams.len() > 1 {
                (config.default_upstream.clone(), format!("/{}", url_path))
            } else {
                (
                    Some(config.upstreams.iter().next().unwrap().0.to_string()),
                    format!("/{}", url_path),
                )
            }
        }
    };

    // Time mark
    let request_decoded = start.elapsed();

    if upstream_name.is_none() {
        log::error!(
            "[N/A] Request {connection_id}\n\n-> {method} {upstream_url_path}\n----------------------------------------\nRequest headers: {request_headers_string}\n----------------------------------------\nRequest data: {request_body_string}\n========================================\nUpstream not found\n"
        );
        return HttpResponse::NotFound().body("Upstream not found");
    }

    let upstream_name = upstream_name.unwrap();
    let upstream_host = config.upstreams.get(&upstream_name).unwrap().to_string();
    let upstream_url = format!("{upstream_host}{upstream_url_path}");

    let mut upstream_request = match method {
        Method::GET => client.get(&upstream_url),
        Method::POST => client.post(&upstream_url),
        Method::PUT => client.put(&upstream_url),
        Method::PATCH => client.patch(&upstream_url),
        Method::DELETE => client.delete(&upstream_url),
        _ => {
            log::error!(
                "[N/A] Request {connection_id}\n\n-> {method} {upstream_url_path}\n----------------------------------------\nRequest headers: {request_headers_string}\n----------------------------------------\nRequest data: {request_body_string}\n========================================\nMethod {method} is not supported\n"
            );
            return HttpResponse::InternalServerError()
                .body(format!("Method {} is not supported", method));
        }
    };

    // Forward original headers
    for (name, value) in req.headers() {
        if !name.as_str().starts_with("host") {
            // Skip host header as it will be set by the client
            if let Ok(header_value) = reqwest::header::HeaderValue::from_bytes(value.as_bytes()) {
                upstream_request = upstream_request.header(name.as_str(), header_value);
            }
        }
    }

    // Time mark
    let proxy_request_encodeed = start.elapsed();

    let res = upstream_request.body(request_body).send().await;

    // Time mark
    let proxy_sent = start.elapsed();

    match res {
        Ok(res) => {
            let status = res.status();
            let response_url = res.url().clone();
            let headers = res.headers().clone();

            let mut response_headers_vec = headers
                .iter()
                .map(|(k, v)| format!("\"{}: {}\"", k, v.to_str().unwrap_or("N/A")))
                .collect::<Vec<String>>();
            response_headers_vec.sort();
            let response_headers_string = response_headers_vec.join(", ");

            let response_body = res.bytes().await.unwrap_or_default();
            let response_body_string = if let Ok(utf8_str) = std::str::from_utf8(&response_body) {
                if utf8_str.len() == 0 {
                    format!("None")
                } else if let Ok(json) = serde_json::from_str::<Value>(utf8_str) {
                    match to_string_pretty(&json) {
                        Ok(pretty_json) => format!("JSON:\n{}", pretty_json),
                        Err(_) => format!("JSON (unformatted): {}", json),
                    }
                } else {
                    format!("UTF-8 string: {}", utf8_str)
                }
            } else {
                format!("Binary: {:?}", &response_body)
            };

            // Time mark
            let proxy_response_decoded = start.elapsed();

            log::info!(
                r#"[{upstream_name}] Request {connection_id}

{} {} -> {}
----------------------------------------
Request headers: {}
----------------------------------------
Request data: {}
========================================
{} <- {}
----------------------------------------
Response headers: {}
----------------------------------------
Response data: {}
"#,
                method.to_string().bright_cyan(),
                upstream_url_path.bright_white(),
                upstream_url,
                request_headers_string.green(),
                request_body_string.bright_yellow(),
                if status.as_u16() == 200 { status.to_string().bright_green() } else { status.to_string().bright_magenta() },
                response_url,
                response_headers_string.green(),
                response_body_string.bright_yellow(),
            );

            // Create response builder with the same status code
            let mut client_response = HttpResponse::build(
                actix_web::http::StatusCode::from_u16(status.as_u16()).unwrap_or_default(),
            );

            // Copy all headers from upstream response
            for (header_name, header_value) in headers.iter() {
                if let Ok(name) =
                    actix_web::http::header::HeaderName::from_bytes(header_name.as_ref())
                {
                    if let Ok(value) =
                        actix_web::http::header::HeaderValue::from_bytes(header_value.as_bytes())
                    {
                        client_response.insert_header((name, value));
                    }
                }
            }

            // Time mark
            let response_encoded = start.elapsed();
            log::debug!(
                r#"[{upstream_name}] Request {} OK

Proxy: {:?}
Total: {:?}
"#,
                connection_id,
                proxy_sent - proxy_request_encodeed,
                response_encoded,
            );

            // Return response with headers and body
            client_response.body(response_body)
        }
        Err(err) => {
            log::error!(
                "[N/A] Request {connection_id}\n\n-> {method} {upstream_url_path}\n----------------------------------------\nRequest headers: {request_headers_string}\n----------------------------------------\nRequest data: {request_body_string}\n========================================\nError proxying to upstream: {err}\n"
            );
            return HttpResponse::InternalServerError().finish();
        }
    }
}
