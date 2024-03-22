use std::str::FromStr;

pub enum RedisCommand {
    Ping,
    Echo(String),
}

impl FromStr for RedisCommand {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.trim().split_whitespace();
        let command = parts.next().ok_or_else(|| "Empty Command".to_string())?;

        let command = match command.to_lowercase().as_str() {
            "ping" => RedisCommand::Ping,
            "echo" => {
                let echoed_string = parts.collect::<String>();
                if echoed_string.is_empty() {
                    return Err("Echo requires a body afterwards".to_string());
                };
                RedisCommand::Echo(echoed_string)
            }
            other => return Err(format!("Command {other} not supported")),
        };

        Ok(command)
    }
}
