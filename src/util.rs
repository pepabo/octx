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
