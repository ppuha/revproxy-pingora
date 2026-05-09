use pingora::prelude::Opt;
use pingora_core::server::Server;
use pingora_proxy::http_proxy_service;

mod proxy;
use crate::proxy::{App, Proxy};

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut server = Server::new(Opt::parse_args()).unwrap();
    server.bootstrap();

    let mut app_proxy = Proxy::new();
    let app = App::new("foo".to_string(), "127.0.0.1:6002".to_string());
    app_proxy.add_app(&app);

    let mut proxy = http_proxy_service(&server.configuration, app_proxy);
    proxy.add_tcp("127.0.0.1:6006");
    server.add_service(proxy);

    log::info!("running on port 6006");
    server.run_forever();
}
