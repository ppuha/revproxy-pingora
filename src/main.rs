use async_trait::async_trait;
use pingora::prelude::{HttpPeer, Opt, RequestHeader};
use pingora::{ConnectNoRoute, Error};
use pingora_core::Result;
use pingora_core::server::Server;
use pingora_load_balancing::Backend;
use pingora_proxy::{ProxyHttp, Session, http_proxy_service};
use std::collections::HashMap;

pub struct SimpleProxy {
    upstreams: HashMap<String, Backend>,
}

pub struct Upstream {
    path: String,
}

impl SimpleProxy {
    fn new() -> Self {
        Self {
            upstreams: HashMap::new(),
        }
    }
    fn add_upstream(&mut self, path: String, addr: &str) {
        let backend = Backend::new(addr).unwrap();
        self.upstreams.insert(path, backend);
    }
}

#[async_trait]
impl ProxyHttp for SimpleProxy {
    type CTX = Upstream;
    async fn upstream_peer(
        &self,
        session: &mut Session,
        context: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let path = session.req_header().uri.path();
        let mut parts: Vec<&str> = path.split("/").collect();
        let base = parts.remove(1);
        let mut upstream_path = parts.join("/");
        if upstream_path == "" {
            upstream_path = "/".to_string();
        }
        log::info!("base is: {}, upstream is: {}", base, upstream_path);
        context.path = upstream_path;
        let key = format!("/{}", base);

        self.upstreams
            .get(key.as_str())
            .map_or(Err(Error::new(ConnectNoRoute)), |us| {
                Ok(Box::new(HttpPeer::new(
                    us,
                    false,
                    "one.one.one.one".to_string(),
                )))
            })
    }
    fn new_ctx(&self) -> Self::CTX {
        return Upstream {
            path: "/".to_string(),
        };
    }
    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        let uri = ctx.path.parse().unwrap();
        upstream_request.set_uri(uri);
        Ok(())
    }
}

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
