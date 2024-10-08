mod error;

pub use self::error::{Error, Result};

use mdns_sd::{Receiver, ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::sync::Arc;
use strum::{Display, EnumString};

const SERVICE_TYPE: &'static str = "_lararium._udp.local.";
const PORT: u16 = 10101;

pub struct Discovery {
    service_daemon: Arc<ServiceDaemon>,
    services: HashMap<String, Service>,
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
            services: HashMap::new(),
        })
    }

    pub fn register(
        &self,
        hostname: &str,
        service_type: ServiceType,
    ) -> Result<Registration> {
        let service_hostname = format!("{hostname}{SERVICE_TYPE}");
        let properties = &[("service_type", service_type)];
        let service_info = ServiceInfo::new(
            SERVICE_TYPE,
            &format!("{hostname}"),
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

    pub async fn listen(&mut self) -> Result<()> {
        let receiver = self.service_daemon.browse(SERVICE_TYPE)?;
        while let Ok(event) = receiver.recv_async().await {
            match event {
                ServiceEvent::ServiceFound(service_type, fullname) => {
                    tracing::debug!("Found service: {} ({})", fullname, service_type);
                }
                ServiceEvent::ServiceResolved(info) => {
                    let Some(service_type) = info.get_property_val_str("service_type") else {
                        continue;
                    };
                    let Ok(service_type) = service_type.parse::<ServiceType>() else {
                        continue;
                    };
                    self.services.insert(
                        info.get_hostname().into(),
                        Service {
                            service_type,
                            addresses: info.get_addresses().clone(),
                        },
                    );
                    tracing::debug!(
                        "Resolved service: {} ({})",
                        info.get_fullname(),
                        info.get_type(),
                    );
                }
                ServiceEvent::ServiceRemoved(service_type, fullname) => {
                    tracing::debug!("Removed service: {} ({})", fullname, service_type);
                }
                _ => (),
            }
        }
        Ok(())
    }
}

impl Drop for Registration {
    fn drop(&mut self) {
        let _ = self.service_daemon.unregister(&self.service_fullname);
    }
}

//
//
//
//    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel();
//    ctrlc::set_handler(move || {
//        mdns.unregister(&service_fullname).unwrap();
//        let _ = shutdown_tx.send(());
//    }).unwrap();
//    shutdown_rx.recv().unwrap();
//}
