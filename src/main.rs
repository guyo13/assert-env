use std::collections::HashMap;
use std::env;
use std::fs;
use std::process::{Command, exit};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

#[derive(Debug, Clone, Copy, PartialEq)]
enum VarType {
    Str,
    Int,
    Float,
    Any,
}

impl VarType {
    fn from_str(s: &str) -> Option<Self> {
        match s.trim() {
            "str" => Some(VarType::Str),
            "int" => Some(VarType::Int),
            "float" => Some(VarType::Float),
            "any" => Some(VarType::Any),
            _ => None,
        }
    }

    fn validate(&self, value: &str) -> bool {
        match self {
            VarType::Str => !value.is_empty(),
            VarType::Int => value.parse::<i64>().is_ok(),
            VarType::Float => value.parse::<f64>().is_ok(),
            VarType::Any => true,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            VarType::Str => "str",
            VarType::Int => "int",
            VarType::Float => "float",
            VarType::Any => "any",
        }
    }
}

struct Config {
    required: HashMap<String, VarType>,
    optional: HashMap<String, VarType>,
}

fn parse_config(content: &str) -> Result<Config, String> {
    let mut required = HashMap::new();
    let mut optional = HashMap::new();
    let mut current_section = "";

    for (i, line) in content.lines().enumerate() {
        let mut line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Strip inline comments
        if let Some(comment_pos) = line.find('#') {
            line = line[..comment_pos].trim();
        }

        if line.is_empty() {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            current_section = &line[1..line.len() - 1];
            continue;
        }

        if let Some(pos) = line.find('=') {
            let key = line[..pos].trim().to_string();
            let val_str = line[pos + 1..].trim();
            let var_type = VarType::from_str(val_str).ok_or_else(|| {
                format!(
                    "Line {}: Invalid type '{}' for key '{}'",
                    i + 1,
                    val_str,
                    key
                )
            })?;

            match current_section {
                "required" => {
                    required.insert(key, var_type);
                }
                "optional" => {
                    optional.insert(key, var_type);
                }
                _ => {
                    return Err(format!(
                        "Line {}: Assignment outside of [required] or [optional] section",
                        i + 1
                    ));
                }
            }
        } else {
            return Err(format!("Line {}: Invalid line format", i + 1));
        }
    }

    Ok(Config { required, optional })
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        println!("assert-env - Simple runtime assertions for environment variables\n");
        println!("Usage:");
        println!("  assert-env [-f <path/to/toml>] <command>\n");
        println!("Options:");
        println!("  -f, --file <path>  Path to AssertEnv.toml (default: AssertEnv.toml)");
        println!("  -h, --help         Show this help message\n");
        println!("Example:");
        println!("  assert-env \"node index.js\"");
        exit(0);
    }

    if args.len() < 2 {
        eprintln!("Error: No command provided. Use -h for help.");
        exit(1);
    }

    let mut toml_path = "AssertEnv.toml".to_string();
    let mut cmd_start_idx = 1;

    if args[1] == "-f" || args[1] == "--file" {
        if args.len() < 4 {
            eprintln!("Error: Missing path after {} flag", args[1]);
            exit(1);
        }
        toml_path = args[2].clone();
        cmd_start_idx = 3;
    }

    if cmd_start_idx >= args.len() {
        eprintln!("Error: No command provided");
        exit(1);
    }

    let content = match fs::read_to_string(&toml_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: Could not read config file '{}': {}", toml_path, e);
            exit(1);
        }
    };

    let config = match parse_config(&content) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: Parsing config failed: {}", e);
            exit(1);
        }
    };

    let mut errors = Vec::new();

    for (key, var_type) in &config.required {
        match env::var(key) {
            Ok(val) => {
                if val.is_empty() {
                    errors.push(format!("Required variable '{}' is empty", key));
                } else if !var_type.validate(&val) {
                    errors.push(format!(
                        "Required variable '{}' has invalid value '{}' (expected {})",
                        key,
                        val,
                        var_type.as_str()
                    ));
                }
            }
            Err(_) => {
                errors.push(format!("Required variable '{}' is missing", key));
            }
        }
    }

    for (key, var_type) in &config.optional {
        if let Ok(val) = env::var(key) {
            if !var_type.validate(&val) {
                errors.push(format!(
                    "Optional variable '{}' has invalid value '{}' (expected {})",
                    key,
                    val,
                    var_type.as_str()
                ));
            }
        }
    }

    if !errors.is_empty() {
        for err in errors {
            eprintln!("Assertion Error: {}", err);
        }
        exit(1);
    }

    // Execute the command
    let (cmd_bin, cmd_args) = if args.len() - cmd_start_idx == 1 {
        let parts: Vec<String> = args[cmd_start_idx]
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        if parts.is_empty() {
            eprintln!("Error: Empty command provided");
            exit(1);
        }
        let bin = parts[0].clone();
        let args = parts[1..].to_vec();
        (bin, args)
    } else {
        let bin = args[cmd_start_idx].clone();
        let cmd_args = args[cmd_start_idx + 1..].to_vec();
        (bin, cmd_args)
    };

    let mut cmd = Command::new(cmd_bin);
    cmd.args(cmd_args);

    #[cfg(unix)]
    {
        let err = cmd.exec();
        eprintln!("Error: Failed to execute command: {}", err);
        exit(1);
    }

    #[cfg(not(unix))]
    {
        let status = match cmd.status() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error: Failed to execute command: {}", e);
                exit(1);
            }
        };
        exit(status.code().unwrap_or(0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let content = "
[required]
KEY1=str  # some comment
KEY2 = int

[optional]
  KEY3=float
KEY4=any
";
        let config = parse_config(content).unwrap();
        assert_eq!(config.required.get("KEY1"), Some(&VarType::Str));
        assert_eq!(config.required.get("KEY2"), Some(&VarType::Int));
        assert_eq!(config.optional.get("KEY3"), Some(&VarType::Float));
        assert_eq!(config.optional.get("KEY4"), Some(&VarType::Any));
    }

    #[test]
    fn test_validate() {
        assert!(VarType::Str.validate("hello"));
        assert!(!VarType::Str.validate(""));
        assert!(VarType::Int.validate("123"));
        assert!(!VarType::Int.validate("abc"));
        assert!(VarType::Float.validate("1.23"));
        assert!(!VarType::Float.validate("abc"));
        assert!(VarType::Any.validate(""));
        assert!(VarType::Any.validate("anything"));
    }
}
