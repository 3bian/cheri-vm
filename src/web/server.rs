// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

mod device;
mod server_settings;
mod shared;
mod slot;

use device::Device;
pub use server_settings::ServerSettings;
use shared::Shared;
use slot::Slot;

use super::DEVICES;

use super::pages::reply;

use std::io::{self, Read};

use std::net::{Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream};

use std::sync::atomic::{AtomicBool, Ordering};

use std::sync::{Arc, Mutex};

use std::time::Duration;

const WRITE_TIMEOUT: Duration = Duration::from_secs(10);

pub struct Server {
    address: SocketAddr,
    ends: Vec<(String, TcpStream)>,
    stop: Arc<AtomicBool>,
}

impl Server {
    pub fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn device_url(&self, device: &str) -> String {
        format!("ws://{}{DEVICES}{device}", self.address)
    }

    pub fn start(settings: ServerSettings) -> io::Result<Self> {
        let wanted = SocketAddr::new(settings.host, settings.port);
        let listener = TcpListener::bind(wanted).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!(
                    "could not listen on {wanted}: {error}; --web-port \
                     chooses another port, and 0 takes a free one"
                ),
            )
        })?;
        let address = listener.local_addr()?;
        let mut ends = Vec::new();
        let mut devices = Vec::new();
        for name in settings.devices {
            let (near, far) = pair()?;
            ends.push((name.clone(), near));
            devices.push(Device {
                name,
                slot: Mutex::new(Slot::Free(far)),
            });
        }
        let shared = Arc::new(Shared {
            address,
            devices,
            root: settings.root,
        });
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        std::thread::Builder::new().name("web".to_owned()).spawn(
            move || {
                for stream in listener.incoming() {
                    if stopped.load(Ordering::Relaxed) {
                        break;
                    }
                    let Ok(stream) = stream else {
                        continue;
                    };
                    let shared = shared.clone();
                    let _ = std::thread::Builder::new()
                        .name("web connection".to_owned())
                        .spawn(move || {
                            let _ = shared.handle(stream);
                        });
                }
            },
        )?;
        Ok(Self {
            address,
            ends,
            stop,
        })
    }

    pub fn take(&mut self, device: &str) -> Option<TcpStream> {
        let index = self.ends.iter().position(|(name, _)| name == device)?;
        Some(self.ends.remove(index).1)
    }

    pub fn url(&self) -> String {
        format!("http://{}/", self.address)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = TcpStream::connect_timeout(
            &self.address,
            Duration::from_millis(200),
        );
    }
}

fn finish(mut stream: TcpStream) {
    let _ = stream.shutdown(Shutdown::Write);
    let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
    let mut sink = [0_u8; 4096];
    let mut drained = 0;
    while drained < 64 * 1024 {
        match stream.read(&mut sink) {
            Ok(0) | Err(_) => break,
            Ok(count) => drained += count,
        }
    }
}

fn pair() -> io::Result<(TcpStream, TcpStream)> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
    let near = TcpStream::connect(listener.local_addr()?)?;
    let from = near.local_addr()?;
    loop {
        let (far, peer) = listener.accept()?;
        if peer == from {
            near.set_nodelay(true)?;
            far.set_nodelay(true)?;
            return Ok((near, far));
        }
    }
}

fn refuse(
    mut stream: TcpStream,
    status: u16,
    reason: &str,
    headers: &str,
    page: Option<&str>,
) -> io::Result<()> {
    reply(&mut stream, status, reason, headers, page)?;
    finish(stream);
    Ok(())
}
