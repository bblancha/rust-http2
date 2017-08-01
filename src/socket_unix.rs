use std::io;
use std::any::Any;
use std::fs;
use std::path::Path;

use tokio_core::reactor;
use tokio_uds::UnixListener;
use tokio_uds::UnixStream;

use futures::stream::Stream;

use socket::ToSocketListener;
use socket::ToTokioListener;
use socket::ToStream;
use socket::StreamItem;

use server_conf::ServerConf;


impl ToSocketListener for String {
    fn to_listener(&self, _conf: &ServerConf) -> Box<ToTokioListener + Send> {
        if Path::new(self.as_str()).exists() {
            fs::remove_file(self.as_str()).expect("remove socket before binding");
        }
        Box::new(::std::os::unix::net::UnixListener::bind(self).unwrap())
    }
}

impl ToTokioListener for ::std::os::unix::net::UnixListener {
    fn to_tokio_listener(self: Box<Self>, handle: &reactor::Handle) -> Box<ToStream> {
        Box::new(UnixListener::from_listener(*self, handle).unwrap())
    }

    fn local_addr(&self) -> io::Result<Box<Any>> {
        let addr = self.local_addr().unwrap();
        let path = addr.as_pathname().unwrap();
        let string = path.to_str().unwrap().to_owned();

        Ok(Box::new(string))
    }
}

impl ToStream for UnixListener {
    fn incoming(self: Box<Self>)
        -> Box<Stream<Item=(Box<StreamItem>, Box<Any>), Error=io::Error>>
    {
        let stream = (*self).incoming().map(|(stream, addr)|
            (Box::new(stream) as Box<StreamItem>, Box::new(addr) as Box<Any>)
        );
        Box::new(stream)
    }
}

impl StreamItem for UnixStream {
    fn is_tcp(&self) -> bool {
        false
    }

    fn set_nodelay(&self, _no_delay: bool) -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::Other, "Cannot set nodelay on unix domain socket"))
    }
}
