use crate::template::{Template, TemplateItem, Variable};
use serde_yaml::{Mapping, Value};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub struct LintReport {
    pub errors: Vec<String>,
}

pub fn lint_template_file(path: &Path) -> LintReport {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            return LintReport {
                errors: vec![format!("failed to read template: {}", e)],
            };
        }
    };

    let value = match serde_yaml::from_str::<Value>(&content) {
        Ok(value) => value,
        Err(e) => {
            return LintReport {
                errors: vec![format!("failed to parse template: {}", e)],
            };
        }
    };

    let mut errors = Vec::new();
    let variables = lint_yaml_template(&value, &mut errors);
    lint_placeholders(&value, &variables, &mut errors);

    if let Ok(template) = serde_yaml::from_value::<Template>(value) {
        errors.extend(lint_template(&template));
    }

    LintReport { errors }
}

fn lint_template(template: &Template) -> Vec<String> {
    let mut errors = Vec::new();

    check_non_empty("name", &template.name, &mut errors);
    check_non_empty("description", &template.description, &mut errors);

    if let Some(vars) = template.variables.as_ref() {
        for (idx, var) in vars.iter().enumerate() {
            match var {
                Variable::Simple(name) => {
                    check_non_empty(&format!("variables[{}]", idx), name, &mut errors);
                }
                Variable::WithDefault(map) => {
                    if map.len() != 1 {
                        errors.push(format!(
                            "variables[{}] must contain exactly one key",
                            idx
                        ));
                    }
                    for name in map.keys() {
                        check_non_empty(&format!("variables[{}]", idx), name, &mut errors);
                    }
                }
            }
        }
    }

    lint_items(&template.items, "items", &mut errors);
    errors
}

fn lint_items(items: &[TemplateItem], path: &str, errors: &mut Vec<String>) {
    for (idx, item) in items.iter().enumerate() {
        let item_path = format!("{}[{}]", path, idx);
        match item {
            TemplateItem::Directory { name, items } => {
                check_non_empty(&format!("{}.name", item_path), name, errors);
                if let Some(inner) = items.as_ref() {
                    lint_items(inner, &format!("{}.items", item_path), errors);
                }
            }
            TemplateItem::File { name, .. } => {
                check_non_empty(&format!("{}.name", item_path), name, errors);
            }
        }
    }
}

fn check_non_empty(field: &str, value: &str, errors: &mut Vec<String>) {
    if value.trim().is_empty() {
        errors.push(format!("{} must not be empty", field));
    }
}

fn lint_yaml_template(value: &Value, errors: &mut Vec<String>) -> HashSet<String> {
    let Some(map) = value.as_mapping() else {
        errors.push("template must be a mapping".to_string());
        return HashSet::new();
    };

    let mut variables = HashSet::new();
    let allowed: HashSet<&str> = [
        "name",
        "description",
        "variables",
        "pre_hook",
        "post_hook",
        "items",
    ]
    .into_iter()
    .collect();

    for key in map.keys() {
        let Some(key_str) = key.as_str() else {
            errors.push("top-level keys must be strings".to_string());
            continue;
        };
        if !allowed.contains(key_str) {
            errors.push(format!("unknown field '{}' at top level", key_str));
        }
    }

    match map.get(&Value::from("name")) {
        Some(Value::String(name)) => check_non_empty("name", name, errors),
        Some(_) => errors.push("name must be a string".to_string()),
        None => errors.push("name is required".to_string()),
    }

    match map.get(&Value::from("description")) {
        Some(Value::String(desc)) => check_non_empty("description", desc, errors),
        Some(_) => errors.push("description must be a string".to_string()),
        None => errors.push("description is required".to_string()),
    }

    if let Some(Value::String(hook)) = map.get(&Value::from("pre_hook")) {
        check_non_empty("pre_hook", hook, errors);
    } else if let Some(value) = map.get(&Value::from("pre_hook")) {
        if !value.is_null() {
            errors.push("pre_hook must be a string".to_string());
        }
    }

    if let Some(Value::String(hook)) = map.get(&Value::from("post_hook")) {
        check_non_empty("post_hook", hook, errors);
    } else if let Some(value) = map.get(&Value::from("post_hook")) {
        if !value.is_null() {
            errors.push("post_hook must be a string".to_string());
        }
    }

    if let Some(vars_val) = map.get(&Value::from("variables")) {
        match vars_val {
            Value::Sequence(seq) => lint_variables(seq, errors, &mut variables),
            _ => errors.push("variables must be a list".to_string()),
        }
    }

    match map.get(&Value::from("items")) {
        Some(Value::Sequence(items)) => lint_items_value(items, "items", errors),
        Some(_) => errors.push("items must be a list".to_string()),
        None => errors.push("items is required".to_string()),
    }

    variables
}

