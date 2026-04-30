use std::net::IpAddr;
use std::time::Instant;

use sysinfo::Networks;

use super::Collector;

#[derive(Clone, Default)]
pub struct NetworkSnapshot {
    pub interface: String,
    pub rx_bytes_per_sec: f64,
    pub tx_bytes_per_sec: f64,
    pub rx_total: u64,
    pub tx_total: u64,
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
}

pub struct NetworkCollector {
    networks: Networks,
    last: Option<Instant>,
}

impl NetworkCollector {
    pub fn new() -> Self {
        Self {
            networks: Networks::new_with_refreshed_list(),
            last: None,
        }
    }
}

impl Default for NetworkCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector for NetworkCollector {
    type Snapshot = Vec<NetworkSnapshot>;

    fn collect(&mut self) -> Option<Vec<NetworkSnapshot>> {
        self.networks.refresh(true);
        let now = Instant::now();
        let dt = self
            .last
            .map(|t| now.duration_since(t).as_secs_f64().max(0.001))
            .unwrap_or(1.0);
        self.last = Some(now);

        let mut out: Vec<NetworkSnapshot> = self
            .networks
            .list()
            .iter()
            .map(|(name, data)| {
                let mut ipv4 = None;
                let mut ipv6 = None;
                for net in data.ip_networks() {
                    match net.addr {
                        IpAddr::V4(a) => {
                            if ipv4.is_none() {
                                ipv4 = Some(a.to_string());
                            }
                        }
                        IpAddr::V6(a) => {
                            if ipv6.is_none() {
                                ipv6 = Some(a.to_string());
                            }
                        }
                    }
                }
                NetworkSnapshot {
                    interface: name.clone(),
                    rx_bytes_per_sec: data.received() as f64 / dt,
                    tx_bytes_per_sec: data.transmitted() as f64 / dt,
                    rx_total: data.total_received(),
                    tx_total: data.total_transmitted(),
                    ipv4,
                    ipv6,
                }
            })
            .collect();
        out.sort_by(|a, b| a.interface.cmp(&b.interface));
        Some(out)
    }

    fn name(&self) -> &'static str {
        "network"
    }
}
