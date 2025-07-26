const DEFAULT_RULES: &str = r#"
fn can_access(entity, operation, request, resource) {
    // default: allow all
    return true;
}
"#;

pub fn load_rules(path: Option<String>) -> String {
    match path {
        Some(p) if std::path::Path::new(&p).exists() => {
            let script = std::fs::read_to_string(p).expect("Failed to read rules file");
            script
        }
        _ => {
            log::warn!(
                "⚠️  No rules file provided or file not found. Falling back to default ALLOW ALL rules. ⚠️ "
            );
            DEFAULT_RULES.to_string()
        }
    }
}
