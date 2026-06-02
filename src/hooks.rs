use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Run a shell hook (single command string) in `target_path` with variables substituted.
/// Returns Err(String) on non-zero exit or execution error.
pub fn run_hook(
    name: &str,
    hook: &str,
    target_path: &Path,
    variable_map: &HashMap<String, String>,
    template_name: &str,
    template_description: &str,
) -> Result<(), String> {
    let rendered_hook = replace_variables(hook, variable_map);
    let mut command = Command::new("/bin/sh");
    command
        .arg("-c")
        .arg(&rendered_hook)
        .current_dir(target_path)
        .env("CX_OUTPUT_DIR", target_path)
        .env("CX_TEMPLATE_NAME", template_name)
        .env("CX_TEMPLATE_DESCRIPTION", template_description);

    for (key, value) in variable_map {
        if is_valid_env_key(key) {
            command.env(key, value);
        }
    }

    let status = command.status().map_err(|e| e.to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "{} exited with status {}",
            name,
            status.code().unwrap_or(-1)
        ))
    }
}

/// Replace occurrences of `{{ key }}` in the input string with values from `variable_map`.
pub fn replace_variables(input: &str, variable_map: &HashMap<String, String>) -> String {
    let mut resolved = input.to_string();
    for (key, val) in variable_map {
        let target = format!("{{{{ {} }}}}", key);
        resolved = resolved.replace(&target, val);
    }
    resolved
}

fn is_valid_env_key(key: &str) -> bool {
    let mut chars = key.chars();
    match chars.next() {
        Some(first) if first == '_' || first.is_ascii_alphabetic() => {}
        _ => return false,
    }

    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::replace_variables;
    use std::collections::HashMap;

    #[test]
    fn replace_variables_substitutes() {
        let mut m = HashMap::new();
        m.insert("name".to_string(), "Alice".to_string());
        assert_eq!(replace_variables("Hello {{ name }}!", &m), "Hello Alice!");
    }

    #[test]
    fn replace_variables_no_match() {
        let m: HashMap<String, String> = HashMap::new();
        assert_eq!(replace_variables("No vars", &m), "No vars");
    }
}
