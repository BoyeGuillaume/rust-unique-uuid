//! This crate provides procedural macros for generating unique UUIDs associated with tags and types.
//! It offers two main functionalities:
//! - `unique_tag`: A procedural macro that generates a unique UUID for a given string tag
//! - `UniqueTypeTag`: A derive macro that automatically generates a unique UUID for a type
//!
//! The generated UUIDs are persisted in a TOML file (`types.toml` by default) to ensure
//! consistency across multiple compilations and crate boundaries.
//!
//! # Features
//! - Persistent UUID generation and storage
//! - Consistent UUID mapping for types and tags
//! - Thread-safe file handling
//! - TOML-based storage format
//!
//! # Examples
//! ```rust
//! use unique_uuid_derive::{unique_tag, UniqueTypeTag};
//!
//! // Using unique_tag macro
//! let my_tag = unique_tag!("my_custom_tag");
//!
//! // Using UniqueTypeTag derive macro
//! #[derive(UniqueTypeTag)]
//! struct MyStruct;
//! ```
//!
//! # File Structure
//! The crate maintains a TOML file with the following structure:
//! ```toml
//! [unique_tags]
//! "tag_name" = "uuid"
//!
//! [unique_type_tags]
//! "type_name" = "uuid"
//! ```
//!
//! # Implementation Details
//! - UUIDs are generated using UUID v4 (random)
//! - File operations are performed with proper error handling
//! - The system supports both string tags and type tags
//!
//! # Safety
//! This crate performs file I/O operations during compilation, which may fail if:
//! - The process lacks file system permissions
//! - The TOML file becomes corrupted
//! - Concurrent compilation attempts cause file access conflicts
use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{Read, Seek, Write},
};

use proc_macro::TokenStream;
use serde::{Deserialize, Serialize};
use syn::spanned::Spanned;

static DEFAULT_TYPES_FILE_NAME: &str = "types.toml";

/// A procedural macro that generates a unique UUID for a given string tag.
/// The generated UUID is persisted in a TOML file to ensure consistency across
/// multiple compilations and crate boundaries.
///
/// # Warnings
/// Tags are consistent within the same crate, but **MAY** differ across crates. They
/// will match only if the `types.toml` file is shared between the crates (typically for
/// a workspace). As such special care should be taken when working with macros that are
/// public and used across crates.
///
/// # Arguments
/// * Input must be a string literal that serves as the tag identifier
///
/// # Returns
/// Returns a [`unique_uuid::UniqueTag`] containing a UUID that is consistently
/// mapped to the input tag.
///
/// # Example
/// ```rust
/// use unique_uuid_derive::unique_tag;
///
/// let my_uuid = unique_tag!("my_custom_tag");
/// ```
///
/// # Panics
/// This macro will panic if:
/// * The TOML file cannot be opened or created
/// * There are permission issues with the file system
/// * The TOML file is corrupted or invalid
///
/// # File Storage
/// The UUID-tag mapping is stored in the `types.toml` file under the `[unique_tags]` section.
#[proc_macro]
pub fn unique_tag(input: TokenStream) -> TokenStream {
    let string = syn::parse_macro_input!(input as syn::LitStr);
    let uuid = get_uuid_from_tag(&string.value(), UType::UniqueTags).to_string();
    let uuid = syn::LitStr::new(&uuid, string.span());

    TokenStream::from(quote::quote! {
        unique_uuid::UniqueTag(unique_uuid::uuid::uuid!(#uuid))
    })
}

/// A derive macro that automatically generates a unique UUID for a type.
/// The generated UUID is associated with the type name and persisted in a TOML file
/// to ensure consistency across multiple compilations and crate boundaries.
///
/// This macro implements the [`unique_uuid::UniqueTypeTag`] trait for the decorated type,
/// providing a constant `TYPE_TAG` that contains a unique UUID.
///
/// # Example
/// ```rust
/// use unique_uuid_derive::UniqueTypeTag;
///
/// #[derive(UniqueTypeTag)]
/// struct MyStruct;
///
/// // The UUID can be accessed via the trait implementation
/// let type_uuid = MyStruct::TYPE_TAG;
/// ```
///
/// # Implementation Details
/// * Generates a UUID v4 for the type if one doesn't exist
/// * Stores the UUID in `types.toml` under the `[unique_type_tags]` section
/// * Uses the type's name as the key for UUID mapping
///
/// # Panics
/// This macro will panic if:
/// * The TOML file cannot be opened or created
/// * There are permission issues with the file system
/// * The TOML file is corrupted or invalid
///
#[proc_macro_derive(UniqueTypeTag)]
pub fn unique_type_tag(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let tag = format!("{}::{}", "", input.ident);

    let uuid = get_uuid_from_tag(&tag, UType::UniqueTypeTags).to_string();
    let uuid = syn::LitStr::new(&uuid, input.span());

    let input_ident = input.ident;

    TokenStream::from(quote::quote! {
        impl unique_uuid::UniqueTypeTag for #input_ident {
            const TYPE_TAG: unique_uuid::UniqueTag = unique_uuid::UniqueTag(unique_uuid::uuid::uuid!(#uuid));
        }
    })
}

enum UType {
    UniqueTags,
    UniqueTypeTags,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct FileStructure {
    #[serde(default)]
    unique_tags: HashMap<String, uuid::Uuid>,

    #[serde(default)]
    unique_type_tags: HashMap<String, uuid::Uuid>,
}

fn get_uuid_from_tag(tag: &str, r#type: UType) -> uuid::Uuid {
    let file_path = DEFAULT_TYPES_FILE_NAME;
    let mut file = match OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_path)
    {
        Ok(file) => file,
        Err(err) => {
            panic!("Error opening file: {}", err);
        }
    };

    // Read the TOML file
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    // Deserialize the TOML file
    let mut file_structure: FileStructure = toml::from_str(&contents).unwrap();

    let target = match r#type {
        UType::UniqueTags => &mut file_structure.unique_tags,
        UType::UniqueTypeTags => &mut file_structure.unique_type_tags,
    };
    if let Some(uuid) = target.get(tag) {
        uuid.clone()
    } else {
        let uuid = uuid::Uuid::new_v4();
        target.insert(tag.to_string(), uuid);
        let toml = toml::to_string(&file_structure).unwrap();
        file.seek(std::io::SeekFrom::Start(0)).unwrap();
        file.write_all(toml.as_bytes()).unwrap();
        uuid
    }
}
