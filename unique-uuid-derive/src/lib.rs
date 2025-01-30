use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{Read, Seek, Write},
};

use proc_macro::TokenStream;
use serde::{Deserialize, Serialize};
use syn::spanned::Spanned;

static DEFAULT_TYPES_FILE_NAME: &str = "types.toml";

/// This macro allows for the automatic generation of [`unique_uuid::UniqueTag`] for
/// the provided string. Notice that if the string is the same, then the generated tag
/// will be the same. This work within a same crate, but no guarantees are made for
/// cross-crate compatibility. This means that if a tag is generated for `"my-tag-unique"`,
/// then another crate may generate a different tag for the same string.
///
/// # Example
/// ```rust
/// let tag = unique_tag!("my-tag-unique");
/// ```
#[proc_macro]
pub fn unique_tag(input: TokenStream) -> TokenStream {
    let string = syn::parse_macro_input!(input as syn::LitStr);
    let uuid = get_uuid_from_tag(&string.value(), UType::UniqueTags).to_string();
    let uuid = syn::LitStr::new(&uuid, string.span());

    TokenStream::from(quote::quote! {
        unique_uuid::UniqueTag(unique_uuid::uuid::uuid!(#uuid))
    })
}

/// This macro allows for the automatic generation of [`unique_uuid::UniqueTypeTag`] for
/// the provided type. This will generate a unique tag for the type, which will be the
/// same for the same type across crates. This is useful for serialization and deserialization
/// of types, where the type is used as a key.
///
/// # Example
/// ```rust
/// #[derive(UniqueTypeTag)]
/// pub struct MyType;
/// ```
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
