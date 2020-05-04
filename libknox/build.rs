fn main() {
    protoc_rust::Codegen::new()
        .out_dir("src/")
        .inputs(&["protobufs/pb.proto"])
        .includes(&["protobufs"])
        .run()
        .expect("protoc");
}
