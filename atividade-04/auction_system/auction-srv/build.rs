fn main() {
    tonic_build::configure()
        .build_server(true) // generate server code
        .build_client(true) // generate client code
        .compile(
            &["../../proto-definitions/auction-srv/auction-srv.proto"], // path to your proto
            &["../../proto-definitions/auction-srv/"],               // include path
        )
        .unwrap();
}
