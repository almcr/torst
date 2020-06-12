use std::collections::HashMap;

use bendy::decoding::{Error as BError, FromBencode, Object};

use crypto::{digest::Digest, sha1::Sha1};

use url::Url;

use crate::common::Result as CResult;
use crate::tracker::{Announce, Tracker, TrackerEvent, TrackerRequest};
use crate::peer::Peer;


pub const CLIENT_PEER_ID: &str = "1234zerozpeapzoekzsd";

pub struct Torrent {
  pub info: Info,
  tracker: Tracker,
  uploaded: u32,
  downloaded: u32,
  peers: HashMap<usize, Peer>
}

impl Torrent {
  fn first_announce(&self) -> Announce {
    Announce::new(
      0,
      0,
      &self.info.info_hash,
      TrackerEvent::Started,
      CLIENT_PEER_ID,
      self.info.total_length,
    )
  }

  pub async fn start(&mut self) -> CResult<()> {
    // make first tracker annoucement
    let reponse = self.tracker.send(&TrackerRequest::Announce(self.first_announce())).await?;
    
    // add peers
    for peer_addr in reponse.peers {
      self.peers.insert(0, Peer::new(peer_addr, self));
    }

    Ok(())
  }
}

#[derive(Clone)]
pub struct Info {
  // tracker request url
  pub announce: Url,
  // torrent SHA1 hash value
  pub info_hash: [u8; 20],
  // number of bytes in each piece
  pub piece_len: u32,
  // 20-byte SHA1 hash values for each piece
  pub pieces: Vec<u8>,
  // total file size in byte
  pub total_length: u32,
  // torrent file name
  pub name: String,
}

impl FromBencode for Info {
  fn decode_bencode_object(object: Object) -> Result<Self, BError> {
    let mut total_length = None;
    let mut pieces = None;
    let mut piece_len = None;
    let mut announce = None;
    let mut name = None;
    let mut info_hash = None;

    let mut torrent_dict = object.try_into_dictionary()?;
    // torrent bencode dictionary decoding
    while let Some(pair) = torrent_dict.next_pair()? {
      match pair {
        (b"info", value) => {
          let mut info_dict = value.try_into_dictionary()?;
          // decode bencode info dictionary
          while let Some(info_pair) = info_dict.next_pair()? {
            match info_pair {
              (b"piece length", v) => {
                piece_len = Some(v.try_into_integer()?.parse()?);
              }
              (b"pieces", v) => {
                pieces = Some(v.try_into_bytes()?.to_vec());
              }
              (b"name", v) => {
                name = Some(String::from_utf8(v.try_into_bytes()?.to_vec())?);
              }
              (b"length", value) => {
                total_length = Some(value.try_into_integer()?.parse()?);
              }
              (_, _) => {}
            }
          }

          // compute sha1 infohash
          let raw = info_dict.into_raw()?;
          let mut hasher = Sha1::new();
          let mut result = [0; 20];
          hasher.input(&raw);
          hasher.result(&mut result);
          info_hash = Some(result);
        }
        (b"announce", value) => {
          announce = Some(Url::parse(std::str::from_utf8(value.try_into_bytes()?)?)?);
        }
        (_, _) => {}
      }
    }

    Ok(Info {
      announce: announce.ok_or_else(|| BError::missing_field("announce"))?,
      info_hash: info_hash.ok_or_else(|| BError::missing_field("info_hash"))?,
      piece_len: piece_len.ok_or_else(|| BError::missing_field("piece_len"))?,
      pieces: pieces.ok_or_else(|| BError::missing_field("pieces"))?,
      total_length: total_length.ok_or_else(|| BError::missing_field("total_length"))?,
      name: name.ok_or_else(|| BError::missing_field("name"))?,
    })
  }
}
