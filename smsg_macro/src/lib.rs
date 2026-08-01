mod codegen;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse::Parser;
use syn::{ItemMod, LitStr};

/// The `#[smsg("path/to/chat.proto", ...)]` attribute macro.
///
/// Parses the given `.proto` files at compile time (with the pure-Rust `protox`
/// compiler — no protoc, no build script, no `OUT_DIR`) and generates, inside the
/// annotated module:
///
/// - a prost-style struct per message (via `#[derive(::prost::Message)]`), and
/// - an `impl soul_msg::EnvelopeMeta` block per message.
///
/// Paths are resolved relative to `CARGO_MANIFEST_DIR`. The consumer crate must
/// depend on `soul_msg` and `prost`.
///
/// ```ignore
/// #[smsg("proto/chat.proto")]
/// pub mod chat {}
/// ```
#[proc_macro_attribute]
pub fn smsg(attr: TokenStream, item: TokenStream) -> TokenStream {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| expand(attr, item)));
    match result {
        Ok(Ok(ts)) => ts.into(),
        Ok(Err(e)) => {
            let msg = e.to_string();
            quote! { compile_error!(#msg) }.into()
        }
        Err(payload) => {
            let msg = if let Some(s) = payload.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = payload.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "unknown panic".to_string()
            };
            let msg = format!("smsg macro panicked: {}", msg);
            quote! { compile_error!(#msg) }.into()
        }
    }
}
fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let paths: syn::punctuated::Punctuated<LitStr, syn::Token![,]> =
        syn::punctuated::Punctuated::parse_terminated.parse(attr)?;
    if paths.is_empty() {
        return Err(syn::Error::new(
            Span::call_site(),
            "expected at least one `.proto` path, e.g. #[smsg(\"proto/chat.proto\")]",
        ));
    }

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| syn::Error::new(Span::call_site(), "CARGO_MANIFEST_DIR is not set"))?;
    let manifest = Path::new(&manifest_dir);

    let proto_paths: Vec<PathBuf> = paths.iter().map(|p| manifest.join(p.value())).collect();
    for p in &proto_paths {
        if !p.exists() {
            return Err(syn::Error::new(
                Span::call_site(),
                format!("proto file not found: {}", p.display()),
            ));
        }
    }

    let includes: Vec<PathBuf> = proto_paths
        .iter()
        .filter_map(|p| p.parent().map(|d| d.to_path_buf()))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let set = protox::compile(&proto_paths, &includes)
        .map_err(|e| syn::Error::new(Span::call_site(), format!("protox failed: {}", e)))?;

    let generated = codegen::generate(&set)
        .map_err(|e| syn::Error::new(Span::call_site(), format!("codegen failed: {}", e)))?;

    let mut item_mod: ItemMod = syn::parse(item)?;
    if let Some((_, items)) = item_mod.content.as_mut() {
        items.extend(generated);
    }

    Ok(quote!(#item_mod))
}
