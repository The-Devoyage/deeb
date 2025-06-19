use std::path::Path;

pub fn create_rules(path: String) -> Result<(), std::io::Error> {
    let path_ref = Path::new(&path);
    if path_ref.exists() {
        log::error!("❌ A rules file already exists at: {}", path);
        std::process::exit(1);
    }
    std::fs::write(
        &path,
        r#"// Default rules.rhai
// operations: "insert_one" | "insert_many" | "find_one" | "find_many" | "update_one" | "update_many" | "delete_one" | "delete_many" 
// request: {
//   user: {
//     _id: String
//   }
// }

// Dynamically modify a query from the client
fn apply_query(entity, operation, user, payload) {
    if entity == "user" {
        if user == () {
            throw "User does not exist.";
        }

        return #{ "Eq": ["_id", request.user._id] }
    }

    // Return nothing to allow the user query without modification.
    return
}

fn check_rule(entity, operation, request, resource) {
    if entity == "user" {
        return users_rules(operation, request, resource);
    }

    // default: deny
    return false;
}

// Rules for `users` collection
fn users_rules(operation, request, resource) {
    // Only the user can read/update themselves
    if operation == "find_one" || operation == "find_many" || operation == "update_one" {
        return request.user._id == resource.id;
    }

    return false;
}
"#,
    )?;
    log::info!("Default rules file created at: {}", path);
    Ok(())
}
