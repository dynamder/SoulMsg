pub mod derive_gen;
pub mod struct_gen;
pub mod validate;

use crate::ir::SmsgFile;
use proc_macro2::TokenStream;

pub trait CodeGenerator {
    fn generate(&self, smsg_file: &SmsgFile) -> TokenStream;
}
