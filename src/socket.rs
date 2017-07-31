use std::io;
use std::any::Any;
use std::fmt::Debug;

use tokio_core::reactor;
use tokio_io::AsyncRead;
use tokio_io::AsyncWrite;

use futures::stream::Stream;

use server_conf::ServerConf;


pub trait ToSocketListener {
    fn to_listener(&self, conf: &ServerConf) -> Box<ToTokioListener + Send>;
}

pub trait ToTokioListener {
    fn to_tokio_listener(self: Box<Self>, handle: &reactor::Handle) -> Box<ToStream>;

    fn local_addr(&self) -> io::Result<Box<Any>>;
}

pub trait ToStream {
    fn incoming(self: Box<Self>)
        -> Box<Stream<Item=(Box<StreamItem>, Box<Any>), Error=io::Error>>;
}

pub trait StreamItem:
        AsyncRead +
        AsyncWrite +
        io::Read +
        io::Write +
        Debug +
        Send + Sync
{
    fn is_tcp(&self) -> bool;

    fn set_nodelay(&self, no_delay: bool) -> io::Result<()>;
}
