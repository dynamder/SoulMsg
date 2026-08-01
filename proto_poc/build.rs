use std::env;
use std::path::PathBuf;

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc");
    // SAFETY: build scripts are single-threaded; setting PROTOC is fine here.
    unsafe { std::env::set_var("PROTOC", protoc) };

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let descriptor_path = out_dir.join("descriptors.pb");

    let mut config = prost_build::Config::new();
    config.file_descriptor_set_path(&descriptor_path);

    config
        .compile_protos(&["proto/chat.proto", "proto/chat_old.proto"], &["proto"])
        .expect("failed to compile protos");

    println!("cargo:rerun-if-changed=proto/chat.proto");
    println!("cargo:rerun-if-changed=proto/chat_old.proto");
}
