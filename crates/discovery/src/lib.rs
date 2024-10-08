mod error;

pub use self::error::{Error, Result};

use futures::{stream, Stream};
use mdns_sd::{Receiver, ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::pin::Pin;
use std::sync::Arc;
use strum::{Display, EnumString};
use tokio::sync::Mutex;

const SERVICE_TYPE: &'static str = "_lararium._udp.local.";
const PORT: u16 = 10101;

pub struct Discovery {
    service_daemon: Arc<ServiceDaemon>,
    services: Arc<Mutex<HashMap<String, Service>>>,
}

#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    ServiceFound {
        name: String,
    },
    ServiceResolved {
        name: String,
        service_type: ServiceType,
    },
    ServiceRemoved {
        name: String,
        service_type: ServiceType,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, Hash)]
#[strum(serialize_all = "snake_case")]
pub enum ServiceType {
    Server,
    Station,
}

#[derive(Debug, Clone)]
struct Service {
    service_type: ServiceType,
    addresses: HashSet<IpAddr>,
}

pub struct Registration {
    service_daemon: Arc<ServiceDaemon>,
    service_fullname: String,
}

pub struct Listener {
    receiver: Receiver<ServiceEvent>,
}

impl Discovery {
    pub fn new() -> Result<Self> {
        Ok(Self {
            service_daemon: ServiceDaemon::new()?.into(),
            services: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn register(
        &self,
        hostname: &str,
        service_type: ServiceType,
    ) -> Result<Registration> {
        let service_hostname = format!("{hostname}.{SERVICE_TYPE}");
        let properties = &[("service_type", service_type)];
        let service_info = ServiceInfo::new(
            SERVICE_TYPE,
            hostname,
            &service_hostname,
            "",
            PORT,
            &properties[..],
        )?
        .enable_addr_auto();
        let service_fullname = service_info.get_fullname().into();
        self.service_daemon.register(service_info)?;
        Ok(Registration {
            service_daemon: self.service_daemon.clone(),
            service_fullname,
        })
    }

    pub fn listen(&self) -> Result<Pin<Box<dyn Stream<Item = Result<DiscoveryEvent>> + Send>>> {
        let receiver = self.service_daemon.browse(SERVICE_TYPE)?;
        let services = self.services.clone();
        let stream = stream::unfold(receiver, move |receiver| {
            let services = services.clone();
            async move {
                loop {
                    match receiver.recv_async().await {
                        Ok(event) => match handle_event(event, &services).await {
                            Ok(Some(result)) => return Some((Ok(result), receiver)),
                            Ok(None) => continue,
                            Err(e) => return Some((Err(e), receiver)),
                        },
                        Err(_) => return None,
                    }
                }
            }
        });
        Ok(Box::pin(stream))
    }
}

async fn handle_event(
    event: ServiceEvent,
    services: &Arc<tokio::sync::Mutex<HashMap<String, Service>>>,
) -> Result<Option<DiscoveryEvent>> {
    match event {
        ServiceEvent::ServiceFound(service_type, fullname) => {
            if service_type != SERVICE_TYPE {
                return Ok(None);
            };
            let Some(name) = fullname.strip_suffix(&format!(".{SERVICE_TYPE}")) else {
                return Ok(None);
            };
            tracing::debug!("Found service: {} ({})", name, service_type);
            Ok(Some(DiscoveryEvent::ServiceFound { name: name.into() }))
        }
        ServiceEvent::ServiceResolved(info) => {
            let Some(name) = info
                .get_fullname()
                .strip_suffix(&format!(".{SERVICE_TYPE}"))
            else {
                return Ok(None);
            };
            if let Some(service_type) = info.get_property_val_str("service_type") {
                if let Ok(service_type) = service_type.parse::<ServiceType>() {
                    let mut services = services.lock().await;
                    services.insert(
                        name.into(),
                        Service {
                            service_type,
                            addresses: info.get_addresses().clone(),
                        },
                    );
                    tracing::debug!("Resolved service: {} ({})", name, info.get_type(),);
                    Ok(Some(DiscoveryEvent::ServiceResolved {
                        name: name.into(),
                        service_type,
                    }))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        }
        ServiceEvent::ServiceRemoved(service_type, fullname) => {
            if service_type != SERVICE_TYPE {
                return Ok(None);
            };
            let Some(name) = fullname.strip_suffix(&format!(".{SERVICE_TYPE}")) else {
                return Ok(None);
            };
            let Some(Service { service_type, .. }) = services.lock().await.remove(name) else {
                return Ok(None);
            };
            tracing::debug!("Removed service: {} ({})", name, service_type);
            Ok(Some(DiscoveryEvent::ServiceRemoved {
                name: name.into(),
                service_type,
            }))
        }
        _ => Ok(None),
    }
}

impl Drop for Registration {
    fn drop(&mut self) {
        let _ = self.service_daemon.unregister(&self.service_fullname);
    }
}
