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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RedisTopologyMode {
    #[default]
    Standalone,
    Cluster,
    Sentinel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SentinelSlaveInfo {
    pub ip: String,
    pub port: u16,
    pub flags: String,
    pub link_status: String,
    pub repl_offset: u64,
    pub lag_sec: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SentinelPeerInfo {
    pub id: String,
    pub ip: String,
    pub port: u16,
    pub flags: String,
    pub is_healthy: bool,
    pub last_ok_ping_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SentinelMasterInfo {
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub status: String,
    pub flags: String,
    pub quorum: u32,
    pub num_slaves: usize,
    pub num_other_sentinels: usize,
    pub down_after_ms: u64,
    pub failover_timeout_ms: u64,
    pub slaves: Vec<SentinelSlaveInfo>,
    pub sentinels: Vec<SentinelPeerInfo>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SentinelTopology {
    pub masters: Vec<SentinelMasterInfo>,
    pub total_sentinels: usize,
    pub tilt_mode: bool,
    pub running_scripts: usize,
    pub my_sentinel: Option<SentinelPeerInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterTopology {
    pub mode: RedisTopologyMode,
    pub shards: Vec<ClusterShard>,
    pub standalone_nodes: Vec<ClusterNode>,
    pub total_nodes: usize,
    pub healthy_nodes: usize,
    pub covered_slots: u16,
    pub is_fully_covered: bool,
    pub is_cluster: bool,
    pub replication: Option<ReplicationInfo>,
    pub sentinel: Option<SentinelTopology>,
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
            mode: RedisTopologyMode::Cluster,
            shards,
            standalone_nodes: Vec::new(),
            total_nodes: 6,
            healthy_nodes: 6,
            covered_slots: 16384,
            is_fully_covered: true,
            is_cluster: true,
            replication: None,
            sentinel: None,
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
            mode: RedisTopologyMode::Standalone,
            shards: Vec::new(),
            standalone_nodes: vec![node],
            total_nodes: 1,
            healthy_nodes: 1,
            covered_slots: 0,
            is_fully_covered: true,
            is_cluster: false,
            replication: Some(ReplicationInfo::default()),
            sentinel: None,
        }
    }

    pub fn mock_sentinel_topology() -> Self {
        let slaves = vec![
            SentinelSlaveInfo {
                ip: "127.0.0.1".to_string(),
                port: 6380,
                flags: "slave".to_string(),
                link_status: "ok".to_string(),
                repl_offset: 142980,
                lag_sec: 0,
            },
            SentinelSlaveInfo {
                ip: "127.0.0.1".to_string(),
                port: 6381,
                flags: "slave".to_string(),
                link_status: "ok".to_string(),
                repl_offset: 142980,
                lag_sec: 1,
            },
        ];

        let sentinels = vec![
            SentinelPeerInfo {
                id: "sentinel-2".to_string(),
                ip: "127.0.0.1".to_string(),
                port: 26380,
                flags: "sentinel".to_string(),
                is_healthy: true,
                last_ok_ping_ms: 180,
            },
            SentinelPeerInfo {
                id: "sentinel-3".to_string(),
                ip: "127.0.0.1".to_string(),
                port: 26381,
                flags: "sentinel".to_string(),
                is_healthy: true,
                last_ok_ping_ms: 240,
            },
        ];

        let master = SentinelMasterInfo {
            name: "mymaster".to_string(),
            ip: "127.0.0.1".to_string(),
            port: 6379,
            status: "ok".to_string(),
            flags: "master".to_string(),
            quorum: 2,
            num_slaves: 2,
            num_other_sentinels: 2,
            down_after_ms: 30000,
            failover_timeout_ms: 180000,
            slaves,
            sentinels,
        };

        let my_sentinel = SentinelPeerInfo {
            id: "sentinel-1".to_string(),
            ip: "127.0.0.1".to_string(),
            port: 26379,
            flags: "sentinel".to_string(),
            is_healthy: true,
            last_ok_ping_ms: 0,
        };

        let sentinel_topo = SentinelTopology {
            masters: vec![master],
            total_sentinels: 3,
            tilt_mode: false,
            running_scripts: 0,
            my_sentinel: Some(my_sentinel),
        };

        let sentinel_node = ClusterNode {
            id: "sentinel-1".to_string(),
            raw_id: "sentinel_instance_26379".to_string(),
            address: "127.0.0.1:26379".to_string(),
            cport: 0,
            role: "Sentinel".to_string(),
            master_id: None,
            is_healthy: true,
            ping_ms: 0.42,
            slots_raw: "Quorum: 2 (3 in total) (Monitoring: mymaster)".to_string(),
            slot_ranges: Vec::new(),
            slot_count: 0,
            key_count: 0,
        };

        Self {
            mode: RedisTopologyMode::Sentinel,
            shards: Vec::new(),
            standalone_nodes: vec![sentinel_node],
            total_nodes: 3,
            healthy_nodes: 3,
            covered_slots: 0,
            is_fully_covered: true,
            is_cluster: false,
            replication: None,
            sentinel: Some(sentinel_topo),
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
            mode: RedisTopologyMode::Cluster,
            shards,
            standalone_nodes: Vec::new(),
            total_nodes,
            healthy_nodes,
            covered_slots,
            is_fully_covered,
            is_cluster: true,
            replication: None,
            sentinel: None,
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

    pub fn parse_info_sentinel(raw_str: &str) -> SentinelTopology {
        let mut total_sentinels = 1;
        let mut tilt_mode = false;
        let mut running_scripts = 0;
        let mut masters = Vec::new();

        for line in raw_str.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((k, v)) = line.split_once(':') {
                let k_trimmed = k.trim();
                let v_trimmed = v.trim();
                match k_trimmed {
                    "sentinel_tilt" => tilt_mode = v_trimmed == "1",
                    "sentinel_running_scripts" => running_scripts = v_trimmed.parse().unwrap_or(0),
                    _ if k_trimmed.starts_with("master") => {
                        // master0:name=mymaster,status=ok,address=127.0.0.1:6379,slaves=2,sentinels=3
                        let mut name = "mymaster".to_string();
                        let mut status = "ok".to_string();
                        let mut ip = "127.0.0.1".to_string();
                        let mut port = 6379u16;
                        let mut slaves_cnt = 0usize;
                        let mut sentinels_cnt = 1usize;

                        for part in v_trimmed.split(',') {
                            if let Some((sub_k, sub_v)) = part.split_once('=') {
                                match sub_k.trim() {
                                    "name" => name = sub_v.trim().to_string(),
                                    "status" => status = sub_v.trim().to_string(),
                                    "address" => {
                                        if let Some((addr_ip, addr_port)) = sub_v.trim().split_once(':') {
                                            ip = addr_ip.to_string();
                                            port = addr_port.parse().unwrap_or(6379);
                                        }
                                    }
                                    "slaves" => slaves_cnt = sub_v.trim().parse().unwrap_or(0),
                                    "sentinels" => {
                                        sentinels_cnt = sub_v.trim().parse().unwrap_or(1);
                                        total_sentinels = total_sentinels.max(sentinels_cnt);
                                    }
                                    _ => {}
                                }
                            }
                        }

                        masters.push(SentinelMasterInfo {
                            name,
                            ip,
                            port,
                            status,
                            flags: "master".to_string(),
                            quorum: 2,
                            num_slaves: slaves_cnt,
                            num_other_sentinels: sentinels_cnt.saturating_sub(1),
                            down_after_ms: 30000,
                            failover_timeout_ms: 180000,
                            slaves: Vec::new(),
                            sentinels: Vec::new(),
                        });
                    }
                    _ => {}
                }
            }
        }

        SentinelTopology {
            masters,
            total_sentinels,
            tilt_mode,
            running_scripts,
            my_sentinel: None,
        }
    }

    pub fn parse_sentinel_masters_entries(entries: &[std::collections::HashMap<String, String>]) -> Vec<SentinelMasterInfo> {
        let mut masters = Vec::new();
        for map in entries {
            let name = map.get("name").cloned().unwrap_or_else(|| "mymaster".to_string());
            let ip = map.get("ip").cloned().unwrap_or_else(|| "127.0.0.1".to_string());
            let port = map.get("port").and_then(|p| p.parse().ok()).unwrap_or(6379);
            let flags = map.get("flags").cloned().unwrap_or_else(|| "master".to_string());
            let status = if flags.contains("odown") {
                "odown".to_string()
            } else if flags.contains("sdown") {
                "sdown".to_string()
            } else {
                "ok".to_string()
            };
            let quorum = map.get("quorum").and_then(|q| q.parse().ok()).unwrap_or(2);
            let num_slaves = map.get("num-slaves").and_then(|s| s.parse().ok()).unwrap_or(0);
            let num_other_sentinels = map.get("num-other-sentinels").and_then(|s| s.parse().ok()).unwrap_or(0);
            let down_after_ms = map.get("down-after-milliseconds").and_then(|d| d.parse().ok()).unwrap_or(30000);
            let failover_timeout_ms = map.get("failover-timeout").and_then(|f| f.parse().ok()).unwrap_or(180000);

            masters.push(SentinelMasterInfo {
                name,
                ip,
                port,
                status,
                flags,
                quorum,
                num_slaves,
                num_other_sentinels,
                down_after_ms,
                failover_timeout_ms,
                slaves: Vec::new(),
                sentinels: Vec::new(),
            });
        }
        masters
    }

    pub fn parse_sentinel_slaves_entries(entries: &[std::collections::HashMap<String, String>]) -> Vec<SentinelSlaveInfo> {
        let mut slaves = Vec::new();
        for map in entries {
            let ip = map.get("ip").cloned().unwrap_or_else(|| "127.0.0.1".to_string());
            let port = map.get("port").and_then(|p| p.parse().ok()).unwrap_or(6379);
            let flags = map.get("flags").cloned().unwrap_or_else(|| "slave".to_string());
            let link_status = map.get("master-link-status").cloned().unwrap_or_else(|| "ok".to_string());
            let repl_offset = map.get("slave-repl-offset").and_then(|o| o.parse().ok()).unwrap_or(0);
            let lag_sec = map.get("master-link-down-time").and_then(|l| l.parse::<u64>().ok()).map(|ms| ms / 1000).unwrap_or(0);

            slaves.push(SentinelSlaveInfo {
                ip,
                port,
                flags,
                link_status,
                repl_offset,
                lag_sec,
            });
        }
        slaves
    }

    pub fn parse_sentinel_peers_entries(entries: &[std::collections::HashMap<String, String>]) -> Vec<SentinelPeerInfo> {
        let mut peers = Vec::new();
        for map in entries {
            let id = map.get("name").or_else(|| map.get("runid")).cloned().unwrap_or_else(|| "sentinel-peer".to_string());
            let ip = map.get("ip").cloned().unwrap_or_else(|| "127.0.0.1".to_string());
            let port = map.get("port").and_then(|p| p.parse().ok()).unwrap_or(26379);
            let flags = map.get("flags").cloned().unwrap_or_else(|| "sentinel".to_string());
            let is_healthy = !flags.contains("sdown") && !flags.contains("disconnected");
            let last_ok_ping_ms = map.get("last-ok-ping-reply").and_then(|p| p.parse().ok()).unwrap_or(0);

            peers.push(SentinelPeerInfo {
                id,
                ip,
                port,
                flags,
                is_healthy,
                last_ok_ping_ms,
            });
        }
        peers
    }
}
