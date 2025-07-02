use std::path::{Path, Component};
use crate::utils::store::AutoTagRule;

pub fn tags_for_path(path: &Path, rules: &[AutoTagRule]) -> Vec<String> {
    let mut components = Vec::new();
    if let Some(parent) = path.parent() {
        for comp in parent.components() {
            if let Component::Normal(os) = comp {
                if let Some(s) = os.to_str() {
                    components.push(s.to_string());
                }
            }
        }
    }

    let mut tags = Vec::new();
    for rule in rules {
        if components.iter().any(|c| c == &rule.folder) {
            for t in rule.tags.split_whitespace() {
                if !t.is_empty() && !tags.contains(&t.to_string()) {
                    tags.push(t.to_string());
                }
            }
        }
    }
    tags
}
