use crate::codegen::CodeGenerator;
use crate::hash::{compute_message_hash, compute_name_hash};
use crate::ir::{MessageDef, SmsgFile};
use proc_macro2::Ident;
use quote::quote;

pub struct DeriveGenerator;

impl DeriveGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DeriveGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator for DeriveGenerator {
    fn generate(&self, smsg_file: &SmsgFile) -> proc_macro2::TokenStream {
        let message_impls: Vec<proc_macro2::TokenStream> = smsg_file
            .messages
            .iter()
            .map(generate_message_meta)
            .collect();

        quote! {
            #(#message_impls)*
        }
    }
}

fn generate_message_meta(message: &MessageDef) -> proc_macro2::TokenStream {
    let struct_name = Ident::new(&message.name, proc_macro2::Span::call_site());
    let version_hash = compute_message_hash(message);
    let name_hash = compute_name_hash(&message.name);
    let message_name = message.name.as_str();

    quote! {
        impl ::soul_msg::MessageMeta for #struct_name {
            fn version_hash() -> [u8; 32] {
                [#(#version_hash),*]
            }

            fn name_hash() -> [u8; 32] {
                [#(#name_hash),*]
            }

            fn message_name() -> &'static str {
                #message_name
            }
        }
    }
}
