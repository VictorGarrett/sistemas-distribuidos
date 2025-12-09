fn main() {
    tonic_build::configure()
        .build_server(true) // generate server code
        .build_client(true) // generate client code
        .compile(
            &["../../proto-definitions/bid-srv/bid-srv.proto"], // path to your proto
            &["../../proto-definitions/bid-srv/"],               // include path
        )
        .unwrap();
}