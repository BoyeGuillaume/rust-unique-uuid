# unique-uuid

A Rust library that provides stable, cross-platform unique identifiers for types using UUIDs.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
unique-uuid = "0.1.0"
```

## Usage

### Basic Type Tagging

```rust
use unique_uuid::UniqueTypeTag;

#[derive(UniqueTypeTag)]
struct MyType;

// Get the unique identifier for the type
let type_id = MyType::TYPE_TAG;
```

### Custom Tags

```rust
use unique_uuid_derive::unique_tag;

// Generate a unique identifier for a custom tag
let custom_id = unique_tag!("my-custom-tag");
```

## How It Works

The library generates and stores UUIDs in a `types.toml` file to ensure consistency across different compilations. Unlike `std::any::TypeId`, these identifiers remain stable across different builds and platforms.

## ⚠️ Known Limitations

Currently, structs with the same name (even in different modules or crates) will receive the same UUID. This is a known bug that will be addressed in future versions. Please ensure your struct names are unique across your entire project.

## Storage

UUIDs are stored in a `types.toml` file with the following structure:

```toml
[unique_tags]
"custom-tag" = "uuid-value"

[unique_type_tags]
"MyType" = "uuid-value"
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
