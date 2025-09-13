# Deeb - JSON Database Ecosystem

Call it "Deeb," "D-b," or "That Cool JSON Thing"—this is a complete JSON database ecosystem perfect for tiny sites, rapid experiments, and lightweight applications.

Inspired by the flexibility of MongoDB and the lightweight nature of SQLite, Deeb transforms JSON files into a powerful, ACID-compliant database system with both embedded and server-based solutions.

## 🚀 Quick Overview

Deeb is more than just a database—it's a complete ecosystem consisting of four specialized crates that work together to provide a flexible, lightweight data storage solution including Deeb(Client), Deeb Server, Deeb Core, and Deeb Macros(macro support).

## 📦 Crates Overview

### 🎯 **Deeb** - Core Database Library
The main embedded database library that turns JSON files into a lightweight, ACID-compliant database.

**Perfect for:**
- Embedded applications
- Rapid prototyping
- Small to medium-sized datasets
- Applications requiring human-readable data storage

[📖 **Full Deeb Documentation & Quick Start →**](./deeb/README.md)

### 🌐 **Deeb Server** - HTTP API Server
A complete web server built on top of Deeb with authentication, access control, and RESTful APIs.

**Perfect for:**
- Web applications
- API backends
- Multi-user applications
- Remote database access

**Key Features:**
- Built-in user authentication (JWT-based)
- Flexible access control rules using Rhai scripting
- RESTful API endpoints
- Dynamic entity creation
- Applied queries for row-level security

**Quick Start:**
```bash
# Install
cargo install deeb-server

# Initialize rules
deeb-server init-rules

# Run server
deeb-server serve --rules ./rules.rhai
```

[📖 **Deeb Server Documentation →**](./deeb_server/README.md)

### ⚙️ **Deeb Core** - Database Engine
The foundational library containing the core database operations, transaction management, and storage engine.

**Provides:**
- ACID transaction support
- File-based storage with locking
- Query processing engine
- Index management
- Data persistence layer

### 🔧 **Deeb Macros** - Procedural Macros
Compile-time macros that provide the ergonomic `#[derive(Collection)]` interface and associated functionality.

**Enables:**
- Automatic collection trait implementation
- Type-safe database operations
- Compile-time entity validation
- Streamlined API usage

## 🎯 Choose Your Path

### For Embedded Applications
Use **Deeb** directly in your Rust applications:

```bash
cargo add deeb
```

```rust
use deeb::*;
use serde::{Serialize, Deserialize};

#[derive(Collection, Serialize, Deserialize)]
#[deeb(name = "user", primary_key = "id")]
struct User {
    id: i32,
    name: String,
    email: String,
}

// Full example in deeb/README.md
```

### For Web Applications
Use **Deeb Server** for HTTP-based access:

```bash
# Install the server
cargo install deeb-server

# Start serving your JSON database over HTTP
deeb-server serve --rules ./rules.rhai
```

```bash
# Insert data via HTTP
curl -X POST \
  -H 'Content-Type: application/json' \
  -d '{"document": {"name": "John", "email": "john@example.com"}}' \
  http://localhost:8080/insert-one/user
```

## ✨ Key Features

- **🔒 ACID Compliant**: Full transaction support with rollback capabilities
- **📄 JSON-Based**: Human-readable storage that's easy to inspect and modify
- **🚀 Schemaless**: No predefined schema required—adapt on the fly
- **🔍 Advanced Querying**: Complex queries with nested conditions
- **📊 Indexing**: Speed up queries with single and multi-field indexes
- **🔐 Authentication**: Built-in user management (Deeb Server)
- **🛡️ Access Control**: Flexible rule-based security (Deeb Server)
- **⚡ Lightweight**: Minimal dependencies and fast performance

## 🛠️ Development Status

Both Deeb and Deeb Server are under active development. Deeb is more mature and stable, while Deeb Server is newer and evolving rapidly.

## 📚 Documentation

- **[Deeb Database →](./deeb/README.md)** - Complete guide for the embedded database
- **[Deeb Server →](./deeb_server/README.md)** - HTTP server setup and API reference
- **[API Documentation →](https://docs.rs/deeb/latest/deeb/)** - Rust API docs
- **[Official Website →](https://www.deebkit.com)** - Docs, Examples, tutorials, and more

## 🤝 Contributing

We welcome contributions! Whether it's bug fixes, feature additions, or documentation improvements, please feel free to open issues and pull requests.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Ready to get started?** Check out the [Deeb Quick Start Guide](./deeb/README.md) for embedded usage or [Deeb Server Guide](./deeb_server/README.md) for web applications! Or explore [Deebkit](https://www.deebkit.com) for more information.