fn lint_variables(seq: &[Value], errors: &mut Vec<String>, variables: &mut HashSet<String>) {
    for (idx, item) in seq.iter().enumerate() {
        match item {
            Value::String(name) => {
                if name.trim().is_empty() {
                    errors.push(format!("variables[{}] must not be empty", idx));
                } else if !variables.insert(name.trim().to_string()) {
                    errors.push(format!("variables[{}] is duplicated", idx));
                }
            }
            Value::Mapping(map) => {
                if map.len() != 1 {
                    errors.push(format!(
                        "variables[{}] must contain exactly one key",
                        idx
                    ));
                }
                for (key, value) in map {
                    let Some(name) = key.as_str() else {
                        errors.push(format!("variables[{}] key must be a string", idx));
                        continue;
                    };
                    if name.trim().is_empty() {
                        errors.push(format!("variables[{}] must not be empty", idx));
                    } else if !variables.insert(name.trim().to_string()) {
                        errors.push(format!("variables[{}] is duplicated", idx));
                    }
                    if !matches!(value, Value::String(_)) {
                        errors.push(format!(
                            "variables[{}] default value must be a string",
                            idx
                        ));
                    }
                }
            }
            _ => errors.push(format!("variables[{}] must be a string or map", idx)),
        }
    }
}

fn lint_items_value(items: &[Value], path: &str, errors: &mut Vec<String>) {
    for (idx, item) in items.iter().enumerate() {
        let item_path = format!("{}[{}]", path, idx);
        let Some(map) = item.as_mapping() else {
            errors.push(format!("{} must be a mapping", item_path));
            continue;
        };

        let item_type = match map.get(&Value::from("type")) {
            Some(Value::String(kind)) => kind.as_str(),
            Some(_) => {
                errors.push(format!("{}.type must be a string", item_path));
                continue;
            }
            None => {
                errors.push(format!("{}.type is required", item_path));
                continue;
            }
        };

        match item_type {
            "directory" => lint_directory_item(map, &item_path, errors),
            "file" => lint_file_item(map, &item_path, errors),
            _ => errors.push(format!("{}.type must be 'directory' or 'file'", item_path)),
        }
    }
}

fn lint_directory_item(map: &Mapping, path: &str, errors: &mut Vec<String>) {
    for key in map.keys() {
        let Some(key_str) = key.as_str() else {
            errors.push(format!("{} has a non-string key", path));
            continue;
        };
        if !matches!(key_str, "type" | "name" | "items") {
            errors.push(format!("unknown field '{}' at {}", key_str, path));
        }
    }

    match map.get(&Value::from("name")) {
        Some(Value::String(name)) => check_non_empty(&format!("{}.name", path), name, errors),
        Some(_) => errors.push(format!("{}.name must be a string", path)),
        None => errors.push(format!("{}.name is required", path)),
    }

    if let Some(items_val) = map.get(&Value::from("items")) {
        match items_val {
            Value::Sequence(items) => lint_items_value(items, &format!("{}.items", path), errors),
            _ => errors.push(format!("{}.items must be a list", path)),
        }
    }
}

fn lint_file_item(map: &Mapping, path: &str, errors: &mut Vec<String>) {
    for key in map.keys() {
        let Some(key_str) = key.as_str() else {
            errors.push(format!("{} has a non-string key", path));
            continue;
        };
        if !matches!(key_str, "type" | "name" | "content") {
            errors.push(format!("unknown field '{}' at {}", key_str, path));
        }
    }

    match map.get(&Value::from("name")) {
        Some(Value::String(name)) => check_non_empty(&format!("{}.name", path), name, errors),
        Some(_) => errors.push(format!("{}.name must be a string", path)),
        None => errors.push(format!("{}.name is required", path)),
    }

    if let Some(content) = map.get(&Value::from("content")) {
        if !matches!(content, Value::String(_)) && !content.is_null() {
            errors.push(format!("{}.content must be a string", path));
        }
    }
}

fn lint_placeholders(value: &Value, variables: &HashSet<String>, errors: &mut Vec<String>) {
    let Some(map) = value.as_mapping() else {
        return;
    };

    if let Some(Value::String(hook)) = map.get(&Value::from("pre_hook")) {
        check_placeholders("pre_hook", hook, variables, errors);
    }

    if let Some(Value::String(hook)) = map.get(&Value::from("post_hook")) {
        check_placeholders("post_hook", hook, variables, errors);
    }

    if let Some(Value::Sequence(items)) = map.get(&Value::from("items")) {
        lint_item_placeholders(items, "items", variables, errors);
    }
}

