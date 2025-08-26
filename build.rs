fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if protoc is available
    if std::env::var("PROTOC").is_err() {
        // Try to find protoc in PATH
        if std::process::Command::new("protoc")
            .arg("--version")
            .output()
            .is_err()
        {
            eprintln!("❌ Error: protoc (Protocol Buffers compiler) not found!");
            eprintln!("");
            eprintln!("🔧 To fix this issue, run:");
            eprintln!("   ./fix_protobuf.sh");
            eprintln!("");
            eprintln!("📦 Or install manually:");
            eprintln!("   Ubuntu/Debian: sudo apt-get install protobuf-compiler");
            eprintln!("   CentOS/RHEL:  sudo yum install protobuf-compiler");
            eprintln!("   macOS:        brew install protobuf");
            eprintln!("");
            eprintln!("📥 Or download from: https://github.com/protocolbuffers/protobuf/releases");
            eprintln!("");
            eprintln!("📚 For more help: https://docs.rs/prost-build/#sourcing-protoc");
            std::process::exit(1);
        }
    }

    // Compile protobuf files
    tonic_build::compile_protos("proto/receipt.proto")?;
    Ok(())
} 