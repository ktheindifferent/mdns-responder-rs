//! A multicast DNS (mDNS) responder library for Rust.
//!
//! This library allows you to register and advertise services on the local network
//! using multicast DNS (mDNS/Bonjour/Avahi). Services can be discovered by other
//! devices on the same network without requiring a central DNS server.
//!
//! # Example
//!
//! ```no_run
//! use mdns_responder_rs as mdns;
//!
//! # fn main() -> std::io::Result<()> {
//! let responder = mdns::Responder::new()?;
//! let _service = responder.register(
//!     "_http._tcp".to_owned(),
//!     "My Web Server".to_owned(),
//!     8080,
//!     &["path=/", "version=1.0"],
//! );
//!
//! // Service will be advertised until it goes out of scope
//! std::thread::sleep(std::time::Duration::from_secs(60));
//! # Ok(())
//! # }
//! ```

use log::{error, warn};
use tokio_core as tokio;

use dns_parser::Name;
use futures::Future;
use futures::sync::mpsc;
use std::cell::RefCell;
use std::io;
use std::sync::{Arc, RwLock, Mutex};
use std::thread;
use tokio::reactor::{Core, Handle};

mod address_family;
mod fsm;
#[cfg(windows)]
#[path = "netwin.rs"]
mod net;
#[cfg(not(windows))]
mod net;
mod services;

use crate::address_family::{Inet, Inet6};
use crate::fsm::{Command, FSM};
use crate::services::{ServiceData, Services, ServicesInner};

/// Default Time-To-Live for DNS records (in seconds)
const DEFAULT_TTL: u32 = 60;

/// Standard mDNS port number
const MDNS_PORT: u16 = 5353;

/// The main mDNS responder that manages service registration and advertisement.
///
/// The `Responder` handles all mDNS network communication and maintains a registry
/// of advertised services. It runs a background thread to handle mDNS queries and
/// responses.
pub struct Responder {
    services: Services,
    commands: RefCell<CommandSender>,
    shutdown: Arc<Shutdown>,
}

/// A handle to a registered mDNS service.
///
/// When this handle is dropped, the service will be unregistered and will stop
/// being advertised on the network. Keep this handle alive as long as you want
/// the service to be discoverable.
pub struct Service {
    id: usize,
    services: Services,
    commands: CommandSender,
    _shutdown: Arc<Shutdown>,
}

type ResponderTask = Box<dyn Future<Item = (), Error = io::Error> + Send>;

impl Responder {
    /// Internal helper to set up the tokio event loop core
    fn setup_core() -> io::Result<(Core, ResponderTask, Responder)> {
        let core = Core::new()?;
        let (responder, task) = Self::with_handle(&core.handle())?;
        Ok((core, task, responder))
    }

    /// Creates a new mDNS responder with its own background thread.
    ///
    /// This will spawn a dedicated thread for handling mDNS traffic.
    /// The responder will automatically bind to the mDNS multicast addresses
    /// for both IPv4 and IPv6 if available.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The mDNS port (5353) is already in use
    /// - Network interfaces cannot be accessed
    /// - The background thread fails to start
    pub fn new() -> io::Result<Responder> {
        let (tx, rx) = std::sync::mpsc::sync_channel(0);
        let handle = thread::Builder::new()
            .name("mdns-responder".to_owned())
            .spawn(move || match Self::setup_core() {
                Ok((mut core, task, responder)) => {
                    tx.send(Ok(responder)).expect("tx responder channel closed");
                    core.run(task).expect("mdns thread failed");
                }
                Err(err) => {
                    tx.send(Err(err)).expect("tx responder channel closed");
                }
            })?;

        let mut responder = rx.recv().expect("rx responder channel closed")?;
        if let Some(shutdown) = Arc::get_mut(&mut responder.shutdown) {
            *shutdown.thread_handle.lock().unwrap() = Some(handle);
        }
        Ok(responder)
    }

    /// Creates a new mDNS responder using an existing tokio event loop.
    ///
    /// This is useful when you already have a tokio runtime and want to
    /// integrate the mDNS responder into it.
    ///
    /// # Arguments
    ///
    /// * `handle` - A handle to the tokio reactor where tasks will be spawned
    pub fn spawn(handle: &Handle) -> io::Result<Responder> {
        let (responder, task) = Responder::with_handle(handle)?;
        handle.spawn(task.map_err(|e| {
            warn!("mdns error {e:?}");
            
        }));
        Ok(responder)
    }

