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
        if ["find_one", "find_many", "delete_many", "delete_one", "update_one", "update_many"].contains(operation) {
            return #{ "Eq": [ "_created_by", user._id ] } 
        } 

        throw "Operation not permitted."
    }

    // if entity == "command" {
    //     if user == () {
    //         throw "User does not exist.";
    //     }
    //     return 
    // }

    return
}

fn check_rule(entity, operation, request, resource) {
    if entity == "dog" {
        return true
    }

    if entity == "user" {
        return true
    }

    if entity == "command" {
        return true
    }

    // default: deny
    return false
}
