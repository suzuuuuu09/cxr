use std::collections::HashMap;
use std::fs;

pub fn load_vars_from_file(path: &str) -> Result<HashMap<String, String>, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut errors = Vec::new();

    // Try YAML first, then JSON
    if let Ok(yaml_val) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
        if let Some(map) = yaml_val.as_mapping() {
            let mut result = HashMap::new();
            for (k, v) in map {
                let Some(key) = k.as_str() else {
                    errors.push("vars-file keys must be strings".to_string());
                    continue;
                };

                let Some(val) = v.as_str() else {
                    errors.push(format!("vars-file value for '{}' must be a string", key));
                    continue;
                };

                result.insert(key.to_string(), val.to_string());
            }

            if errors.is_empty() {
                return Ok(result);
            } else {
                return Err(errors.join("; "));
            }
        } else {
            errors.push("vars-file must be a mapping".to_string());
        }
    }

    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&content) {
        if let Some(map) = json_val.as_object() {
            let mut result = HashMap::new();
            for (k, v) in map {
                let Some(val_str) = v.as_str() else {
                    errors.push(format!("vars-file value for '{}' must be a string", k));
                    continue;
                };

                result.insert(k.clone(), val_str.to_string());
            }

            if errors.is_empty() {
                return Ok(result);
            } else {
                return Err(errors.join("; "));
            }
        } else {
            errors.push("vars-file must be a mapping".to_string());
        }
    }

    if errors.is_empty() {
        Err("Failed to parse vars-file as mapping of strings".to_string())
    } else {
        Err(errors.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use super::load_vars_from_file;
    use std::env::temp_dir;
    use std::fs;

    #[test]
    fn load_yaml_file() {
        let mut path = temp_dir();
        path.push("cx_test_vars.yaml");
        let content = "project_name: testproj\nauthor: \"Jane\"";
        fs::write(&path, content).unwrap();
        let map = load_vars_from_file(path.to_str().unwrap()).unwrap();
        assert_eq!(
            map.get("project_name").map(|s| s.as_str()),
            Some("testproj")
        );
        assert_eq!(map.get("author").map(|s| s.as_str()), Some("Jane"));
        fs::remove_file(&path).ok();
    }

    #[test]
    fn load_json_file() {
        let mut path = temp_dir();
        path.push("cx_test_vars.json");
        let content = r#"{"project_name": "jsonproj", "author":"Bob"}"#;
        fs::write(&path, content).unwrap();
        let map = load_vars_from_file(path.to_str().unwrap()).unwrap();
        assert_eq!(
            map.get("project_name").map(|s| s.as_str()),
            Some("jsonproj")
        );
        assert_eq!(map.get("author").map(|s| s.as_str()), Some("Bob"));
        fs::remove_file(&path).ok();
    }

    #[test]
    fn fail_on_non_mapping() {
        let mut path = temp_dir();
        path.push("cx_test_vars_nok.yaml");
        let content = "- item1\n- item2";
        fs::write(&path, content).unwrap();
        assert!(load_vars_from_file(path.to_str().unwrap()).is_err());
        fs::remove_file(&path).ok();
    }

    #[test]
    fn fail_on_non_string_values() {
        let mut path = temp_dir();
        path.push("cx_test_vars_bad.yaml");
        let content = "project_name: 123\nflag: true";
        fs::write(&path, content).unwrap();
        let err = load_vars_from_file(path.to_str().unwrap()).unwrap_err();
        assert!(err.contains("project_name"));
        assert!(err.contains("flag"));
        fs::remove_file(&path).ok();
    }
}
