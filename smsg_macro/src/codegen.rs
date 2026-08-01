//! Generates prost-style message structs + `EnvelopeMeta` impls from a parsed
//! protobuf file descriptor set. Used at compile time by the `#[smsg]` macro.

use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro2::{Ident, Span, TokenStream};
use prost_types::field_descriptor_proto::{Label, Type};
use prost_types::{DescriptorProto, FieldDescriptorProto, FileDescriptorSet};
use quote::quote;

const RUST_KEYWORDS: &[&str] = &[
    "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "crate",
    "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "if", "impl", "in",
    "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
    "return", "Self", "self", "static", "struct", "super", "trait", "true", "try", "type",
    "typeof", "unsafe", "unsized", "use", "virtual", "where", "while", "yield", "_",
];

fn sanitize_identifier(ident: String) -> String {
    if RUST_KEYWORDS.contains(&ident.as_str()) {
        format!("{}_", ident)
    } else if ident.starts_with(|c: char| c.is_numeric()) {
        format!("_{}", ident)
    } else {
        ident
    }
}

/// Generates one item per top-level message: a prost struct + its `EnvelopeMeta`.
pub fn generate(set: &FileDescriptorSet) -> Result<Vec<syn::Item>, String> {
    let mut all = proc_macro2::TokenStream::new();
    for file in &set.file {
        for msg in &file.message_type {
            all.extend(generate_message_tokens(file.package(), msg)?);
        }
    }
    let file: syn::File = syn::parse2(all).map_err(|e| format!("codegen parse: {}", e))?;
    Ok(file.items)
}

fn generate_message_tokens(package: &str, msg: &DescriptorProto) -> Result<TokenStream, String> {
    if !msg.nested_type.is_empty() {
        return Err(format!(
            "message '{}' uses nested types or map fields, which are not supported yet",
            msg.name()
        ));
    }
    if !msg.oneof_decl.is_empty() {
        return Err(format!(
            "message '{}' uses oneof, which is not supported yet",
            msg.name()
        ));
    }

    let struct_name = Ident::new(
        &sanitize_identifier(msg.name().to_upper_camel_case()),
        Span::call_site(),
    );
    let mut fields = Vec::new();
    for f in &msg.field {
        fields.push(generate_field(f)?);
    }

    let full_name = if package.is_empty() {
        msg.name().to_string()
    } else {
        format!("{}.{}", package, msg.name())
    };
    let name_hash = smsg_core::compute_name_hash(msg.name());
    let version_hash = smsg_core::compute_message_version_hash(msg, &full_name);
    let nh = array_literal(&name_hash);
    let vh = array_literal(&version_hash);
    let full_name_lit = full_name.as_str();

    Ok(quote! {
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct #struct_name {
            #(#fields)*
        }
        impl soul_msg::EnvelopeMeta for #struct_name {
            const NAME_HASH: [u8; 32] = #nh;
            const VERSION_HASH: [u8; 32] = #vh;
            const FULL_NAME: &'static str = #full_name_lit;
        }
    })
}

fn generate_field(f: &FieldDescriptorProto) -> Result<TokenStream, String> {
    if f.oneof_index.is_some() {
        return Err(format!(
            "field '{}' is part of a oneof, not supported yet",
            f.name()
        ));
    }

    let field_ident = Ident::new(
        &sanitize_identifier(f.name().to_snake_case()),
        Span::call_site(),
    );
    let tag = f.number();
    let repeated = f.label() == Label::Repeated;

    let (kind, base_ty) = field_kind_and_type(f)?;

    let (attr_ts, ty_ts) = if repeated {
        (
            quote! { #kind, repeated },
            quote! { ::std::vec::Vec<#base_ty> },
        )
    } else if f.r#type() == Type::Message {
        // Singular message fields are optional in proto3.
        (quote! { #kind }, quote! { ::std::option::Option<#base_ty> })
    } else {
        (quote! { #kind }, base_ty)
    };

    Ok(quote! {
        #[prost(#attr_ts, tag = #tag)]
        pub #field_ident: #ty_ts,
    })
}

/// Returns the prost derive type keyword and the base Rust type (no Vec/Option
/// wrapper) for a field.
fn field_kind_and_type(f: &FieldDescriptorProto) -> Result<(TokenStream, TokenStream), String> {
    let ty = f.r#type();
    Ok(match ty {
        Type::String => (quote! { string }, quote! { String }),
        Type::Bytes => (quote! { bytes }, quote! { ::std::vec::Vec<u8> }),
        Type::Bool => (quote! { bool }, quote! { bool }),
        Type::Int32 => (quote! { int32 }, quote! { i32 }),
        Type::Int64 => (quote! { int64 }, quote! { i64 }),
        Type::Uint32 => (quote! { uint32 }, quote! { u32 }),
        Type::Uint64 => (quote! { uint64 }, quote! { u64 }),
        Type::Sint32 => (quote! { sint32 }, quote! { i32 }),
        Type::Sint64 => (quote! { sint64 }, quote! { i64 }),
        Type::Fixed32 => (quote! { fixed32 }, quote! { u32 }),
        Type::Fixed64 => (quote! { fixed64 }, quote! { u64 }),
        Type::Sfixed32 => (quote! { sfixed32 }, quote! { i32 }),
        Type::Sfixed64 => (quote! { sfixed64 }, quote! { i64 }),
        Type::Float => (quote! { float }, quote! { f32 }),
        Type::Double => (quote! { double }, quote! { f64 }),
        Type::Message => {
            let tn = f.type_name();
            let name = tn.rsplit('.').next().unwrap_or(tn);
            let ident = Ident::new(
                &sanitize_identifier(name.to_upper_camel_case()),
                Span::call_site(),
            );
            (quote! { message }, quote! { #ident })
        }
        Type::Enum => {
            return Err(format!(
                "field '{}' uses an enum, not supported yet",
                f.name()
            ))
        }
        Type::Group => {
            return Err(format!(
                "field '{}' uses a group, not supported yet",
                f.name()
            ))
        }
    })
}

fn array_literal(bytes: &[u8; 32]) -> TokenStream {
    let parts = bytes
        .iter()
        .map(|b| proc_macro2::Literal::u8_unsuffixed(*b));
    quote!([#(#parts),*])
}
