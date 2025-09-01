fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Compile protobuf files using prost_build
    use prost_build::compile_protos;
    
    compile_protos(
        &["src/protobuf/schema.proto"],
        &["src/protobuf"],
    )
    .unwrap();
} 