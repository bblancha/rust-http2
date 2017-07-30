use std::net::SocketAddr;
use std::io;

use futures::stream::Stream;

use tokio_core::reactor;
use tokio_core::net::TcpStream;
use tokio_core::net::TcpListener;

use net2;

use server_conf::ServerConf;

pub trait ToSocketListener {
    fn to_listener(&self, conf: &ServerConf) -> Box<ToTokioListener + Send>;
}

impl ToSocketListener for SocketAddr {
    fn to_listener(&self, conf: &ServerConf) -> Box<ToTokioListener + Send> {
        Box::new(listener(&self, conf).unwrap())
    }
}

fn listener(
    addr: &SocketAddr,
    conf: &ServerConf)
        -> io::Result<::std::net::TcpListener>
{
    println!("about to bind address {:?}", addr);
    let listener = match *addr {
        SocketAddr::V4(_) => net2::TcpBuilder::new_v4()?,
        SocketAddr::V6(_) => net2::TcpBuilder::new_v6()?,
    };
    configure_tcp(&listener, conf)?;
    listener.reuse_address(true)?;
    listener.bind(addr)?;
    let backlog = conf.backlog.unwrap_or(1024);
    println!("address bound");
    listener.listen(backlog)
}

#[cfg(unix)]
fn configure_tcp(tcp: &net2::TcpBuilder, conf: &ServerConf) -> io::Result<()> {
    use net2::unix::UnixTcpBuilderExt;
    if let Some(reuse_port) = conf.reuse_port {
        tcp.reuse_port(reuse_port)?;
    }
    Ok(())
}

#[cfg(windows)]
fn configure_tcp(_tcp: &net2::TcpBuilder, conf: &ServerConf) -> io::Result<()> {
    Ok(())
}

pub trait ToTokioListener {
    fn to_tokio_listener(self: Box<Self>, handle: &reactor::Handle) -> Box<ToStream>;
}

impl ToTokioListener for ::std::net::TcpListener {
    fn to_tokio_listener(self: Box<Self>, handle: &reactor::Handle) -> Box<ToStream> {
        let local_addr = self.local_addr().unwrap();
        let tokio_listener = TcpListener::from_listener(*self, &local_addr, handle).unwrap();
        Box::new(tokio_listener)
    }
}

pub trait ToStream {
    fn incoming(self: Box<Self>) -> Box<Stream<Item=(TcpStream, SocketAddr), Error=io::Error>>;
}

impl ToStream for TcpListener {
    fn incoming(self: Box<Self>) -> Box<Stream<Item=(TcpStream, SocketAddr), Error=io::Error>> {
        Box::new((*self).incoming())
    }
}
