// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use std::io::{self, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

const INDEX: &str = "index.html";
const MAX_PAGE: u64 = 4 * 1024 * 1024;

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js" | "mjs") => "text/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("ico") => "image/x-icon",
        Some("txt") => "text/plain; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}

pub(crate) fn escaped(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn file(root: &Path, path: &str) -> Option<PathBuf> {
    let relative = path.strip_prefix('/')?;
    let plain = relative.split('/').enumerate().all(|(index, part)| {
        let last = index + 1 == relative.split('/').count();
        (last && part.is_empty())
            || (!part.is_empty()
                && !part.starts_with('.')
                && part.bytes().all(|byte| {
                    byte.is_ascii_alphanumeric() || b"._-".contains(&byte)
                }))
    });
    if !plain {
        return None;
    }
    let root = std::fs::canonicalize(root).ok()?;
    let mut found = std::fs::canonicalize(root.join(relative)).ok()?;
    if found.is_dir() {
        found = std::fs::canonicalize(found.join(INDEX)).ok()?;
    }
    let small = std::fs::metadata(&found)
        .is_ok_and(|metadata| metadata.is_file() && metadata.len() <= MAX_PAGE);
    (small && found.starts_with(&root)).then_some(found)
}

pub(crate) fn listed(root: Option<&Path>) -> Vec<String> {
    let mut pages: Vec<String> = root
        .and_then(|root| std::fs::read_dir(root).ok())
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.ends_with(".html") && !name.starts_with('.'))
        .collect();
    pages.sort();
    pages
}

pub(crate) fn reply(
    stream: &mut TcpStream,
    status: u16,
    reason: &str,
    headers: &str,
    page: Option<&str>,
) -> io::Result<()> {
    let (kind, body) = match page {
        Some(page) => ("text/html", page.to_owned()),
        None => ("text/plain", format!("{reason}\n")),
    };
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {kind}; \
         charset=utf-8\r\nContent-Length: {}\r\nX-Content-Type-Options: \
         nosniff\r\nCache-Control: no-store\r\n{headers}Connection: \
         close\r\n\r\n{body}",
        body.len()
    )?;
    stream.flush()
}

pub(crate) fn serve(
    stream: &mut TcpStream,
    root: Option<&Path>,
    host: &str,
    path: &str,
) -> io::Result<()> {
    let found = root.and_then(|root| file(root, path));
    let Some((found, body)) = found
        .and_then(|found| Some((std::fs::read(&found).ok()?, found)))
        .map(|(body, found)| (found, body))
    else {
        return reply(stream, 404, "Not Found", "", None);
    };
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: \
         {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: \
         nosniff\r\nContent-Security-Policy: default-src 'none'; script-src \
         'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src \
         'self' data:; font-src 'self'; connect-src 'self' ws://{host}; \
         base-uri 'none'; form-action 'none'; frame-ancestors \
         'none'\r\nReferrer-Policy: no-referrer\r\nConnection: close\r\n\r\n",
        content_type(&found),
        body.len()
    )?;
    stream.write_all(&body)?;
    stream.flush()
}
