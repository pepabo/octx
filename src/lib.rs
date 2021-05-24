extern crate serde_urlencoded;

pub mod comments;
pub mod events;
pub mod issues;
pub mod labels;
pub mod users;

use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Params {
    pub per_page: Option<u8>,
}

impl Params {
    pub fn to_query(&self) -> String {
        serde_urlencoded::to_string(self).unwrap()
    }
}
