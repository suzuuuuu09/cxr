use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
// テンプレート全体を表す構造体
pub struct Template {
    pub name: String,
    pub description: String,
    pub variables: Option<Vec<Variable>>,
    pub pre_hook: Option<String>,
    pub post_hook: Option<String>,
    pub tags: Option<Vec<String>>,
    pub extends: Option<String>,
    pub items: Vec<TemplateItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Variable {
    Simple(String),
    WithDefault(std::collections::HashMap<String, String>),
}

impl Variable {
    pub fn resolve(&self) -> Result<(String, Option<String>), String> {
        match self {
            Variable::Simple(name) => {
                let name = name.trim();
                if name.is_empty() {
                    Err("variable name must not be empty".to_string())
                } else {
                    Ok((name.to_string(), None))
                }
            }
            Variable::WithDefault(map) => {
                if map.len() != 1 {
                    return Err("variable with default must contain exactly one key".to_string());
                }

                let Some((name, value)) = map.iter().next() else {
                    return Err("variable with default must contain exactly one key".to_string());
                };

                let name = name.trim();
                if name.is_empty() {
                    return Err("variable name must not be empty".to_string());
                }

                Ok((name.to_string(), Some(value.clone())))
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
// テンプレートのアイテムを表す列挙型
pub enum TemplateItem {
    Directory {
        name: String,
        items: Option<Vec<TemplateItem>>,
        when: Option<String>,
    },
    File {
        name: String,
        content: Option<String>,
        when: Option<String>,
    },
}

impl Template {
    pub fn merge_with_parent(mut self, parent: Template) -> Template {
        if self.name.trim().is_empty() {
            self.name = parent.name;
        }

        if self.description.trim().is_empty() {
            self.description = parent.description;
        }

        self.variables = merge_variables(parent.variables, self.variables);
        self.pre_hook = self.pre_hook.or(parent.pre_hook);
        self.post_hook = self.post_hook.or(parent.post_hook);
        self.tags = merge_tags(parent.tags, self.tags);

        let mut items = parent.items;
        items.extend(self.items);
        self.items = items;
        self.extends = None;
        self
    }
}

pub fn load_template(config_dir: &Path, template_name: &str) -> Result<Template, String> {
    let mut stack = Vec::new();
    load_template_recursive(config_dir, template_name, &mut stack)
}

fn load_template_recursive(
    config_dir: &Path,
    template_name: &str,
    stack: &mut Vec<String>,
) -> Result<Template, String> {
    if stack.iter().any(|name| name == template_name) {
        stack.push(template_name.to_string());
        return Err(format!(
            "cyclic template inheritance detected: {}",
            stack.join(" -> ")
        ));
    }

    stack.push(template_name.to_string());

    let path = config_dir.join(format!("{}.yaml", template_name));
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("failed to read template '{}': {}", path.display(), e))?;
    let template = serde_yaml::from_str::<Template>(&content)
        .map_err(|e| format!("failed to parse template '{}': {}", path.display(), e))?;

    let resolved = if let Some(parent_name) = template.extends.clone() {
        let parent = load_template_recursive(config_dir, &parent_name, stack)?;
        template.merge_with_parent(parent)
    } else {
        template
    };

    stack.pop();
    Ok(resolved)
}

pub fn template_name_matches(template: &Template, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return true;
    }

    let mut haystack = vec![
        template.name.to_lowercase(),
        template.description.to_lowercase(),
    ];

    if let Some(tags) = &template.tags {
        haystack.extend(tags.iter().map(|tag| tag.to_lowercase()));
    }

    haystack.iter().any(|value| value.contains(&query))
}

pub fn merge_variables(
    parent: Option<Vec<Variable>>,
    child: Option<Vec<Variable>>,
) -> Option<Vec<Variable>> {
    let mut merged = parent.unwrap_or_default();
    if let Some(child_vars) = child {
        for var in child_vars {
            let Ok((child_name, _)) = var.resolve() else {
                merged.push(var);
                continue;
            };
            let exists = merged.iter().any(|existing| {
                existing
                    .resolve()
                    .map(|(name, _)| name == child_name)
                    .unwrap_or(false)
            });
            if !exists {
                merged.push(var);
            }
        }
    }

    if merged.is_empty() {
        None
    } else {
        Some(merged)
    }
}

fn merge_tags(parent: Option<Vec<String>>, child: Option<Vec<String>>) -> Option<Vec<String>> {
    let mut merged = Vec::new();
    let mut seen = HashSet::<String>::new();

    for tag in parent
        .into_iter()
        .flatten()
        .chain(child.into_iter().flatten())
    {
        let normalized = tag.trim();
        if normalized.is_empty() {
            continue;
        }
        if seen.insert(normalized.to_lowercase()) {
            merged.push(normalized.to_string());
        }
    }

    if merged.is_empty() {
        None
    } else {
        Some(merged)
    }
}

pub fn should_render_item(
    condition: &Option<String>,
    variable_map: &std::collections::HashMap<String, String>,
) -> Result<bool, String> {
    let Some(condition) = condition.as_ref() else {
        return Ok(true);
    };

    let condition = condition.trim();
    if condition.is_empty() {
        return Err("condition must not be empty".to_string());
    }

    if let Some(rest) = condition.strip_prefix('!') {
        let key = rest.trim();
        if key.is_empty() {
            return Err("condition must reference a variable".to_string());
        }
        return Ok(!is_truthy(variable_map.get(key)));
    }

    if let Some((left, right)) = condition.split_once("==") {
        let key = left.trim();
        let expected = strip_quotes(right.trim());
        if key.is_empty() {
            return Err("condition must reference a variable".to_string());
        }
        return Ok(variable_map
            .get(key)
            .map(|value| value == expected)
            .unwrap_or(false));
    }

    if let Some((left, right)) = condition.split_once("!=") {
        let key = left.trim();
        let expected = strip_quotes(right.trim());
        if key.is_empty() {
            return Err("condition must reference a variable".to_string());
        }
        return Ok(variable_map
            .get(key)
            .map(|value| value != expected)
            .unwrap_or(true));
    }

    Ok(is_truthy(variable_map.get(condition)))
}

fn is_truthy(value: Option<&String>) -> bool {
    match value.map(|value| value.trim().to_lowercase()) {
        Some(ref value) if value.is_empty() => false,
        Some(value) if matches!(value.as_str(), "false" | "0" | "no" | "off") => false,
        Some(_) => true,
        None => false,
    }
}

fn strip_quotes(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap_or(value)
}

#[cfg(test)]
mod tests {
    use super::{Variable, load_template, should_render_item};
    use std::collections::HashMap;
    use std::env::temp_dir;
    use std::fs;

    #[test]
    fn resolve_rejects_empty_default_map() {
        let variable = Variable::WithDefault(HashMap::new());
        assert!(variable.resolve().is_err());
    }

    #[test]
    fn resolve_returns_name_and_default() {
        let mut map = HashMap::new();
        map.insert("project_name".to_string(), "demo".to_string());
        let variable = Variable::WithDefault(map);
        let resolved = variable.resolve().unwrap();
        assert_eq!(resolved.0, "project_name");
        assert_eq!(resolved.1.as_deref(), Some("demo"));
    }

    #[test]
    fn should_render_item_supports_basic_conditions() {
        let mut vars = HashMap::new();
        vars.insert("enabled".to_string(), "true".to_string());
        vars.insert("mode".to_string(), "web".to_string());

        assert_eq!(
            should_render_item(&Some("enabled".to_string()), &vars),
            Ok(true)
        );
        assert_eq!(
            should_render_item(&Some("!enabled".to_string()), &vars),
            Ok(false)
        );
        assert_eq!(
            should_render_item(&Some("mode == web".to_string()), &vars),
            Ok(true)
        );
        assert_eq!(
            should_render_item(&Some("mode != api".to_string()), &vars),
            Ok(true)
        );
    }

    #[test]
    fn load_template_resolves_extends() {
        let mut dir = temp_dir();
        dir.push("cx_test_template_extends");
        fs::remove_dir_all(&dir).ok();
        fs::create_dir_all(&dir).unwrap();

        fs::write(
            dir.join("base.yaml"),
            r#"
name: base
description: base desc
variables:
  - base_var
tags:
  - shared
items:
  - type: file
    name: base.txt
    content: base
"#,
        )
        .unwrap();

        fs::write(
            dir.join("child.yaml"),
            r#"
name: child
description: child desc
extends: base
variables:
  - child_var
tags:
  - child
items:
  - type: file
    name: child.txt
    content: child
"#,
        )
        .unwrap();

        let template = load_template(&dir, "child").unwrap();
        assert_eq!(template.name, "child");
        assert_eq!(template.description, "child desc");
        assert_eq!(template.items.len(), 2);
        assert!(template.tags.unwrap().contains(&"shared".to_string()));

        fs::remove_dir_all(&dir).ok();
    }
}
