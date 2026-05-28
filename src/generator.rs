use crate::template::TemplateItem;
use colored::*;
use inquire::Confirm;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

use crate::overwrite::OverwriteStrategy;

/// テンプレートアイテムのリストを処理して、ディレクトリやファイルを作成する関数
///
/// * `items`: テンプレートアイテムのリスト
/// * `base_path`: 作成するディレクトリやファイルの基準となるパス
/// * `variable_map`: テンプレート内の変数を置換するためのマップ
pub fn create_items(
    items: &[TemplateItem],
    base_path: &Path,
    variable_map: &HashMap<String, String>,
    dry_run: bool,
    overwrite: OverwriteStrategy,
) {
    for item in items {
        match item {
            TemplateItem::Directory {
                name,
                items: sub_items,
            } => {
                let resolved_dir_name = apply_vars(name.as_str(), variable_map);

                let dir_path = base_path.join(&resolved_dir_name);

                if dry_run {
                    if dir_path.exists() {
                        match overwrite {
                            OverwriteStrategy::Backup => println!(
                                "  {} {} -> {}",
                                "Would backup dir:".yellow().bold(),
                                resolved_dir_name.as_str(),
                                backup_name(&resolved_dir_name)
                            ),
                            _ => println!(
                                "  {} {}",
                                "Would reuse dir:".yellow().bold(),
                                resolved_dir_name.as_str()
                            ),
                        }
                    } else {
                        println!(
                            "  {} {}",
                            "Would create dir:".yellow().bold(),
                            resolved_dir_name.as_str()
                        );
                    }
                } else if matches!(overwrite, OverwriteStrategy::Backup) && dir_path.exists() {
                    if let Err(e) = backup_dir(&dir_path, &resolved_dir_name) {
                        eprint_error(
                            &format!("Failed to backup directory '{}'", resolved_dir_name),
                            &e,
                        );
                        continue;
                    }
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
                } else {
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
                }

                if let Some(inner_items) = sub_items {
                    create_items(inner_items, &dir_path, variable_map, dry_run, overwrite);
                }
            }
            TemplateItem::File { name, content } => {
                let resolved_file_name = apply_vars(name.as_str(), variable_map);

                let file_path = base_path.join(&resolved_file_name);

                if dry_run {
                    if file_path.exists() {
                        match overwrite {
                            OverwriteStrategy::Skip => println!(
                                "  {} {}",
                                "Would skip file:".yellow().bold(),
                                resolved_file_name
                            ),
                            OverwriteStrategy::Backup => println!(
                                "  {} {} -> {}",
                                "Would backup file:".yellow().bold(),
                                resolved_file_name,
                                backup_name(&resolved_file_name)
                            ),
                            OverwriteStrategy::Force => println!(
                                "  {} {}",
                                "Would overwrite file:".yellow().bold(),
                                resolved_file_name
                            ),
                            OverwriteStrategy::Prompt => println!(
                                "  {} {}",
                                "Would prompt overwrite:".yellow().bold(),
                                resolved_file_name
                            ),
                        }
                    } else if content.is_some() {
                        println!(
                            "  {} {}",
                            "Would create file:".yellow().bold(),
                            resolved_file_name
                        );
                    } else {
                        println!(
                            "  {} {} (empty)",
                            "Would create file:".yellow().bold(),
                            resolved_file_name
                        );
                    }
                    continue;
                }

                if file_path.exists() {
                    match overwrite {
                        OverwriteStrategy::Force => {}
                        OverwriteStrategy::Backup => {
                            if let Err(e) = backup_file(&file_path, &resolved_file_name) {
                                eprint_error(
                                    &format!("Failed to backup file '{}'", resolved_file_name),
                                    &e,
                                );
                                continue;
                            }
                        }
                        OverwriteStrategy::Skip => {
                            println!("   {} {}", "Skipped:".yellow(), resolved_file_name);
                            continue;
                        }
                        OverwriteStrategy::Prompt => {
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
                    }
                }

                // ファイルの作成
                match File::create(&file_path) {
                    Ok(mut file) => {
                        use std::io::Write;
                        if let Some(content_str) = content {
                            // ファイル中身の変数を置換する
                            let content_to_write = apply_vars(&content_str, variable_map);
                            if let Err(e) = file.write_all(content_to_write.as_bytes()) {
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

fn apply_vars(input: &str, variable_map: &HashMap<String, String>) -> String {
    let mut resolved = input.to_string();
    for (key, val) in variable_map {
        let target = format!("{{{{ {} }}}}", key);
        resolved = resolved.replace(&target, val);
    }
    resolved
}

// エラー表示用の共通ヘルパー関数
fn eprint_error(context: &str, err: &str) {
    eprintln!("{} {}: {}", "Error:".red().bold(), context, err);
}

fn backup_name(name: &str) -> String {
    format!("{}.bak", name)
}

fn backup_path(path: &Path, name: &str) -> PathBuf {
    path.with_file_name(backup_name(name))
}

fn backup_file(path: &Path, name: &str) -> Result<(), String> {
    let backup = backup_path(path, name);
    if backup.exists() {
        std::fs::remove_file(&backup).map_err(|e| e.to_string())?;
    }
    std::fs::rename(path, backup).map_err(|e| e.to_string())
}

fn backup_dir(path: &Path, name: &str) -> Result<(), String> {
    let backup = backup_path(path, name);
    if backup.exists() {
        std::fs::remove_dir_all(&backup).map_err(|e| e.to_string())?;
    }
    std::fs::rename(path, backup).map_err(|e| e.to_string())
}
