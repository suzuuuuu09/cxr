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

pub fn collect_variables(
    variables: Vec<Variable>,
    variable_map: &mut HashMap<String, String>,
) -> bool {
    for var in variables {
        let var_name = var.name();

        if variable_map.contains_key(&var_name) {
            continue;
        }

        let prompt_msg = format!("Enter value for {}:", var_name);
        let mut text_prompt = Text::new(&prompt_msg);

        let default_val = var.default_value();
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
                        "{} {}: {}",
                        "Error:".red().bold(),
                        format!("Variable '{}' cannot be empty", var_name),
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
                    "{} {}: {}",
                    "Error:".red().bold(),
                    format!("Failed to get input for variable '{}'", var_name),
                    e.to_string()
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
