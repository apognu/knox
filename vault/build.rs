use protoc_rust::Customize;

fn main() {
	protoc_rust::run(protoc_rust::Args {
	    out_dir: "src/",
	    input: &["protobufs/pb.proto"],
	    includes: &["protobufs"],
	    customize: Customize {
	      ..Default::default()
	    },
	}).expect("protoc");
}
