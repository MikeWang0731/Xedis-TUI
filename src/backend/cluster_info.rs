#[derive(Debug, Clone, PartialEq)]
pub struct ClusterNode {
    pub id: String,
    pub raw_id: String,
    pub address: String,
    pub cport: u16,
    pub role: String, // "Master" or "Replica"
    pub master_id: Option<String>,
    pub is_healthy: bool,
    pub ping_ms: f64,
    pub slots_raw: String,
    pub slot_ranges: Vec<(u16, u16)>,
    pub slot_count: u16,
    pub key_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterShard {
    pub shard_index: usize,
    pub master: ClusterNode,
    pub replicas: Vec<ClusterNode>,
    pub slot_ranges: Vec<(u16, u16)>,
    pub total_slots: u16,
    pub key_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SlaveNodeInfo {
    pub ip: String,
    pub port: u16,
    pub state: String,
    pub offset: u64,
    pub lag: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplicationInfo {
    pub role: String, // "master" or "slave"
    pub connected_slaves: usize,
    pub slaves: Vec<SlaveNodeInfo>,
    pub master_host: Option<String>,
    pub master_port: Option<u16>,
    pub master_link_status: Option<String>, // "up" or "down"
    pub master_repl_offset: u64,
}

impl Default for ReplicationInfo {
    fn default() -> Self {
        Self {
            role: "master".to_string(),
            connected_slaves: 2,
            slaves: vec![
                SlaveNodeInfo {
                    ip: "127.0.0.1".to_string(),
                    port: 6380,
                    state: "online".to_string(),
                    offset: 142980,
                    lag: 0,
                },
                SlaveNodeInfo {
                    ip: "127.0.0.1".to_string(),
                    port: 6381,
                    state: "online".to_string(),
                    offset: 142980,
                    lag: 1,
                },
            ],
            master_host: None,
            master_port: None,
            master_link_status: None,
            master_repl_offset: 142980,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterTopology {
    pub shards: Vec<ClusterShard>,
    pub standalone_nodes: Vec<ClusterNode>,
    pub total_nodes: usize,
    pub healthy_nodes: usize,
    pub covered_slots: u16,
    pub is_fully_covered: bool,
    pub is_cluster: bool,
    pub replication: Option<ReplicationInfo>,
}

impl Default for ClusterTopology {
    fn default() -> Self {
        Self::mock_cluster_topology()
    }
}

impl ClusterTopology {
    pub fn mock_cluster_topology() -> Self {
        let master1 = ClusterNode {
            id: "node-1".to_string(),
            raw_id: "e01a1b2c3d4e5f60718293a4b5c6d7e8f9012345".to_string(),
            address: "127.0.0.1:6379".to_string(),
            cport: 16379,
            role: "Master".to_string(),
            master_id: None,
            is_healthy: true,
            ping_ms: 0.38,
            slots_raw: "0-5460".to_string(),
            slot_ranges: vec![(0, 5460)],
            slot_count: 5461,
            key_count: 402_830,
        };

        let replica1 = ClusterNode {
            id: "node-4".to_string(),
            raw_id: "e04a1b2c3d4e5f60718293a4b5c6d7e8f9012345".to_string(),
            address: "127.0.0.1:6382".to_string(),
            cport: 16382,
            role: "Replica".to_string(),
            master_id: Some("node-1".to_string()),
            is_healthy: true,
            ping_ms: 0.45,
            slots_raw: "replica-of node-1".to_string(),
            slot_ranges: Vec::new(),
            slot_count: 0,
            key_count: 402_830,
        };

        let master2 = ClusterNode {
            id: "node-2".to_string(),
            raw_id: "e02a1b2c3d4e5f60718293a4b5c6d7e8f9012345".to_string(),
            address: "127.0.0.1:6380".to_string(),
            cport: 16380,
            role: "Master".to_string(),
            master_id: None,
            is_healthy: true,
            ping_ms: 0.41,
            slots_raw: "5461-10922".to_string(),
            slot_ranges: vec![(5461, 10922)],
            slot_count: 5462,
            key_count: 398_410,
        };

        let replica2 = ClusterNode {
            id: "node-5".to_string(),
            raw_id: "e05a1b2c3d4e5f60718293a4b5c6d7e8f9012345".to_string(),
            address: "127.0.0.1:6383".to_string(),
            cport: 16383,
            role: "Replica".to_string(),
            master_id: Some("node-2".to_string()),
            is_healthy: true,
            ping_ms: 0.50,
            slots_raw: "replica-of node-2".to_string(),
            slot_ranges: Vec::new(),
            slot_count: 0,
            key_count: 398_410,
        };

        let master3 = ClusterNode {
            id: "node-3".to_string(),
            raw_id: "e03a1b2c3d4e5f60718293a4b5c6d7e8f9012345".to_string(),
            address: "127.0.0.1:6381".to_string(),
            cport: 16381,
            role: "Master".to_string(),
            master_id: None,
            is_healthy: true,
            ping_ms: 0.35,
            slots_raw: "10923-16383".to_string(),
            slot_ranges: vec![(10923, 16383)],
            slot_count: 5461,
            key_count: 407_250,
        };

        let replica3 = ClusterNode {
            id: "node-6".to_string(),
            raw_id: "e06a1b2c3d4e5f60718293a4b5c6d7e8f9012345".to_string(),
            address: "127.0.0.1:6384".to_string(),
            cport: 16384,
            role: "Replica".to_string(),
            master_id: Some("node-3".to_string()),
            is_healthy: true,
            ping_ms: 0.48,
            slots_raw: "replica-of node-3".to_string(),
            slot_ranges: Vec::new(),
            slot_count: 0,
            key_count: 407_250,
        };

        let shards = vec![
            ClusterShard {
                shard_index: 1,
                master: master1,
                replicas: vec![replica1],
                slot_ranges: vec![(0, 5460)],
                total_slots: 5461,
                key_count: 402_830,
            },
            ClusterShard {
                shard_index: 2,
                master: master2,
                replicas: vec![replica2],
                slot_ranges: vec![(5461, 10922)],
                total_slots: 5462,
                key_count: 398_410,
            },
            ClusterShard {
                shard_index: 3,
                master: master3,
                replicas: vec![replica3],
                slot_ranges: vec![(10923, 16383)],
                total_slots: 5461,
                key_count: 407_250,
            },
        ];

        Self {
            shards,
            standalone_nodes: Vec::new(),
            total_nodes: 6,
            healthy_nodes: 6,
            covered_slots: 16384,
            is_fully_covered: true,
            is_cluster: true,
            replication: None,
        }
    }

    #[allow(dead_code)]
    pub fn mock_standalone_topology() -> Self {
        let node = ClusterNode {
            id: "standalone".to_string(),
            raw_id: "local_standalone_instance_001".to_string(),
            address: "127.0.0.1:6379".to_string(),
            cport: 0,
            role: "Master".to_string(),
            master_id: None,
            is_healthy: true,
            ping_ms: 0.38,
            slots_raw: "All keys (Standalone DB 0~15)".to_string(),
            slot_ranges: Vec::new(),
            slot_count: 0,
            key_count: 1_208_490,
        };

        Self {
            shards: Vec::new(),
            standalone_nodes: vec![node],
            total_nodes: 1,
            healthy_nodes: 1,
            covered_slots: 0,
            is_fully_covered: true,
            is_cluster: false,
            replication: Some(ReplicationInfo::default()),
        }
    }
}

pub struct ClusterTopologyParser;

impl ClusterTopologyParser {
    pub fn parse_cluster_nodes(raw_str: &str, default_ping_ms: f64) -> ClusterTopology {
        let mut raw_nodes = Vec::new();
        let mut node_map = std::collections::HashMap::new();

        for line in raw_str.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 8 {
                continue;
            }

            let raw_id = parts[0].to_string();
            let short_id = if raw_id.len() > 8 {
                raw_id[..8].to_string()
            } else {
                raw_id.clone()
            };

            let addr_part = parts[1];
            let (address, cport) = if let Some((ip_port, cport_str)) = addr_part.split_once('@') {
                let cport = cport_str.parse::<u16>().unwrap_or(0);
                (ip_port.to_string(), cport)
            } else {
                (addr_part.to_string(), 0)
            };

            let flags = parts[2];
            let is_master = flags.contains("master");
            let role = if is_master { "Master" } else { "Replica" }.to_string();

            let master_id = if parts[3] != "-" && !parts[3].is_empty() {
                Some(parts[3].to_string())
            } else {
                None
            };

            let link_state = parts[7];
            let is_healthy = link_state == "connected" && !flags.contains("fail");

            let mut slot_ranges = Vec::new();
            let mut total_slots = 0u16;
            let mut slots_raw = String::new();

            if parts.len() > 8 {
                let slot_parts = &parts[8..];
                slots_raw = slot_parts.join(" ");
                for slot_token in slot_parts {
                    // Ignore importing/migrating slot syntax like [123->-node]
                    if slot_token.starts_with('[') {
                        continue;
                    }
                    if let Some((start_s, end_s)) = slot_token.split_once('-') {
                        if let (Ok(s), Ok(e)) = (start_s.parse::<u16>(), end_s.parse::<u16>()) {
                            if e >= s {
                                slot_ranges.push((s, e));
                                total_slots += e - s + 1;
                            }
                        }
                    } else if let Ok(s) = slot_token.parse::<u16>() {
                        slot_ranges.push((s, s));
                        total_slots += 1;
                    }
                }
            } else if !is_master {
                slots_raw = format!("replica of {}", parts[3].chars().take(8).collect::<String>());
            }

            let node = ClusterNode {
                id: short_id.clone(),
                raw_id: raw_id.clone(),
                address,
                cport,
                role,
                master_id,
                is_healthy,
                ping_ms: default_ping_ms,
                slots_raw,
                slot_ranges,
                slot_count: total_slots,
                key_count: 0,
            };

            node_map.insert(raw_id.clone(), node.clone());
            node_map.insert(short_id, node.clone());
            raw_nodes.push(node);
        }

        // Group into Shards deterministically sorted by slot range start and address
        let mut shards = Vec::new();
        let mut masters: Vec<ClusterNode> = raw_nodes.iter().filter(|n| n.role == "Master").cloned().collect();

        // Sort masters by primary slot range start or address to prevent jumping across poll ticks
        masters.sort_by(|a, b| {
            let a_slot = a.slot_ranges.first().map(|(s, _)| *s).unwrap_or(u16::MAX);
            let b_slot = b.slot_ranges.first().map(|(s, _)| *s).unwrap_or(u16::MAX);
            a_slot.cmp(&b_slot).then_with(|| a.address.cmp(&b.address))
        });

        for (idx, master) in masters.into_iter().enumerate() {
            let master_full_id = master.raw_id.clone();
            let master_short_id = master.id.clone();

            let mut replicas: Vec<ClusterNode> = raw_nodes
                .iter()
                .filter(|n| {
                    n.role == "Replica"
                        && (n.master_id.as_deref() == Some(&master_full_id)
                            || n.master_id.as_deref() == Some(&master_short_id))
                })
                .cloned()
                .collect();

            // Sort replicas deterministically by address
            replicas.sort_by(|a, b| a.address.cmp(&b.address));

            let shard_slot_ranges = master.slot_ranges.clone();
            let shard_total_slots = master.slot_count;

            shards.push(ClusterShard {
                shard_index: idx + 1,
                master,
                replicas,
                slot_ranges: shard_slot_ranges,
                total_slots: shard_total_slots,
                key_count: 0,
            });
        }

        let total_nodes = raw_nodes.len();
        let healthy_nodes = raw_nodes.iter().filter(|n| n.is_healthy).count();
        let covered_slots: u16 = shards.iter().map(|s| s.total_slots).sum();
        let is_fully_covered = covered_slots == 16384;

        ClusterTopology {
            shards,
            standalone_nodes: Vec::new(),
            total_nodes,
            healthy_nodes,
            covered_slots,
            is_fully_covered,
            is_cluster: true,
            replication: None,
        }
    }

    pub fn parse_info_replication(raw_str: &str) -> ReplicationInfo {
        let mut role = "master".to_string();
        let mut connected_slaves = 0usize;
        let mut slaves = Vec::new();
        let mut master_host = None;
        let mut master_port = None;
        let mut master_link_status = None;
        let mut master_repl_offset = 0u64;

        for line in raw_str.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((k, v)) = line.split_once(':') {
                match k {
                    "role" => role = v.to_string(),
                    "connected_slaves" => connected_slaves = v.parse().unwrap_or(0),
                    "master_host" => master_host = Some(v.to_string()),
                    "master_port" => master_port = v.parse().ok(),
                    "master_link_status" => master_link_status = Some(v.to_string()),
                    "master_repl_offset" => master_repl_offset = v.parse().unwrap_or(0),
                    _ if k.starts_with("slave") => {
                        // slave0:ip=127.0.0.1,port=6380,state=online,offset=1234,lag=0
                        let mut ip = "127.0.0.1".to_string();
                        let mut port = 6379u16;
                        let mut state = "online".to_string();
                        let mut offset = 0u64;
                        let mut lag = 0u64;

                        for part in v.split(',') {
                            if let Some((sub_k, sub_v)) = part.split_once('=') {
                                match sub_k {
                                    "ip" => ip = sub_v.to_string(),
                                    "port" => port = sub_v.parse().unwrap_or(6379),
                                    "state" => state = sub_v.to_string(),
                                    "offset" => offset = sub_v.parse().unwrap_or(0),
                                    "lag" => lag = sub_v.parse().unwrap_or(0),
                                    _ => {}
                                }
                            }
                        }

                        slaves.push(SlaveNodeInfo {
                            ip,
                            port,
                            state,
                            offset,
                            lag,
                        });
                    }
                    _ => {}
                }
            }
        }

        ReplicationInfo {
            role,
            connected_slaves,
            slaves,
            master_host,
            master_port,
            master_link_status,
            master_repl_offset,
        }
    }
}
