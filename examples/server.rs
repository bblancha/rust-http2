extern crate httpbis;
extern crate tls_api;
extern crate tls_api_openssl;

use tls_api::TlsAcceptorBuilder as tls_api_TlsAcceptorBuilder;

use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::thread;


struct ServiceImpl {
    counter: Arc<AtomicUsize>,
}

impl ServiceImpl {
    fn new() -> ServiceImpl {
        ServiceImpl {
            counter: Arc::new(AtomicUsize::new(0))
        }
    }
}

impl httpbis::Service for ServiceImpl {
    fn start_request(&self, req_headers: httpbis::Headers, _req: httpbis::HttpPartStream)
        -> httpbis::Response
    {
        println!("starting request: {:?}", req_headers);

        if req_headers.method() == "POST" {
            self.counter.fetch_add(1, Ordering::Relaxed);
            httpbis::Response::redirect_302("/")
        } else {
            let mut resp_headers = httpbis::Headers::ok_200();
            resp_headers.add("content-type", "text/html; charset=utf-8");

            let page = format!("
<html>
    <head>
        <title>httpbis demo</title>
    </head>
    <body>
        <h3>httpbis demo</h3>
        <div>Counter: {}</div>
        <div>
            <form method='POST' action='/inc'>
                <button type='submit'>Inc</button>
            </form>
        </div>
    </body>
</html>
        ", self.counter.load(Ordering::Relaxed));

            httpbis::Response::headers_and_bytes(resp_headers, page)
        }
    }
}


fn main() {

    let pkcs12 = include_bytes!("../tests/identity.p12");
    let mut tls_acceptor = tls_api_openssl::TlsAcceptorBuilder::from_pkcs12(pkcs12, "mypass")
        .expect("acceptor builder");
    tls_acceptor.set_alpn_protocols(&[b"h2"]).expect("set_alpn_protocols");

    let mut conf = httpbis::ServerConf::new();
    conf.alpn = Some(httpbis::ServerAlpn::Require);
    let mut server = httpbis::ServerBuilder::new();
    server.set_port(8443);
    server.set_tls(tls_acceptor.build().expect("tls acceptor"));
    server.conf = conf;
    server.service.set_service("/", Arc::new(ServiceImpl::new()));
    let server = server.build().expect("server");

    println!("started server");
    println!("check it at: https://localhost:{}/", server.local_addr().port().unwrap());

    loop {
        thread::park();
    }
}
