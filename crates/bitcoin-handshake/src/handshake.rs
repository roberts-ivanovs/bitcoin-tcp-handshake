use std::{marker::PhantomData, net::SocketAddr};

use tokio::{io::{DuplexStream, AsyncWriteExt, AsyncReadExt, }, net::{TcpStream, tcp::{OwnedReadHalf, OwnedWriteHalf, ReadHalf, WriteHalf}}};
use tracing::instrument;
use tracing::field::valuable;

pub struct Connected;
pub struct VersionSent;
pub struct Open;

pub struct ConnectionHandle<T> {
    stream: TcpStream,
    _type: PhantomData<T>,
}

impl<T> ConnectionHandle<T> {

    pub(crate) async fn write_and_flush(&mut self, buf: &[u8]) {
        self.stream.write_all(buf).await.unwrap();
        self.stream.flush().await.unwrap();
    }

    pub(crate) fn split<'a>(&'a mut self) -> (ReadHalf, WriteHalf) {
        self.stream.split()
    }
}

impl ConnectionHandle<Connected> {

    #[instrument()]
    pub async fn new(addr: SocketAddr) -> Self {
        tracing::info!("Init new connection");
        let stream = TcpStream::connect(addr).await.unwrap();
        tracing::info!("Connected to peer");

        stream.writable().await.unwrap();
        tracing::info!("TCP socket is writable");

        stream.readable().await.unwrap();
        tracing::info!("TCP socket is readable");

        Self {
            stream,
            _type: PhantomData,
        }
    }

    pub async fn process_version(mut self) -> ConnectionHandle<VersionSent> {
        let mut msg = b"f9beb4d976657273696f6e000000000064000000f4de76b762ea00000100000000000000c8c6ae5d00000000010000000000000000000000000000000000ffff68c7b80f208d010000000000000000000000000000000000ffff7f000001208d0f2f736a397699b60f2f5361746f7368693a302e372e322f00000000";
        self.write_and_flush(msg).await;
        tracing::info!("Version message sent");

        let mut buf = vec![];
        self.stream.read_to_end(&mut buf).await.unwrap();
        tracing::info!("Version message received");

        ConnectionHandle {
            stream: self.stream,
            _type: PhantomData,
        }
    }
}

impl ConnectionHandle<VersionSent> {
    pub async fn process_verack(mut self) -> ConnectionHandle<Open> {
        let mut msg = b"f9beb4d976657261636b000000000000000000005df6e0e2";
        self.write_and_flush(msg).await;
        tracing::info!("Verack message sent");

        let mut buf = vec![];
        self.stream.read_to_end(&mut buf).await.unwrap();
        tracing::info!("Verack message received");

        ConnectionHandle {
            stream: self.stream,
            _type: PhantomData,
        }
    }
}
