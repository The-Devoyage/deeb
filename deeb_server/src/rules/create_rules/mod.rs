use std::path::Path;

pub fn create_rules(path: String) -> Result<(), std::io::Error> {
    let path_ref = Path::new(&path);
    if path_ref.exists() {
        log::error!("❌ A rules file already exists at: {}", path);
        std::process::exit(1);
    }
    std::fs::write(
        &path,
        r#"
// Default rules.rhai
// operations: "insert_one" | "find_many" | "find_one" | "update_one" | "update_many" | "delete_one" | "delete_many" 
// request: {
//   user: {
//     _id: String
//   }
// }

fn can_access(entity, operation, request, resource) {
    if entity == "users" {
        return users_rules(operation, request, resource);
    }

    // default: deny
    return false;
}

// Rules for `users` collection
fn users_rules(operation, request, resource) {
    // Only the user can read/update themselves
    if operation == "read" || operation == "update" {
        return request.user._id == resource.id;
    }

    // Only admin can delete a user
    if operation == "delete" {
        return "admin" in request.user.roles;
    }

    // Allow creating a user only if email ends with our domain
    if operation == "create" {
        return request.resource.email.ends_with("@example.com");
    }

    return false;
}
"#,
    )?;
    log::info!("Default rules file created at: {}", path);
    Ok(())
}
