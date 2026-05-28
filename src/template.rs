use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
// テンプレート全体を表す構造体
pub struct Template {
    pub name: String,
    pub description: String,
    pub variables: Option<Vec<Variable>>,
    pub pre_hook: Option<String>,
    pub post_hook: Option<String>,
    pub items: Vec<TemplateItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Variable {
    Simple(String),
    WithDefault(std::collections::HashMap<String, String>),
}

impl Variable {
    pub fn name(&self) -> String {
        match self {
            Variable::Simple(name) => name.clone(),
            Variable::WithDefault(map) => map.keys().next().unwrap().clone(),
        }
    }

    pub fn default_value(&self) -> Option<String> {
        match self {
            Variable::Simple(_) => None,
            Variable::WithDefault(map) => map.values().next().cloned(),
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
    },
    File {
        name: String,
        content: Option<String>,
    },
}
