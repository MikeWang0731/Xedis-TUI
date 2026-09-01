#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandType {
    Macro {
        name: String,
        args: Vec<String>,
    },
    Native {
        cmd: String,
        args: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCommand {
    pub target_node: Option<String>,
    pub command_type: CommandType,
    pub raw_input: String,
}

pub struct CommandRouter;

impl CommandRouter {
    pub fn parse(input: &str) -> Option<ParsedCommand> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }

        let tokens = Self::tokenize(trimmed);
        if tokens.is_empty() {
            return None;
        }

        let mut target_node = None;
        let mut start_idx = 0;

        // Check if first token is a node selector e.g. @node-1 or @127.0.0.1:6379
        if tokens[0].starts_with('@') && tokens[0].len() > 1 {
            target_node = Some(tokens[0][1..].to_string());
            start_idx = 1;
        }

        if start_idx >= tokens.len() {
            return None;
        }

        let first_cmd = &tokens[start_idx];
        let args = tokens[start_idx + 1..].to_vec();

        let command_type = if first_cmd.starts_with('/') {
            CommandType::Macro {
                name: first_cmd.to_lowercase(),
                args,
            }
        } else {
            CommandType::Native {
                cmd: first_cmd.to_uppercase(),
                args,
            }
        };

        Some(ParsedCommand {
            target_node,
            command_type,
            raw_input: trimmed.to_string(),
        })
    }

    /// Tokenize input string preserving quoted segments ('single' or "double")
    pub fn tokenize(input: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut escaped = false;

        for c in input.chars() {
            if escaped {
                current.push(c);
                escaped = false;
                continue;
            }

            if c == '\\' {
                escaped = true;
                continue;
            }

            if c == '\'' && !in_double_quote {
                in_single_quote = !in_single_quote;
                continue;
            }

            if c == '"' && !in_single_quote {
                in_double_quote = !in_double_quote;
                continue;
            }

            if c.is_whitespace() && !in_single_quote && !in_double_quote {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            } else {
                current.push(c);
            }
        }

        if !current.is_empty() {
            tokens.push(current);
        }

        tokens
    }
}