    /// Creates a new mDNS responder with a custom tokio handle.
    ///
    /// Returns both the responder and the future that must be driven to
    /// handle mDNS traffic. This gives you full control over task execution.
    ///
    /// # Arguments
    ///
    /// * `handle` - A handle to the tokio reactor
    pub fn with_handle(handle: &Handle) -> io::Result<(Responder, ResponderTask)> {
        let mut hostname = net::gethostname()?;
        if !hostname.ends_with(".local") {
            hostname.push_str(".local");
        }

        let services = Arc::new(RwLock::new(ServicesInner::new(hostname)));

        let v4 = FSM::<Inet>::new(handle, &services);
        let v6 = FSM::<Inet6>::new(handle, &services);

        let (task, commands): (ResponderTask, _) = match (v4, v6) {
            (Ok((v4_task, v4_command)), Ok((v6_task, v6_command))) => {
                let task = v4_task.join(v6_task).map(|((), ())| ());
                let task = Box::new(task);

                let commands = vec![v4_command, v6_command];
                (task, commands)
            }

            (Ok((v4_task, v4_command)), Err(err)) => {
                warn!("Failed to register IPv6 receiver: {err:?}");
                (Box::new(v4_task), vec![v4_command])
            }

            (Err(err), _) => return Err(err),
        };

        let commands = CommandSender(commands);
        let responder = Responder {
            services,
            commands: RefCell::new(commands.clone()),
            shutdown: Arc::new(Shutdown {
                commands: commands.clone(),
                thread_handle: Mutex::new(None),
            }),
        };

        Ok((responder, task))
    }
}

/// Builds a properly formatted TXT record from string entries.
///
/// Each entry is prefixed with its length as required by DNS TXT records.
/// Empty entries result in a single zero byte.
fn build_txt_record(entries: &[&str]) -> Vec<u8> {
    if entries.is_empty() {
        vec![0]
    } else {
        entries.iter()
            .flat_map(|entry| {
                let bytes = entry.as_bytes();
                if bytes.len() > 255 {
                    panic!("TXT record entry '{}' is too long (max 255 bytes)", entry);
                }
                std::iter::once(bytes.len() as u8)
                    .chain(bytes.iter().cloned())
            })
            .collect()
    }
}

impl Responder {
    /// Registers a new service to be advertised via mDNS.
    ///
    /// # Arguments
    ///
    /// * `svc_type` - The service type (e.g., "_http._tcp")
    /// * `svc_name` - The human-readable service name
    /// * `port` - The port number where the service is listening
    /// * `txt` - TXT record entries as key=value pairs
    ///
    /// # Returns
    ///
    /// A `Service` handle that keeps the service registered. The service will
    /// be automatically unregistered when this handle is dropped.
    ///
    /// # Panics
    ///
    /// Panics if any TXT record entry is longer than 255 bytes.
    pub fn register(&self, svc_type: String, svc_name: String, port: u16, txt: &[&str]) -> Service {
        let txt = build_txt_record(txt);

        let svc = ServiceData {
            typ: Name::from_str(format!("{svc_type}.local"))
                .expect("Invalid service type format"),
            name: Name::from_str(format!("{svc_name}.{svc_type}.local"))
                .expect("Invalid service name format"),
            port,
            txt,
        };

        self.commands
            .borrow_mut()
            .send_unsolicited(svc.clone(), DEFAULT_TTL, true);

        let id = self.services.write().unwrap().register(svc);

        Service {
            id,
            commands: self.commands.borrow().clone(),
            services: self.services.clone(),
            _shutdown: self.shutdown.clone(),
        }
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        let svc = self.services.write().unwrap().unregister(self.id);
        self.commands.send_unsolicited(svc, 0, false);
    }
}

struct Shutdown {
    commands: CommandSender,
    thread_handle: Mutex<Option<thread::JoinHandle<()>>>,
}

impl Drop for Shutdown {
    fn drop(&mut self) {
        self.commands.clone().send_shutdown();
        // Wait for the thread to finish
        if let Some(handle) = self.thread_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }
}

#[derive(Clone)]
struct CommandSender(Vec<mpsc::UnboundedSender<Command>>);
impl CommandSender {
    fn send(&mut self, cmd: Command) {
        for tx in self.0.iter_mut() {
            if let Err(e) = tx.unbounded_send(cmd.clone()) {
                error!("Failed to send command to responder: {e:?}");
            }
        }
    }

    fn send_unsolicited(&mut self, svc: ServiceData, ttl: u32, include_ip: bool) {
        self.send(Command::SendUnsolicited {
            svc,
            ttl,
            include_ip,
        });
    }

    fn send_shutdown(&mut self) {
        self.send(Command::Shutdown);
    }
}
