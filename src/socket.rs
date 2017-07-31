use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::io;

use std::any::Any;

use tokio_core::reactor;
use tokio_core::net::TcpListener;

use server_conf::ServerConf;

use net2;

pub trait ToSocketListener {
    fn to_listener(&self, conf: &ServerConf) -> Box<ToTokioListener + Send>;
}

impl ToSocketListener for SocketAddr {
    fn to_listener(&self, conf: &ServerConf) -> Box<ToTokioListener + Send> {
        let listen_addr = self.to_socket_addrs().unwrap().next().unwrap();
        Box::new(listener(&listen_addr, conf).unwrap())
    }
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

fn listener(
    addr: &SocketAddr,
    conf: &ServerConf)
        -> io::Result<::std::net::TcpListener>
{
    let listener = match *addr {
        SocketAddr::V4(_) => net2::TcpBuilder::new_v4()?,
        SocketAddr::V6(_) => net2::TcpBuilder::new_v6()?,
    };
    configure_tcp(&listener, conf)?;
    listener.reuse_address(true)?;
    listener.bind(addr)?;
    let backlog = conf.backlog.unwrap_or(1024);
    listener.listen(backlog)
}

pub trait ToTokioListener {
    fn to_tokio_listener(self: Box<Self>, handle: &reactor::Handle) -> TcpListener;

    fn local_addr(&self) -> io::Result<Box<Any>>;
}

impl ToTokioListener for ::std::net::TcpListener {
    fn to_tokio_listener(self: Box<Self>, handle: &reactor::Handle) -> TcpListener {
        let local_addr = self.local_addr().unwrap();
        TcpListener::from_listener(*self, &local_addr, handle).unwrap()
    }

    fn local_addr(&self) -> io::Result<Box<Any>> {
        Ok(Box::new(self.local_addr().unwrap()))
    }
}
