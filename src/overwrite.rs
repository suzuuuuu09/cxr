/// Centralized overwrite strategy enum used across the crate.
#[derive(Clone, Copy, Debug)]
pub enum OverwriteStrategy {
    Prompt,
    Force,
    Backup,
    Skip,
}
