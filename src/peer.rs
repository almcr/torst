use crate::bitfield::Bitfield;
use crate::protocol::{Message, Handshake};
use crate::torrent::{Torrent, CLIENT_PEER_ID};

use std::net::SocketAddr;
use tokio::prelude::*;
use tokio::time::{timeout, Duration};

use std::io;
use tokio::net::TcpStream;

pub struct PeerAddr(SocketAddr);

#[derive(Default, Debug)]
pub struct PeerStatus {
  chocked: bool,
  interested: bool,
}

pub struct Peer {
  // available remote peer pieces
  bitfield: Bitfield,
  remote_stats: PeerStatus,
  local_status: PeerStatus,
  conn: TcpStream,
}

impl PeerAddr {
  pub(crate) fn new(bytes: &[u8]) -> Self {
    SocketAddr::new(
      [bytes[0], bytes[1], bytes[2], bytes[3]],
      u16::from_be_bytes([bytes[4], bytes[5]]),
    )
  }
}

impl Peer {
  /// connect to new peer and complete the handshake and receives the bitfield
  pub async fn new(addr: SocketAddr, t: &Torrent) -> io::Result<Self> {
    // connect or timeout
    let stream = timeout(Duration::from_secs(10), TcpStream::connect(addr)).await??;

    // send handshake
    let mut buf = bytes::BytesMut::new();
    Message::handshake(&t.info.info_hash, CLIENT_PEER_ID).encode(&mut buf);
    stream.write_buf(&mut buf).await?;
    // receive handshake
    stream.read_buf(&mut buf).await?;

    // verify recieved handshake
    if buf[0] == 0 {
      Err(io::Error::new(io::ErrorKind::InvalidData, "invalid message length"));
    }

    if &buf[1..20] != b"BitTorrent protocol" {
      Err(io::Error::new(io::ErrorKind::InvalidData, "non BitTorrent handshake"));
    }

    if &t.info.info_hash != &buf[2..27] {
      Err(io::Error::new(io::ErrorKind::InvalidData, "invalid info hash"));
    }
    
    // receive bitfield
    // todo
    stream.read_buf(&mut buf).await?;

    


    Ok(Self { bitfield: bf, conn: stream, Default::default().. })
  }
}
