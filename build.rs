fn main() {
    
    tonic_build::configure()
        .type_attribute("raftservice.UUID", "#[derive(Hash, Eq)]")
        .compile(&["proto/raftservice.proto"], &["proto"])
        .unwrap();
    
    tonic_build::compile_protos("proto/helloworld.proto").unwrap();
}