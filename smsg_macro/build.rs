use std::env;
use std::path::PathBuf;

fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc");
    // SAFETY: build scripts are single-threaded; setting PROTOC is fine here.
    unsafe { env::set_var("PROTOC", protoc) };

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut config = prost_build::Config::new();
    config.file_descriptor_set_path(out_dir.join("descriptors.pb"));

    // Compile the shared fixture protos for the macro integration test.
    config
        .compile_protos(
            &[
                "../proto_poc/proto/chat.proto",
                "../proto_poc/proto/chat_old.proto",
            ],
            &["../proto_poc/proto"],
        )
        .expect("failed to compile fixture protos");

    println!("cargo:rerun-if-changed=../proto_poc/proto/chat.proto");
    println!("cargo:rerun-if-changed=../proto_poc/proto/chat_old.proto");
}
