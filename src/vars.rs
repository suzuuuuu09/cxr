use colored::*;
use inquire::Text;
use std::collections::HashMap;

use crate::template::Variable;

pub fn resolve_variable_input(
    input: &str,
    default_value: Option<String>,
) -> Result<String, &'static str> {
    let input = input.trim();

    if input.is_empty() {
        default_value.ok_or("a value is required")
    } else {
        Ok(input.to_string())
    }
}

pub fn parse_cli_vars(vars: &[String]) -> Result<HashMap<String, String>, String> {
    let mut variable_map = HashMap::new();

    for raw_var in vars {
        let Some((key, value)) = raw_var.split_once('=') else {
            return Err(format!(
                "invalid -v value '{}': expected VAR=VALUE",
                raw_var
            ));
        };

        let key = key.trim();
        if key.is_empty() {
            return Err(format!(
                "invalid -v value '{}': variable name must not be empty",
                raw_var
            ));
        }

        variable_map.insert(key.to_string(), value.trim().to_string());
    }

    Ok(variable_map)
}

pub fn collect_variables(
    variables: Vec<Variable>,
    variable_map: &mut HashMap<String, String>,
) -> bool {
    for var in variables {
        let (var_name, default_val) = match var.resolve() {
            Ok(parts) => parts,
            Err(err) => {
                eprintln!(
                    "{} invalid variable definition: {}",
                    "Error:".red().bold(),
                    err
                );
                return false;
            }
        };

        if variable_map.contains_key(&var_name) {
            continue;
        }

        let prompt_msg = format!("Enter value for {}:", var_name);
        let mut text_prompt = Text::new(&prompt_msg);

        if let Some(ref val) = default_val {
            text_prompt = text_prompt.with_default(val);
        }

        match text_prompt.prompt() {
            Ok(input) => match resolve_variable_input(&input, default_val) {
                Ok(value) => {
                    variable_map.insert(var_name, value);
                }
                Err(err) => {
                    eprintln!(
                        "{} Variable '{}' cannot be empty: {}",
                        "Error:".red().bold(),
                        var_name,
                        err
                    );
                    return false;
                }
            },
            Err(inquire::InquireError::OperationInterrupted)
            | Err(inquire::InquireError::OperationCanceled) => {
                println!("\n{}", "Operation cancelled.".red());
                return false;
            }
            Err(e) => {
                eprintln!(
                    "{} Failed to get input for variable '{}': {}",
                    "Error:".red().bold(),
                    var_name,
                    e
                );
                return false;
            }
        }
    }

    println!("\n{}", "Variables:".bold().cyan());
    for (key, val) in variable_map.iter() {
        println!("  {} {}: {}", "->".cyan(), key, val.green());
    }
    true
}

#[cfg(test)]
mod tests {
    use super::parse_cli_vars;

    #[test]
    fn parse_cli_vars_accepts_valid_entries() {
        let vars = vec!["project_name=demo".to_string(), "author=Jane".to_string()];
        let map = parse_cli_vars(&vars).unwrap();
        assert_eq!(map.get("project_name").map(|s| s.as_str()), Some("demo"));
        assert_eq!(map.get("author").map(|s| s.as_str()), Some("Jane"));
    }

    #[test]
    fn parse_cli_vars_rejects_missing_equals() {
        let vars = vec!["project_name".to_string()];
        assert!(parse_cli_vars(&vars).is_err());
    }

    #[test]
    fn parse_cli_vars_rejects_empty_key() {
        let vars = vec![" =value".to_string()];
        assert!(parse_cli_vars(&vars).is_err());
    }
}
