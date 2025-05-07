# paginator-rs

A modular Rust library providing a reusable trait for paginating any serializable collection. Built using Cargo workspace structure for flexibility and clarity.

## 🧱 Workspace Structure

```
root/
├── paginator-rs/         # Main trait and implementation logic
│   ├── src/lib.rs
│   └── Cargo.toml
├── paginator-utils/      # Shared types and helper functions (e.g. response format)
│   ├── src/lib.rs
│   └── Cargo.toml
├── paginator-examples/   # Usage examples and test cases
│   ├── src/lib.rs
│   ├── src/examples/
│   └── Cargo.toml
├── Cargo.toml            # Root workspace config
└── README.md
```

## ✨ Features

- `PaginatorTrait<T>` for consistent pagination logic
- `PaginatorResponse<T>` as a standardized response structure
- JSON serialization support using [`serde`](https://serde.rs)
- Clean separation of logic, utilities, and examples

## 📦 Installation

To use in your project, add this to your `Cargo.toml`:

```toml
[dependencies]
paginator-rs = "0.1.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

> This project will be published to [crates.io](https://crates.io) soon 🚀

## 🚀 Usage Example

```rust
use paginator_rs::{PaginatorResponse, PaginatorResponseMeta, PaginatorTrait};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UsersData {
    pub id: u32,
    pub name: String,
    pub email: String,
}

impl UsersData {
    pub fn new(id: u32, name: String, email: String) -> Self {
        UsersData { id, name, email }
    }
}

impl PaginatorTrait<UsersData> for Vec<UsersData> {
    fn paginate(&self) -> PaginatorResponse<UsersData> {
        let mut data = self.clone();
        data.sort_by(|a, b| a.id.cmp(&b.id));
        PaginatorResponse {
            data,
            meta: PaginatorResponseMeta {
                page: 1,
                per_page: self.len() as u32,
                total: self.len() as u32,
            },
        }
    }
}


fn main() {
    let users = vec![
        UsersData::new(1, "John Doe".to_string(), "john@doe.com".to_string()),
        UsersData::new(2, "Jane Doe".to_string(), "jane@doe.com".to_string()),
        UsersData::new(3, "Bob Doe".to_string(), "bob@doe.com".to_string()),
    ];
    println!("{:#?}", users.paginate());
    println!("{}", users.paginate_json());
}

```

## 🧪 Sample Output

```json
{
  "data": [
    { "id": 1, "name": "Alpha" },
    { "id": 2, "name": "Beta" }
  ],
  "meta": {
    "page": 1,
    "per_page": 2,
    "total": 2
  }
}
```

## 📄 License

MIT © 2025 Maulana Sodiqin
