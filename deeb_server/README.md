# Deeb Server

A web server on top of Deeb DB with User Auth and Access Control.

Use with caution, undergoing active development!

## Getting Started

1. Install Deeb Server

```bash
cargo install deeb-server
```

2. Configure Instances

Instances are how deeb organizes data. Create an `instances.json` file providing the instance name, entities, assocciations, and indexes.

If using `deeb-server`, as a companion to the deeb client, you can use the `save_instance_config` function within the client to automate the export of the configuration. See the Deeb Readme for more.

```json
{
  "instance_name": {
    "entities": [
      {
        "name": "user",
        "primary_key": "id",
        "associations": [
          {
            "entity_name": "comment",
            "from": "id",
            "to": "user_id",
            "alias": "user_comment"
          }
        ],
        "indexes": [
          {
            "name": "age_index",
            "keys": [
              "age"
            ],
            "options": null
          },
          {
            "name": "name_age_index",
            "keys": [
              "name",
              "age"
            ],
            "options": null
          }
        ]
      },
      {
        "name": "comment",
        "primary_key": "_id",
        "associations": [
          {
            "entity_name": "user",
            "from": "user_id",
            "to": "id",
            "alias": "user"
          }
        ],
        "indexes": []
      }
    ]
  }
}
```

2. Init Rules & Edit Rules

```
deeb-server init-rules
```

- By defaults, all operations are blocked. Enable operations by editing the generated rules file after running the command above.
- There are two types of rules:
    1. Applied Queries - Dynamically force operations to include specific filters to limit access control.
    2. Rules - Write custom rules to enforce access control.

3. Run the server

Point the CLI to the rules file, optionally adjust the host/port, and spin it up.

```bash
deeb-server serve --rules ./rules.rhai
```

## Usage

### Query your server

Once your HTTP server is running, you can start to query it. Deeb functions as a NoSQL database and
there is no schema! This means you can just insert, find, update, and delete freely.

**Endpoints**

- Endpoints follow a simple pattern - `/operation/entity-name`
- All endpoints are POST requests.
- Entity names are dynamic. Use it, and it gets created. Be sure your rules allow you to access the entity.
- If your rules file requires authenticated users, the optional Authorization header is required. See below for rules
docs.

**Examples**

Endpoint Examples:
- /find-one/user
- /find-many/comment
- /insert-one/post
- /insert-many/like
- /update-one/friend
- /update-many/recipe

1. Insert One

Provide property `document` to insert the new document. Every document gets an auto generated `_id`, unless overridden.

```bash
curl -X POST \
    -H 'Authorization: Bearer ${TOKEN}' \
    -H 'Content-Type: application/json' \
    -d '{ document: { "title": "Hello World", "text": "This is my first post!" } }' \
    http://localhost:8080/insert-one/post
```

2. Find Many

Provide properties `query` and optionaly `options` to fetch documents.

```bash
curl -X POST \
    -H 'Authorization: Bearer ${TOKEN}' \
    -H 'Content-Type: application/json' \
    -d '{ "query": { "Eq": ["title", "Hello World"] }, "options": { "limit": 10 } } ' \
    http://localhost:8080/find-many/post
```


3. Update One

Provide properties `query` and `document` to update document.

```bash
curl -X POST \
    -H 'Authorization: Bearer ${TOKEN}' \
    -H 'Content-Type: application/json' \
    -d '{ "query": {"Eq": ["title", "Hello World"]}, "document": {"title": "Bizz Bazz" } }' \
    http://localhost:8080/update-one/post
```

3. Delete One

Provide property `query` to delete document.

```bash
curl -X POST \
    -H 'Authorization: Bearer ${TOKEN}' \
    -H 'Content-Type: application/json' \
    -d '{ "query": {"Eq": ["title", "Bizz Bazz"] }' \
    http://localhost:8080/update-one/post
```

### User Auth

Deeb server provides auth endpoints to manage user accounts for your application. Currently, only password based
authentication is supported.

1. Register a User

```bash
curl -X POST \
    -H 'Content-Type: application/json' \
    -d '{ "email": "user@domain.com", "password": "super_secret_password", "name": "John Doe" }' \
    http://localhost:8080/auth/register
```

2. Authenticate User

Returns JWT Token

Create a `.env` file in the same directory with the variable `JWT_SECRET` populated to sign the token.

```bash
curl -X POST \
    -H 'Content-Type: application/json' \
    -d '{ "email": "user@domain.com", "password": "super_secret_password" }' \
    http://localhost:8080/auth/login
```

2. Fetch Authorized User

```bash
curl -X POST \
    -H 'Content-Type: application/json' \
    -H 'Authorization: Bearer ${TOKEN}' \
    -d '{ "email": "user@domain.com", "password": "super_secret_password" }' \
    http://localhost:8080/auth/me
```

### Applied Queries and Rules

**Applied Queries**

Applied Queries are default queries that can be conditionally injected into the request at run time.

After generating a rules file with the cli, `deeb-server init-rules`, you can conditionally apply queries
to operations to control user access with default filters on the database.

```rust
fn apply_query(entity, operation, user, payload) {
    // Only apply the following rules to a `user` entity.
    if entity == "user" {
        // You can print to debug.
        print("Apply default user query.");

        // Require a logged in user.
        if user == () {
            // Throwing string errors automatically passes the error message to the network response.
            throw "User does not exist.";
        }

        // Only allow the user to find their own user object.
        if ["find_one", "find_many"].contains(operation) {
            // Return the default query which will be merged with the client provided query.
            return #{ "Eq": ["_id", request.user._id] }
        }

        // Don't accept other operations
        throw "User opertaion not permitted."
    }

    // Return to allow the client query without modification.
    return
}

```

**Rules**

```rust
fn check_rule(entity, operation, user, resource) {
    // Only apply the following rules to user entities
    if entity == "user" {
        // You can organize rules into functions.
        return users_rules(operation, request, resource);
    }

    // default: deny all access.
    return false;
}

// Rules for `users` collection
fn users_rules(operation, request, resource) {
    // The authetnicated user can only read/update their own record
    if operation == "find_one" || operation == "find_many" || operation == "update_one" {
        return request.user._id == resource.id;
    }

    // Deny all other operations
    return false;
}
```
