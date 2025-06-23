// Default rules.rhai
// operations: "insert_one" | "insert_many" | "find_one" | "find_many" | "update_one" | "update_many" | "delete_one" | "delete_many" 
// request: {
//   user: {
//     _id: String
//   }
// }

// Appends a default query to the operation
fn apply_query(entity, operation, user, payload) {
    if entity == "dog" {
        if user == () {
            throw "User does not exist.";
        }
        if ["find_one", "find_many"].contains(operation) {
            return #{ "Eq": [ "_created_by", user._id ] } 
        } 

        throw "Operation not permitted."
    }

    return
}

fn check_rule(entity, operation, request, resource) {
    print(entity == "dog");
    if entity == "dog" {
        return true
    }

    if entity == "user" {
        return true
    }

    // default: deny
    return false
}
