use bendy::decoding::{Error, FromBencode};
use reqwest::{Client, Url};
use serde::Serialize;
use url::percent_encoding::{percent_encode, QUERY_ENCODE_SET};

use crate::peer::PeerAddr;

pub struct Tracker {
  http: Client,
  pub url: Url,
  pub request_interval: Option<std::time::Duration>,
  pub last_request: Option<std::time::Instant>,
}

#[derive(Serialize)]
pub struct Announce {
  compact: u32,
  downloaded: u32,
  uploaded: u32,
  // url encoded hash info
  info_hash: String,
  event: TrackerEvent,
  peer_id: &'static str,
  port: u32,
  left: u32,
}

pub enum TrackerRequest {
  Announce(Announce),
  Ping,
  Close,
}

#[derive(Default, Debug)]
pub struct TrackerReponse {
  pub tracker_id: u32,
  pub interval: u32,
  pub seeders: u32,
  pub leechers: u32,
  pub peers: Vec<PeerAddr>,
}

#[derive(Serialize)]
pub enum TrackerEvent {
  #[serde(rename = "started")]
  Started,
  #[serde(rename = "stoped")]
  Stoped,
  #[serde(rename = "completed")]
  Completed,
}

impl Tracker {
  pub fn new(url: Url) -> Self {
    Self {
      http: Client::new(),
      url,
      request_interval: None,
      last_request: None,
    }
  }

  pub async fn send(&mut self, req: &TrackerRequest) -> Result<TrackerReponse, reqwest::Error> {
    match req {
      // for now handling only starting annoucement by default
      TrackerRequest::Announce(ann) => {
        let ben = self
          .http
          .get(self.url.as_ref())
          .query(ann)
          .send()
          .await?
          .bytes()
          .await?;
        let r = TrackerReponse::from_bencode(&ben).unwrap();
        Ok(r)
      }
      TrackerRequest::Ping => Ok(TrackerReponse::default()),
      TrackerRequest::Close => Ok(TrackerReponse::default()),
    }
  }
}

impl FromBencode for TrackerReponse {
  fn decode_bencode_object(object: bendy::decoding::Object) -> Result<Self, bendy::decoding::Error> {
    let mut tracker_id = None;
    let mut interval = None;
    let mut peers = Vec::<PeerAddr>::new();

    let mut tresp_dict = object.try_into_dictionary()?;

    while let Some(pair) = tresp_dict.next_pair()? {
      match pair {
        (b"interval", value) => {
          interval = Some(value.try_into_integer()?.parse()?);
        }
        (b"tracker id", value) => {
          tracker_id = Some(value.try_into_integer()?.parse()?);
        }
        (b"peers", value) => {
          let peers_compact = value.try_into_bytes()?;
          for chunck in peers_compact.chunks(6) {
            peers.push(PeerAddr::new(chunck));
          }
        }
        (_, _) => {}
      }
    }

    Ok(Self {
      tracker_id: tracker_id.ok_or_else(|| Error::missing_field("tracker id"))?,
      interval: interval.ok_or_else(|| Error::missing_field("interval"))?,
      seeders: 0,
      leechers: 0,
      peers,
    })
  }
}

impl Announce {
  pub fn new(
    downloaded: u32,
    uploaded: u32,
    info_hash: &[u8; 20],
    event: TrackerEvent,
    peer_id: &'static str,
    left: u32,
  ) -> Self {
    Self {
      compact: 1,
      downloaded,
      uploaded,
      info_hash: percent_encode(info_hash, QUERY_ENCODE_SET).to_string(),
      event,
      peer_id,
      port: 6969,
      left,
    }
  }
}
