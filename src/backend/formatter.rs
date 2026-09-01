use redis::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormattedValue {
    Status(String),
    Integer(i64),
    String(String),
    Json(String),
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    List(Vec<String>),
    #[allow(dead_code)]
    Tree {
        root: String,
        items: Vec<(String, String)>,
    },
    Nil,
    Error(String),
}

impl FormattedValue {
    pub fn from_redis_value(val: Value) -> Self {
        match val {
            Value::Nil => FormattedValue::Nil,
            Value::Int(i) => FormattedValue::Integer(i),
            Value::BulkString(bytes) => {
                if let Ok(s) = String::from_utf8(bytes.clone()) {
                    let clean = s.replace("\r\n", "\n").replace('\r', "");
                    let trimmed = clean.trim();
                    if (trimmed.starts_with('{') && trimmed.ends_with('}'))
                        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
                    {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
                            if let Ok(pretty) = serde_json::to_string_pretty(&parsed) {
                                return FormattedValue::Json(pretty);
                            }
                        }
                    }
                    FormattedValue::String(clean)
                } else {
                    FormattedValue::String(format!("<Binary data: {} bytes>", bytes.len()))
                }
            }
            Value::SimpleString(s) => {
                let clean = s.replace("\r\n", "\n").replace('\r', "");
                FormattedValue::Status(clean)
            }
            Value::Array(items) => {
                // If it's an even-length list and looks like field-value pairs (e.g. HGETALL)
                if !items.is_empty() && items.len() % 2 == 0 {
                    let mut rows = Vec::new();
                    for chunk in items.chunks(2) {
                        let k = value_to_string(&chunk[0]).replace(['\r', '\n'], " ");
                        let v = value_to_string(&chunk[1]).replace(['\r', '\n'], " ");
                        rows.push(vec![k, v]);
                    }
                    if !rows.is_empty() {
                        return FormattedValue::Table {
                            headers: vec!["Field".to_string(), "Value".to_string()],
                            rows,
                        };
                    }
                }

                // Default to list
                let list: Vec<String> = items.iter().map(|item| value_to_string(item).replace(['\r', '\n'], " ")).collect();
                FormattedValue::List(list)
            }
            Value::Map(map) => {
                // Check if the map is a cluster broadcast response (e.g. node_address -> node_output)
                let is_cluster_node_response = map.iter().any(|(k, v)| {
                    let k_str = value_to_string(k);
                    let v_str = value_to_string(v);
                    (k_str.contains(':') || k_str.starts_with("node")) && (v_str.contains('\n') || v_str.contains("\r") || v_str.starts_with('#'))
                });

                if is_cluster_node_response {
                    let mut sections = Vec::new();
                    for (k, v) in map {
                        let node_addr = value_to_string(&k).replace(['\r', '\n'], "");
                        let v_str = value_to_string(&v).replace("\r\n", "\n").replace('\r', "");
                        sections.push(format!("--- Node: @{} ---\n{}", node_addr, v_str.trim()));
                    }
                    FormattedValue::String(sections.join("\n\n"))
                } else {
                    let rows: Vec<Vec<String>> = map
                        .into_iter()
                        .map(|(k, v)| {
                            let k_clean = value_to_string(&k).replace(['\r', '\n'], " ");
                            let v_clean = value_to_string(&v).replace(['\r', '\n'], " ");
                            vec![k_clean, v_clean]
                        })
                        .collect();
                    FormattedValue::Table {
                        headers: vec!["Key / Field".to_string(), "Value".to_string()],
                        rows,
                    }
                }
            }
            Value::Set(set) => {
                let list: Vec<String> = set.iter().map(|item| value_to_string(item).replace(['\r', '\n'], " ")).collect();
                FormattedValue::List(list)
            }
            Value::Okay => FormattedValue::Status("OK".to_string()),
            Value::ServerError(err) => FormattedValue::Error(format!("{:?}", err).replace(['\r', '\n'], " ")),
            _ => FormattedValue::String(format!("{:?}", val).replace(['\r', '\n'], " ")),
        }
    }
}

pub fn value_to_string(val: &Value) -> String {
    match val {
        Value::Nil => "(nil)".to_string(),
        Value::Int(i) => i.to_string(),
        Value::BulkString(bytes) => {
            let s = String::from_utf8_lossy(bytes).to_string();
            s.replace("\r\n", "\n").replace('\r', "")
        }
        Value::SimpleString(s) => s.replace("\r\n", "\n").replace('\r', ""),
        Value::Array(items) | Value::Set(items) => {
            let str_items: Vec<String> = items.iter().map(value_to_string).collect();
            format!("[{}]", str_items.join(", "))
        }
        Value::Okay => "OK".to_string(),
        Value::ServerError(err) => format!("(error) {:?}", err),
        _ => format!("{:?}", val),
    }
}