fn lint_item_placeholders(
    items: &[Value],
    path: &str,
    variables: &HashSet<String>,
    errors: &mut Vec<String>,
) {
    for (idx, item) in items.iter().enumerate() {
        let item_path = format!("{}[{}]", path, idx);
        let Some(map) = item.as_mapping() else {
            continue;
        };

        if let Some(Value::String(name)) = map.get(&Value::from("name")) {
            check_placeholders(&format!("{}.name", item_path), name, variables, errors);
        }
        if let Some(Value::String(content)) = map.get(&Value::from("content")) {
            check_placeholders(&format!("{}.content", item_path), content, variables, errors);
        }
        if let Some(Value::Sequence(inner)) = map.get(&Value::from("items")) {
            lint_item_placeholders(inner, &format!("{}.items", item_path), variables, errors);
        }
    }
}

fn check_placeholders(
    path: &str,
    input: &str,
    variables: &HashSet<String>,
    errors: &mut Vec<String>,
) {
    for var in extract_placeholders(input) {
        if !variables.contains(&var) {
            errors.push(format!("undefined variable '{}' referenced in {}", var, path));
        }
    }
}

fn extract_placeholders(input: &str) -> Vec<String> {
    let mut vars = Vec::new();
    let mut rest = input;
    while let Some(start) = rest.find("{{") {
        let after_start = &rest[start + 2..];
        let Some(end) = after_start.find("}}") else {
            break;
        };
        let raw = after_start[..end].trim();
        if !raw.is_empty() {
            vars.push(raw.to_string());
        }
        rest = &after_start[end + 2..];
    }
    vars
}

#[cfg(test)]
mod tests {
    use super::{extract_placeholders, lint_template, lint_template_file};
    use crate::template::{Template, TemplateItem, Variable};
    use std::collections::HashMap;
    use std::env::temp_dir;
    use std::fs;

    #[test]
    fn lint_rejects_blank_name() {
        let template = Template {
            name: "   ".to_string(),
            description: "desc".to_string(),
            variables: None,
            pre_hook: None,
            post_hook: None,
            items: vec![],
        };
        let errors = lint_template(&template);
        assert!(errors.iter().any(|e| e.contains("name")));
    }

    #[test]
    fn lint_rejects_variable_with_multiple_keys() {
        let mut map = HashMap::new();
        map.insert("one".to_string(), "1".to_string());
        map.insert("two".to_string(), "2".to_string());
        let template = Template {
            name: "tmpl".to_string(),
            description: "desc".to_string(),
            variables: Some(vec![Variable::WithDefault(map)]),
            pre_hook: None,
            post_hook: None,
            items: vec![],
        };
        let errors = lint_template(&template);
        assert!(errors
            .iter()
            .any(|e| e.contains("must contain exactly one key")));
    }

    #[test]
    fn lint_rejects_blank_item_name() {
        let template = Template {
            name: "tmpl".to_string(),
            description: "desc".to_string(),
            variables: None,
            pre_hook: None,
            post_hook: None,
            items: vec![TemplateItem::File {
                name: "  ".to_string(),
                content: None,
            }],
        };
        let errors = lint_template(&template);
        assert!(errors.iter().any(|e| e.contains("items[0].name")));
    }

    #[test]
    fn lint_flags_unknown_top_level_key() {
        let yaml = r#"
name: tmpl
description: desc
varables:
  - project_name
items: []
"#;
        let mut path = temp_dir();
        path.push("cx_test_lint_unknown.yaml");
        fs::write(&path, yaml).unwrap();
        let report = lint_template_file(&path);
        fs::remove_file(&path).ok();
        assert!(report
            .errors
            .iter()
            .any(|e| e.contains("unknown field 'varables'")));
    }

    #[test]
    fn lint_flags_undefined_variable_in_hook() {
        let yaml = r#"
name: tmpl
description: desc
variables:
  - project_name
pre_hook: "echo {{ project_nmae }}"
items: []
"#;
        let mut path = temp_dir();
        path.push("cx_test_lint_hook.yaml");
        fs::write(&path, yaml).unwrap();
        let report = lint_template_file(&path);
        fs::remove_file(&path).ok();
        assert!(report
            .errors
            .iter()
            .any(|e| e.contains("project_nmae")));
    }

    #[test]
    fn extract_placeholders_handles_multiple() {
        let vars = extract_placeholders("{{ one }} and {{two}}");
        assert_eq!(vars, vec!["one".to_string(), "two".to_string()]);
    }
}
