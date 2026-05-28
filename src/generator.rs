use crate::template::TemplateItem;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

/// テンプレートアイテムのリストを処理して、ディレクトリやファイルを作成する関数
///
/// * `items`: テンプレートアイテムのリスト
/// * `base_path`: 作成するディレクトリやファイルの基準となるパス
/// * `variable_map`: テンプレート内の変数を置換するためのマップ
pub fn create_items(
    items: &[TemplateItem],
    base_path: &Path,
    variable_map: &HashMap<String, String>,
) {
    for item in items {
        match item {
            TemplateItem::Directory {
                name,
                items: sub_items,
            } => {
                let dir_path = base_path.join(name);
                println!("Creating directory: {:?}", dir_path);
                match std::fs::create_dir_all(&dir_path) {
                    Ok(_) => println!("Directory '{:?}' created successfully.", dir_path),
                    Err(e) => eprintln!("Failed to create directory '{:?}': {}", dir_path, e),
                }
                if let Some(inner_items) = sub_items {
                    create_items(inner_items, &dir_path, variable_map);
                }
            }
            TemplateItem::File { name, content } => {
                let file_path = base_path.join(name);
                println!("Creating file: {:?}", file_path);
                match File::create(&file_path) {
                    Ok(mut file) => {
                        use std::io::Write;
                        if let Some(content_str) = content {
                            // 変数の置換を行う
                            let mut content_str = content_str.clone();
                            for (key, val) in variable_map {
                                let target = format!("{{{{ {} }}}}", key);
                                content_str = content_str.replace(&target, val);
                            }
                            if let Err(e) = file.write_all(content_str.as_bytes()) {
                                eprintln!("Failed to write to file '{:?}': {}", file_path, e);
                            } else {
                                println!("File '{:?}' created successfully.", file_path);
                            }
                        } else {
                            println!(
                                "File '{:?}' created successfully (empty content).",
                                file_path
                            );
                        }
                    }
                    Err(e) => eprintln!("Failed to create file '{:?}': {}", file_path, e),
                }
            }
        }
    }
}
