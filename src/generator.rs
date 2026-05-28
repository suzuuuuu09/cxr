use crate::template::TemplateItem;
use colored::*;
use inquire::Confirm;
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
                let mut resolved_dir_name = name.clone();
                for (key, val) in variable_map {
                    let target = format!("{{{{ {} }}}}", key);
                    resolved_dir_name = resolved_dir_name.replace(&target, val);
                }

                let dir_path = base_path.join(&resolved_dir_name);

                match std::fs::create_dir_all(&dir_path) {
                    Ok(_) => println!(
                        "  {} {}",
                        "Created Dir:".green().bold(),
                        resolved_dir_name.as_str()
                    ),
                    Err(e) => eprint_error(
                        &format!("Failed to create directory '{}'", resolved_dir_name),
                        &e.to_string(),
                    ),
                }

                if let Some(inner_items) = sub_items {
                    create_items(inner_items, &dir_path, variable_map);
                }
            }
            TemplateItem::File { name, content } => {
                let mut resolved_file_name = name.clone();
                for (key, val) in variable_map {
                    let target = format!("{{{{ {} }}}}", key);
                    resolved_file_name = resolved_file_name.replace(&target, val);
                }

                let file_path = base_path.join(&resolved_file_name);

                if file_path.exists() {
                    let prompt_msg =
                        format!("File '{}' already exists. Overwrite?", resolved_file_name);
                    let ans = Confirm::new(&prompt_msg).with_default(false).prompt();

                    match ans {
                        Ok(true) => {} // そのまま処理を続行
                        _ => {
                            println!("   {} {}", "Skipped:".yellow(), resolved_file_name);
                            continue; // 次のアイテムの処理へスキップ
                        }
                    }
                }

                // ファイルの作成
                match File::create(&file_path) {
                    Ok(mut file) => {
                        use std::io::Write;
                        if let Some(content_str) = content {
                            // ファイル中身の変数を置換する
                            let mut content_str = content_str.clone();
                            for (key, val) in variable_map {
                                let target = format!("{{{{ {} }}}}", key);
                                content_str = content_str.replace(&target, val);
                            }
                            if let Err(e) = file.write_all(content_str.as_bytes()) {
                                eprint_error(
                                    &format!("Failed to write to file '{}'", resolved_file_name),
                                    &e.to_string(),
                                );
                            } else {
                                println!(
                                    "  {} {}",
                                    "Created File:".green().bold(),
                                    resolved_file_name
                                );
                            }
                        } else {
                            println!(
                                "  {} {} (empty)",
                                "Created File:".green().bold(),
                                resolved_file_name
                            );
                        }
                    }
                    Err(e) => eprint_error(
                        &format!("Failed to create file '{}'", resolved_file_name),
                        &e.to_string(),
                    ),
                }
            }
        }
    }
}

// エラー表示用の共通ヘルパー関数
fn eprint_error(context: &str, err: &str) {
    eprintln!("{} {}: {}", "Error:".red().bold(), context, err);
}
