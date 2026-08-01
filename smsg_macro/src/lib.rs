mod codegen;
mod error;
mod parser;

use smsg_core::{hash, ir};

use codegen::struct_gen::{ModuleGenerator, StructGenerator};
use codegen::validate::{validate_module_structure, validate_smsg_file};
use codegen::{derive_gen::DeriveGenerator, CodeGenerator};
use parser::github::{
    fetch_directory_contents_recursive, fetch_file_content_at_ref, parse_github_url,
    parse_path_with_ref,
};
use parser::package_parser::{build_module_structure, parse_package_toml, walk_package_directory};
use parser::parse_smsg;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse_macro_input, ItemMod};

#[derive(Debug, Clone)]
enum SmsgCategory {
    File,
    Package,
    Git,
}

#[derive(Debug)]
struct SmsgAttribute {
    category: SmsgCategory,
    path: String,
    url: Option<String>,
}

impl SmsgAttribute {
    pub fn parse(attr: &str) -> Result<Self, String> {
        let attr = attr.trim();

        if attr.starts_with('"') {
            return Ok(SmsgAttribute {
                category: SmsgCategory::File,
                path: attr.trim_matches('"').to_string(),
                url: None,
            });
        }

        let parts: Vec<&str> = attr.split(',').collect();
        let mut category = SmsgCategory::File;
        let mut path = String::new();
        let mut url: Option<String> = None;

        for part in parts {
            let part = part.trim();
            if part.starts_with("category") {
                let value = part.split('=').nth(1).map(|s| s.trim()).unwrap_or("");
                category = match value {
                    "package" => SmsgCategory::Package,
                    "file" => SmsgCategory::File,
                    "git" => SmsgCategory::Git,
                    _ => {
                        return Err(format!(
                            "Invalid category: {}. Expected 'file', 'package', or 'git'",
                            value
                        ));
                    }
                };
            } else if part.starts_with("path") {
                path = part
                    .split('=')
                    .nth(1)
                    .map(|s| s.trim().trim_matches('"'))
                    .unwrap_or("")
                    .to_string();
            } else if part.starts_with("url") {
                url = Some(
                    part.split('=')
                        .nth(1)
                        .map(|s| s.trim().trim_matches('"'))
                        .unwrap_or("")
                        .to_string(),
                );
            }
        }

        if path.is_empty() {
            return Err("path is required".to_string());
        }

        if matches!(category, SmsgCategory::Git) && url.is_none() {
            return Err("url is required when category is 'git'".to_string());
        }

        Ok(SmsgAttribute {
            category,
            path,
            url,
        })
    }
}

