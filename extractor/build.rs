fn main() {
    println!("build.rs running");
    // to regenerate the code, run `cargo build`
    println!("cargo:rerun-if-changed=hello.proto");
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(&["proto/extractor.proto"], &["proto/"])
        .unwrap();
}
