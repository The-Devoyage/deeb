use std::path::Path;

pub fn create_rules(path: String) -> Result<(), std::io::Error> {
    let path_ref = Path::new(&path);
    if path_ref.exists() {
        log::error!("❌ A rules file already exists at: {}", path);
        std::process::exit(1);
    }
    std::fs::write(
        &path,
        r#"// Rules Quick Start 
// entity: The name of the given collection. Ex: "user" | "comment" | "profile_image"
// operation: "insert_one" | "insert_many" | "find_one" | "find_many" | "update_one" | "update_many" | "delete_one" | "delete_many" 
// user: {
//     _id: Ulid, // (string)
//     email: String
// }
// payload: `request.body` of the provided request
// resource: The found document.

// Dynamically modify a query from the client
fn apply_query(entity, operation, user, payload) {
    if entity == "user" {
        print("Apply default user query.");

        // Require a logged in user.
        if user == () {
            throw "User does not exist.";
        }

        // Only allow the user to find their own user object.
        if ["find_one", "find_many"].contains(operation) {
            return #{ "Eq": ["_id", request.user._id] }
        }

        // Don't accept other operations
        throw "User opertaion not permitted."
    }

    // Return nothing to allow the user query without modification.
    return
}

fn check_rule(entity, operation, user, resource) {
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