#[proc_macro_attribute]
pub fn smsg(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_str = attr.to_string();

    let smsg_attr = match SmsgAttribute::parse(&attr_str) {
        Ok(a) => a,
        Err(e) => {
            return TokenStream::from(quote! {
                compile_error!(#e)
            });
        }
    };

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let full_path = std::path::Path::new(&manifest_dir).join(&smsg_attr.path);

    match smsg_attr.category {
        SmsgCategory::File => generate_file_type(&full_path, item),
        SmsgCategory::Package => generate_package_type(&full_path, item),
        SmsgCategory::Git => generate_git_type(&smsg_attr, item),
    }
}

fn generate_file_type(full_path: &std::path::Path, item: TokenStream) -> TokenStream {
    let source_code = match std::fs::read_to_string(full_path) {
        Ok(content) => content,
        Err(e) => {
            let err_msg = format!("Failed to read smsg file '{}': {}", full_path.display(), e);
            return TokenStream::from(quote! {
                compile_error!(#err_msg)
            });
        }
    };

    let smsg_file = match parse_smsg(&source_code) {
        Ok(file) => file,
        Err(e) => {
            let err_msg = e.to_string();
            return TokenStream::from(quote! {
                compile_error!(#err_msg)
            });
        }
    };

    if let Err(err_msg) = validate_smsg_file(&smsg_file) {
        return TokenStream::from(quote! {
            compile_error!(#err_msg)
        });
    }

    let item_mod = parse_macro_input!(item as ItemMod);
    let mod_name = Ident::new(&item_mod.ident.to_string(), proc_macro2::Span::call_site());

    let struct_gen = StructGenerator::new();
    let struct_code = struct_gen.generate(&smsg_file);

    let derive_gen = DeriveGenerator::new();
    let derive_code = derive_gen.generate(&smsg_file);

    let expanded = quote! {
        pub mod #mod_name {
            use super::*;

            #struct_code
            #derive_code
        }
    };

    TokenStream::from(expanded)
}

fn generate_package_type(full_path: &std::path::Path, item: TokenStream) -> TokenStream {
    let package_toml_path = full_path.join("package.toml");

    let toml_content = match std::fs::read_to_string(&package_toml_path) {
        Ok(content) => content,
        Err(e) => {
            let err_msg = format!(
                "Failed to read package.toml '{}': {}",
                package_toml_path.display(),
                e
            );
            return TokenStream::from(quote! {
                compile_error!(#err_msg)
            });
        }
    };

    let _package = match parse_package_toml(&toml_content, &full_path.to_string_lossy()) {
        Ok(pkg) => pkg,
        Err(e) => {
            let err_msg = e.to_string();
            return TokenStream::from(quote! {
                compile_error!(#err_msg)
            });
        }
    };

    let smsg_files = match walk_package_directory(full_path) {
        Ok(files) => files,
        Err(e) => {
            let err_msg = format!(
                "Failed to read package directory '{}': {}",
                full_path.display(),
                e
            );
            return TokenStream::from(quote! {
                compile_error!(#err_msg)
            });
        }
    };

    let module_structure = build_module_structure(full_path, &smsg_files);

    if let Err(err_msg) = validate_module_structure(&module_structure) {
        return TokenStream::from(quote! {
            compile_error!(#err_msg)
        });
    }

    let item_mod = parse_macro_input!(item as ItemMod);
    let mod_name = Ident::new(&item_mod.ident.to_string(), proc_macro2::Span::call_site());

    let module_gen = ModuleGenerator::new();
    let module_code = module_gen.generate_module_structure(&module_structure);

    let expanded = quote! {
        pub mod #mod_name {
            use super::*;

            #module_code
        }
    };

    TokenStream::from(expanded)
}

fn generate_git_type(attr: &SmsgAttribute, item: TokenStream) -> TokenStream {
    let url = match &attr.url {
        Some(u) => u.clone(),
        None => {
            return TokenStream::from(quote! {
                compile_error!("url is required when category is 'git'")
            });
        }
    };

    let repo_info = match parse_github_url(&url) {
        Ok(info) => info,
        Err(e) => {
            let err_msg = e.to_string();
            return TokenStream::from(quote! {
                compile_error!(#err_msg)
            });
        }
    };

    let (reference, clean_path) = parse_path_with_ref(&attr.path);

    let package_toml_path = if clean_path.ends_with('/') {
        format!("{}package.toml", clean_path)
    } else {
        format!("{}/package.toml", clean_path)
    };

    let files = fetch_directory_contents_recursive(
        &repo_info.owner,
        &repo_info.repo,
        clean_path,
        reference.as_deref(),
    );

    match files {
        Ok(files) => {
            let has_package_toml = files.contains_key(&package_toml_path);
            if has_package_toml {
                generate_git_package_type(&files, clean_path, item)
            } else {
                let source_code = match fetch_file_content_at_ref(
                    &repo_info.owner,
                    &repo_info.repo,
                    clean_path,
                    reference.as_deref(),
                ) {
                    Ok(content) => content,
                    Err(e) => {
                        let err_msg = e.to_string();
                        return TokenStream::from(quote! {
                            compile_error!(#err_msg)
                        });
                    }
                };

                let smsg_file = match parse_smsg(&source_code) {
                    Ok(file) => file,
                    Err(e) => {
                        let err_msg = e.to_string();
                        return TokenStream::from(quote! {
                            compile_error!(#err_msg)
                        });
                    }
                };

                if let Err(err_msg) = validate_smsg_file(&smsg_file) {
                    return TokenStream::from(quote! {
                        compile_error!(#err_msg)
                    });
                }

                let item_mod = parse_macro_input!(item as ItemMod);
                let mod_name =
                    Ident::new(&item_mod.ident.to_string(), proc_macro2::Span::call_site());

                let struct_gen = StructGenerator::new();
                let struct_code = struct_gen.generate(&smsg_file);

                let derive_gen = DeriveGenerator::new();
                let derive_code = derive_gen.generate(&smsg_file);

                let expanded = quote! {
                    pub mod #mod_name {
                        use super::*;

                        #struct_code
                        #derive_code
                    }
                };

                TokenStream::from(expanded)
            }
        }
        Err(_) => {
            let source_code = match fetch_file_content_at_ref(
                &repo_info.owner,
                &repo_info.repo,
                clean_path,
                reference.as_deref(),
            ) {
                Ok(content) => content,
                Err(e) => {
                    let err_msg = e.to_string();
                    return TokenStream::from(quote! {
                        compile_error!(#err_msg)
                    });
                }
            };

            let smsg_file = match parse_smsg(&source_code) {
                Ok(file) => file,
                Err(e) => {
                    let err_msg = e.to_string();
                    return TokenStream::from(quote! {
                        compile_error!(#err_msg)
                    });
                }
            };

            if let Err(err_msg) = validate_smsg_file(&smsg_file) {
                return TokenStream::from(quote! {
                    compile_error!(#err_msg)
                });
            }

            let item_mod = parse_macro_input!(item as ItemMod);
            let mod_name = Ident::new(&item_mod.ident.to_string(), proc_macro2::Span::call_site());

            let struct_gen = StructGenerator::new();
            let struct_code = struct_gen.generate(&smsg_file);

            let derive_gen = DeriveGenerator::new();
            let derive_code = derive_gen.generate(&smsg_file);

            let expanded = quote! {
                pub mod #mod_name {
                    use super::*;

                    #struct_code
                    #derive_code
                }
            };

            TokenStream::from(expanded)
        }
    }
}

fn generate_git_package_type(
    files: &std::collections::HashMap<String, String>,
    base_path: &str,
    item: TokenStream,
) -> TokenStream {
    let package_toml_path = if base_path.ends_with('/') {
        format!("{}package.toml", base_path)
    } else {
        format!("{}/package.toml", base_path)
    };

    let toml_content = match files.get(&package_toml_path) {
        Some(content) => content.clone(),
        None => {
            let err_msg = format!("package.toml not found at {}", package_toml_path);
            return TokenStream::from(quote! {
                compile_error!(#err_msg)
            });
        }
    };

    let _package = match parse_package_toml(&toml_content, base_path) {
        Ok(pkg) => pkg,
        Err(e) => {
            let err_msg = e.to_string();
            return TokenStream::from(quote! {
                compile_error!(#err_msg)
            });
        }
    };

    let item_mod = parse_macro_input!(item as ItemMod);
    let mod_name = Ident::new(&item_mod.ident.to_string(), proc_macro2::Span::call_site());

    let module_structure = build_git_module_structure(files, base_path);

    if let Err(err_msg) = validate_module_structure(&module_structure) {
        return TokenStream::from(quote! {
            compile_error!(#err_msg)
        });
    }

    let module_gen = ModuleGenerator::new();
    let module_code = module_gen.generate_module_structure(&module_structure);

    let expanded = quote! {
        pub mod #mod_name {
            use super::*;

            #module_code
        }
    };

    TokenStream::from(expanded)
}

fn build_git_module_structure(
    files: &std::collections::HashMap<String, String>,
    base_path: &str,
) -> crate::ir::ModuleStructure {
    use crate::ir::Module;

    let root_name = base_path
        .split('/')
        .filter(|s| !s.is_empty())
        .next_back()
        .unwrap_or("root")
        .to_string();

    let mut root_module = Module::new(root_name.clone(), base_path.to_string());

    for (path, content) in files {
        if path.ends_with("package.toml") {
            continue;
        }

        if !path.ends_with(".smsg") {
            continue;
        }

        let relative = path.strip_prefix(base_path).unwrap_or(path);
        let relative = relative.trim_start_matches('/');
        let parent = std::path::Path::new(relative).parent();

        if let Ok(smsg_file_parsed) = parse_smsg(content) {
            match parent {
                Some(parent_dir) if parent_dir.as_os_str().is_empty() => {
                    root_module.messages.extend(smsg_file_parsed.messages);
                }
                Some(parent_dir) => {
                    let parts: Vec<&str> = parent_dir.iter().filter_map(|p| p.to_str()).collect();
                    add_git_to_nested_module(&mut root_module, &parts, &smsg_file_parsed.messages);
                }
                None => {
                    root_module.messages.extend(smsg_file_parsed.messages);
                }
            }
        }
    }

    crate::ir::ModuleStructure { root_module }
}

fn add_git_to_nested_module(
    parent: &mut crate::ir::Module,
    path_parts: &[&str],
    messages: &[crate::ir::MessageDef],
) {
    if path_parts.is_empty() {
        parent.messages.extend(messages.to_vec());
        return;
    }

    let (first, rest) = path_parts.split_first().unwrap();

    if let Some(child) = parent.children.iter_mut().find(|m| m.name == *first) {
        add_git_to_nested_module(child, rest, messages);
    } else {
        let child_path = format!("{}/{}", parent.path, first);
        let mut new_module = crate::ir::Module::new(first.to_string(), child_path);
        add_git_to_nested_module(&mut new_module, rest, messages);
        parent.children.push(new_module);
    }
}
