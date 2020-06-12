use crate::bitfield::Bitfield;
use byteorder::{BigEndian, WriteBytesExt};
use bytes::{Bytes, BytesMut};
use std::io::{Write, Error, ErrorKind, Result};

pub enum Message {
  // empty message to keep peer connection alive
  KeepAlive,
  // notification prevent requests from/to peer due congestion and upload capacity
  Choke,
  // notification to allow requests from/to peer
  UnChoke,
  // notification from/to peer to start requesting block
  Interested,
  // notification from/to peer to stop requesting
  NotInterested,
  // notification from/to peer that a piece is successfully downloaded
  Have {
    piece_index: u32,
  },
  // message exchanged immediatly after connection initiation
  Handshake(Handshake),
  // message sent after handshake, setted bites indicate that peer have that piece index
  Bitfield(Bitfield),
  // message for requesting a block within a piece
  // from 1.0 specification, clients usually used 32KB block requests, version 4 has enforced 16KB
  // requests, though, due to unclear size request, clients can choose a size from 16 to 128Kb
  Request {
    // piece index
    index: u32,
    // byte offset of block within piece
    begin: u32,
    // block length
    length: u32,
  },
  // message containing a block payload
  Piece {
    // piece index
    index: u32,
    // byte offset of block within piece
    begin: u32,
    // payload
    block: Vec<u8>,
  },
  // message issued to cancel a Request message
  Cancel {
    // piece index
    index: u32,
    // byte offset of block within piece
    begin: u32,
    // block length
    length: u32,
  },
}

pub struct Handshake {
  bytes: Bytes,
}

impl Handshake {
  pub fn try_from(bytes: &Bytes) -> Result<Self> {
    if bytes.len() > 12 {
      Err(Error::new(ErrorKind::InvalidData, "invalid handshake size"))
    }

    Ok(Self {bytes})
  }

  fn resrv(&self) -> &[u8] {
    &self.bytes[..7]
  }

  fn info_hash(&self) -> &[u8] {
    &self.bytes[8..28]
  }

  fn peer_id(&self) -> &[u8] {
    &self.bytes[29..]
  }

  fn len(&self) -> usize {
    self.bytes.len()
  }
}

impl Message {
  pub fn handshake(info_hash: &[u8; 20], cli_id: &'static str) -> Self {
    let mut hsk = BytesMut::with_capacity(48);
    hsk.extend_from_slice([0u8; 8].as_ref()); // 8 bytes
    hsk.extend_from_slice(info_hash.as_ref()); // 20 bytes
    hsk.extend_from_slice(cli_id.as_ref()); // 20 bytes
    Message::Handshake(Handshake::from(hsk.into()))
  }

  pub fn request(index: u32, begin: u32, length: u32) -> Self {
    Message::Request { index, begin, length }
  }

  pub fn cancel(index: u32, begin: u32, length: u32) -> Self {
    Message::Cancel { index, begin, length }
  }

  pub fn piece(index: u32, begin: u32, block: Vec<u8>) -> Self {
    Message::Piece {index, begin, block}
  }

  pub fn encode(&self, mut buf: &mut [u8]) -> Result<()> {
    // all messages take the form of <length prefix><message ID><payload>
    match *self {
      Message::KeepAlive => {
        buf.write_u32::<BigEndian>(0)?;
      }
      Message::Choke => {
        // length prefix
        buf.write_u32::<BigEndian>(1)?;
        // message id
        buf.write_u8(0)?;
      }
      Message::UnChoke => {
        buf.write_u32::<BigEndian>(1)?;
        buf.write_u8(1)?;
      }
      Message::Interested => {
        buf.write_u32::<BigEndian>(1)?;
        buf.write_u8(2)?;
      }
      Message::NotInterested => {
        buf.write_u32::<BigEndian>(1)?;
        buf.write_u8(3)?;
      }
      Message::Have { piece_index } => {
        buf.write_u32::<BigEndian>(5)?;
        buf.write_u8(4)?;
        buf.write_u32::<BigEndian>(piece_index)?;
      }
      Message::Handshake(ref hsk) => {
        buf.write_u8(19)?;
        buf.write_all("BitTorrent protocol".as_ref())?;
        buf.write_all(hsk.resrv())?;
        buf.write_all(hsk.info_hash())?;
        buf.write_all(hsk.peer_id())?;
      }
      Message::Bitfield(ref bf) => {
        buf.write_u32::<BigEndian>(1 + bf.bytes())?;
        buf.write_u8(5)?;
        buf.write_all(bf.as_bytes())?;
      }
      Message::Request { index, begin, length } => {
        buf.write_u32::<BigEndian>(13)?;
        buf.write_u8(6)?;
        buf.write_u32::<BigEndian>(index)?;
        buf.write_u32::<BigEndian>(begin)?;
        buf.write_u32::<BigEndian>(length)?;
      }
      Message::Piece { index, begin, ref block } => {
        buf.write_u32::<BigEndian>(9 + block.len() as u32)?;
        buf.write_u8(7)?;
        buf.write_u32::<BigEndian>(index)?;
        buf.write_u32::<BigEndian>(begin)?;
        buf.write_all(&block)?;
      }
      Message::Cancel { index, begin, length } => {
        buf.write_u32::<BigEndian>(13)?;
        buf.write_u8(8)?;
        buf.write_u32::<BigEndian>(index)?;
        buf.write_u32::<BigEndian>(begin)?;
        buf.write_u32::<BigEndian>(length)?;
      }
    }
    Ok(())
  }
}
