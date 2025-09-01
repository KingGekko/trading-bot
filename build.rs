fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Compile protobuf files
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(
            &["src/protobuf/schema.proto"],
            &["src/protobuf"],
        )
        .unwrap();
} 