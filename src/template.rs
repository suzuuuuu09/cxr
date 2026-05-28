use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
// テンプレート全体を表す構造体
pub struct Template {
    pub name: String,
    pub description: String,
    pub variables: Option<Vec<String>>,
    pub items: Vec<TemplateItem>,
}

#[derive(Debug, Serialize, Deserialize)]
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
