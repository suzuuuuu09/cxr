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
) -> Result<(), String> {
    let command = replace_variables(hook, variable_map);
    let status = Command::new("/bin/sh")
        .arg("-c")
        .arg(&command)
        .current_dir(target_path)
        .status()
        .map_err(|e| e.to_string())?;

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
