// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use super::super::DEVICES;
use super::super::channel::{Channel, Side};
use super::super::handshake::accept_key;
use super::super::head::{Head, Refused};
use super::super::pages::{escaped, listed, serve};
use super::{Device, WRITE_TIMEOUT, finish, refuse};
use std::io::{self, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;

pub(super) struct Shared {
    pub(super) address: SocketAddr,
    pub(super) devices: Vec<Device>,
    pub(super) root: Option<PathBuf>,
}

impl Shared {
    pub(super) fn about(&self, name: &str) -> String {
        let mut page = format!(
            "<!doctype html>\n<meta charset=\"utf-8\">\n<title>cheri-vm \
             device</title>\n<h1>{}</h1>\n",
            escaped(&format!("ws://{}{DEVICES}{name}", self.address))
        );
        page.push_str(match self.device(name) {
            Some(_) => {
                "<p>This is the address of a device of the machine that \
                 cheri-vm is running. A page reaches it with a WebSocket; a \
                 browser that opens it sees this note.</p>\n"
            }
            None => "<p>The machine has no device at this address.</p>\n",
        });
        let list = |title: &str, items: Vec<(String, String)>| {
            if items.is_empty() {
                return String::new();
            }
            let items: String = items
                .iter()
                .map(|(link, text)| match link.as_str() {
                    "" => format!("<li><code>{}</code></li>\n", escaped(text)),
                    link => format!(
                        "<li><a href=\"{}\">{}</a></li>\n",
                        escaped(link),
                        escaped(text)
                    ),
                })
                .collect();
            format!("<h2>{title}</h2>\n<ul>\n{items}</ul>\n")
        };
        page.push_str(&list(
            "Pages",
            listed(self.root.as_deref())
                .into_iter()
                .map(|name| (format!("/{name}"), name))
                .collect(),
        ));
        page.push_str(&list(
            "Devices",
            self.devices
                .iter()
                .map(|device| {
                    (
                        String::new(),
                        format!(
                            "ws://{}{DEVICES}{}",
                            self.address, device.name
                        ),
                    )
                })
                .collect(),
        ));
        page
    }

    pub(super) fn device(&self, name: &str) -> Option<&Device> {
        self.devices.iter().find(|device| device.name == name)
    }

    pub(super) fn handle(&self, mut stream: TcpStream) -> io::Result<()> {
        stream.set_nodelay(true)?;
        stream.set_write_timeout(Some(WRITE_TIMEOUT))?;
        let large = "Request Header Fields Too Large";
        let (head, leftover) = match Head::read(&mut stream)? {
            Ok(read) => read,
            Err(Refused::TooLarge) => {
                return refuse(stream, 431, large, "", None);
            }
            Err(Refused::Bad) => {
                return refuse(stream, 400, "Bad Request", "", None);
            }
        };
        let Some(host) = head.header("host").filter(|host| self.is_host(host))
        else {
            return refuse(stream, 403, "Forbidden", "", None);
        };
        if head.first[0] != "GET" {
            let allow = "Allow: GET\r\n";
            return refuse(stream, 405, "Method Not Allowed", allow, None);
        }
        let path = head.first[1].split('?').next().unwrap_or_default();
        let Some(name) = path.strip_prefix(DEVICES) else {
            serve(&mut stream, self.root.as_deref(), host, path)?;
            finish(stream);
            return Ok(());
        };
        let about = self.about(name);
        let Some(device) = self.device(name) else {
            return refuse(stream, 404, "Not Found", "", Some(&about));
        };
        let upgrade = "Upgrade: websocket\r\n";
        if !head.has("upgrade", "websocket")
            || !head.has("connection", "upgrade")
        {
            let reason = "Upgrade Required";
            return refuse(stream, 426, reason, upgrade, Some(&about));
        }
        if head
            .header("origin")
            .is_some_and(|origin| origin != format!("http://{host}"))
        {
            return refuse(stream, 403, "Forbidden", "", None);
        }
        if head.header("sec-websocket-version") != Some("13") {
            let version = "Sec-WebSocket-Version: 13\r\n";
            return refuse(stream, 426, "Upgrade Required", version, None);
        }
        let Some(key) = head
            .header("sec-websocket-key")
            .filter(|key| key.len() == 24)
        else {
            return refuse(stream, 400, "Bad Request", "", None);
        };
        write!(
            stream,
            "HTTP/1.1 101 Switching Protocols\r\nUpgrade: \
             websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: \
             {}\r\n\r\n",
            accept_key(key)
        )?;
        stream.flush()?;
        let page = Channel::new(stream, Side::Server, leftover)?;
        device.serve(page)
    }

    pub(super) fn is_host(&self, host: &str) -> bool {
        let port = self.address.port();
        let address = self.address.to_string();
        let bare = address
            .strip_suffix(&format!(":{port}"))
            .unwrap_or(&address);
        [address.as_str(), &format!("localhost:{port}")].contains(&host)
            || (port == 80 && [bare, "localhost"].contains(&host))
    }
}
