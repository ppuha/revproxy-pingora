use pingora::prelude::Opt;
use pingora_core::server::Server;
use pingora_proxy::http_proxy_service;

mod proxy;
use crate::proxy::SimpleProxy;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut server = Server::new(Opt::parse_args()).unwrap();
    server.bootstrap();

    let mut upstreams = SimpleProxy::new();
    upstreams.add_upstream("/foo".to_string(), "127.0.0.1:6002");

    let mut proxy = http_proxy_service(&server.configuration, upstreams);
    proxy.add_tcp("127.0.0.1:6006");
    server.add_service(proxy);

    log::info!("running on port 6006");
    server.run_forever();
}
