#[derive(Debug)]
pub enum BitcoinMessage {
    Version(Version),
    Verack(Verack),
    GetAddr(GetAddr),
}

#[derive(Debug)]
pub struct Version;

#[derive(Debug)]
pub struct Verack;

#[derive(Debug)]
pub struct GetAddr;
