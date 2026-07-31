use crate::codegen::CodeGenerator;
use crate::ir::{FieldType, MessageDef, Module, ModuleStructure, SmsgFile};
use proc_macro2::Ident;
use quote::quote;

pub struct StructGenerator;

impl StructGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StructGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator for StructGenerator {
    fn generate(&self, smsg_file: &SmsgFile) -> proc_macro2::TokenStream {
        let message_impls: Vec<proc_macro2::TokenStream> =
            smsg_file.messages.iter().map(generate_struct).collect();

        quote! {
            #(#message_impls)*
        }
    }
}

fn generate_struct(message: &MessageDef) -> proc_macro2::TokenStream {
    let struct_name = Ident::new(&message.name, proc_macro2::Span::call_site());
    let field_idents: Vec<proc_macro2::TokenStream> = message
        .fields
        .iter()
        .map(|f| {
            let name = Ident::new(&f.name, proc_macro2::Span::call_site());
            let ty = convert_field_type_to_rust(&f.field_type);
            quote! { pub #name: #ty }
        })
        .collect();

    let field_names_only: Vec<proc_macro2::TokenStream> = message
        .fields
        .iter()
        .map(|f| {
            let name = Ident::new(&f.name, proc_macro2::Span::call_site());
            quote! { #name }
        })
        .collect();

    let serialize_body: Vec<proc_macro2::TokenStream> = message
        .fields
        .iter()
        .map(|f| {
            let name = Ident::new(&f.name, proc_macro2::Span::call_site());
            quote! { self.#name.serialize(serializer); }
        })
        .collect();

    let deserialize_fields: Vec<proc_macro2::TokenStream> = message
        .fields
        .iter()
        .map(|f| {
            let name = Ident::new(&f.name, proc_macro2::Span::call_site());
            quote! { #name: zenoh_ext::Deserialize::deserialize(deserializer)? }
        })
        .collect();

    quote! {
        #[derive(Debug, Clone, PartialEq)]
        pub struct #struct_name {
            #(#field_idents),*
        }

        impl #struct_name {
            pub fn new() -> Self {
                #struct_name {
                    #(#field_names_only: Default::default()),*
                }
            }
        }

        impl Default for #struct_name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl zenoh_ext::Serialize for #struct_name {
            fn serialize(&self, serializer: &mut zenoh_ext::ZSerializer) {
                #(#serialize_body)*
            }
        }

        impl zenoh_ext::Deserialize for #struct_name {
            fn deserialize(deserializer: &mut zenoh_ext::ZDeserializer) -> Result<Self, zenoh_ext::ZDeserializeError> {
                Ok(#struct_name {
                    #(#deserialize_fields),*
                })
            }
        }
    }
}

fn convert_field_type_to_rust(field_type: &FieldType) -> proc_macro2::TokenStream {
    match field_type {
        FieldType::Primitive(p) => {
            let ty = p.rust_type();
            let ident = Ident::new(ty, proc_macro2::Span::call_site());
            quote! { #ident }
        }
        FieldType::Array(inner, size) => {
            let inner_type = convert_field_type_to_rust(inner);
            if let Some(n) = size {
                quote! { [#inner_type; #n] }
            } else {
                quote! { Vec<#inner_type> }
            }
        }
        FieldType::Nested(name) => {
            let ident = Ident::new(name, proc_macro2::Span::call_site());
            quote! { #ident }
        }
    }
}

pub struct ModuleGenerator;

impl ModuleGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_module_structure(
        &self,
        module_structure: &ModuleStructure,
    ) -> proc_macro2::TokenStream {
        let root_mod = &module_structure.root_module;

        let struct_impls: Vec<proc_macro2::TokenStream> =
            root_mod.messages.iter().map(generate_struct).collect();

        let child_mods: Vec<proc_macro2::TokenStream> =
            root_mod.children.iter().map(generate_module).collect();

        quote! {
            #(#struct_impls)*
            #(#child_mods)*
        }
    }
}

impl Default for ModuleGenerator {
    fn default() -> Self {
        Self::new()
    }
}

fn generate_module(module: &Module) -> proc_macro2::TokenStream {
    let module_name = Ident::new(&module.name, proc_macro2::Span::call_site());

    let struct_impls: Vec<proc_macro2::TokenStream> =
        module.messages.iter().map(generate_struct).collect();

    let child_mods: Vec<proc_macro2::TokenStream> =
        module.children.iter().map(generate_module).collect();

    quote! {
        pub mod #module_name {
            #(#struct_impls)*
            #(#child_mods)*
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Field;
    use crate::ir::PrimitiveType;
    #[test]
    fn test_generate_simple_struct() {
        let smsg_file = SmsgFile {
            messages: vec![MessageDef {
                name: "ChatMessage".to_string(),
                fields: vec![
                    Field {
                        name: "sender".to_string(),
                        field_type: FieldType::Primitive(PrimitiveType::String),
                        line: 1,
                        col: 1,
                    },
                    Field {
                        name: "content".to_string(),
                        field_type: FieldType::Primitive(PrimitiveType::String),
                        line: 2,
                        col: 1,
                    },
                ],
                line: 1,
                col: 1,
            }],
        };

        let generator = StructGenerator::new();
        let tokens = generator.generate(&smsg_file);
        let output = tokens.to_string();
        assert!(output.contains("ChatMessage"));
        assert!(output.contains("sender"));
        assert!(output.contains("content"));
    }
}
