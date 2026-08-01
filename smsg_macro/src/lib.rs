use std::path::Path;

use heck::ToUpperCamelCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use prost::Message;
use prost_types::FileDescriptorSet;
use quote::quote;
use syn::{Ident, ItemMod, LitStr};

/// The `#[smsg("path/to/descriptors.pb")]` attribute macro.
///
/// Reads a protoc `FileDescriptorSet` at compile time and generates
/// `impl soul_msg::EnvelopeMeta` blocks for every message in the package that
/// matches the annotated module's name.
///
/// Usage:
/// ```ignore
/// #[smsg("descriptors.pb")]
/// pub mod chat {
///     include!(concat!(env!("OUT_DIR"), "/chat.rs"));
/// }
/// ```
///
/// The path is resolved relative to `CARGO_MANIFEST_DIR`. The module must contain
/// the prost-generated types for the matching package (e.g. via `include!`).
#[proc_macro_attribute]
pub fn smsg(attr: TokenStream, item: TokenStream) -> TokenStream {
    match expand(attr, item) {
        Ok(ts) => ts.into(),
        Err(msg) => {
            let msg = msg.to_string();
            quote! { compile_error!(#msg) }.into()
        }
    }
}

fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let path_lit: LitStr = syn::parse(attr)?;
    let item_mod: ItemMod = syn::parse(item)?;
    let mod_name = item_mod.ident.to_string();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| syn::Error::new(Span::call_site(), "CARGO_MANIFEST_DIR is not set"))?;
    let full_path = Path::new(&manifest_dir).join(path_lit.value());
    let bytes = std::fs::read(&full_path).map_err(|e| {
        syn::Error::new(
            Span::call_site(),
            format!(
                "failed to read descriptor set '{}': {}",
                full_path.display(),
                e
            ),
        )
    })?;

    let set = FileDescriptorSet::decode(bytes.as_slice()).map_err(|e| {
        syn::Error::new(Span::call_site(), format!("invalid descriptor set: {}", e))
    })?;

    let mut impls: Vec<syn::Item> = Vec::new();
    for file in &set.file {
        if file.package() != mod_name {
            continue;
        }
        for msg in &file.message_type {
            let short = msg.name();
            let full_name = smsg_core::full_message_name(&mod_name, short);
            let name_hash = smsg_core::compute_name_hash(short);
            let version_hash = smsg_core::compute_message_version_hash(msg, &full_name);
            let type_ident = Ident::new(&to_rust_type_name(short), Span::call_site());
            let nh = array_literal(&name_hash);
            let vh = array_literal(&version_hash);
            impls.push(syn::parse_quote! {
                impl soul_msg::EnvelopeMeta for #type_ident {
                    const NAME_HASH: [u8; 32] = #nh;
                    const VERSION_HASH: [u8; 32] = #vh;
                    const FULL_NAME: &'static str = #full_name;
                }
            });
        }
    }

    if impls.is_empty() {
        return Err(syn::Error::new(
            Span::call_site(),
            format!(
                "no messages found in package '{}' of the descriptor set; \
                 the module name must match the .proto package",
                mod_name
            ),
        ));
    }

    let mut item_mod = item_mod;
    if let Some((_, items)) = item_mod.content.as_mut() {
        items.extend(impls);
    }

    Ok(quote!(#item_mod))
}

fn to_rust_type_name(proto_name: &str) -> String {
    // prost-build converts message names to UpperCamelCase and sanitizes
    // identifiers (keywords, leading digits). Mirrors that here.
    sanitize_identifier(proto_name.to_upper_camel_case())
}

fn sanitize_identifier(ident: String) -> String {
    const RUST_KEYWORDS: &[&str] = &[
        "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "crate",
        "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "if", "impl", "in",
        "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
        "return", "Self", "self", "static", "struct", "super", "trait", "true", "try", "type",
        "typeof", "unsafe", "unsized", "use", "virtual", "where", "while", "yield", "_",
    ];
    if RUST_KEYWORDS.contains(&ident.as_str()) {
        format!("{}_", ident)
    } else if ident.starts_with(|c: char| c.is_numeric()) {
        format!("_{}", ident)
    } else {
        ident
    }
}

fn array_literal(bytes: &[u8; 32]) -> proc_macro2::TokenStream {
    let parts = bytes
        .iter()
        .map(|b| proc_macro2::Literal::u8_unsuffixed(*b));
    quote!([#(#parts),*])
}
