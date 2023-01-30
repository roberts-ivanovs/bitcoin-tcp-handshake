use std::net::SocketAddr;

use tokio::{net::TcpStream, io::{ReadHalf, BufReader}};


pub struct DuplexTcpStream {
    read: TcpStreamReader,
    write: TcpStreamWriter,
}

impl DuplexTcpStream {
    pub async fn new(addr: SocketAddr) -> Self {
        let stream = TcpStream::connect(&addr).await.unwrap();
        // Wait for the stream to be ready
        stream.readable().await.unwrap();
        stream.writable().await.unwrap();

        let (reader, writer) = tokio::io::split(stream);
        // tokio::io::DuplexStream::from(stream);
        let reader = TcpStreamReader::new(reader);
        let writer = TcpStreamWriter::new(writer);

        Self {
            read: reader,
            write: writer,
        }
    }
}

struct TcpStreamReader {
    inner: BufReader<ReadHalf<TcpStream>>,
}

impl TcpStreamReader {
    fn new(inner: ReadHalf<TcpStream>) -> Self {
        Self {
            inner: BufReader::new(inner),
        }
    }
}

struct TcpStreamWriter {
    inner: tokio::io::WriteHalf<TcpStream>,
}

impl TcpStreamWriter {
    fn new(inner: tokio::io::WriteHalf<TcpStream>) -> Self {
        Self { inner }
    }
}
