use serde::{Deserialize, Serialize};

pub fn to_string<T>(obj: T) -> String
    where T: Serialize
{
    serde_json::to_string(&obj).unwrap()
}

pub fn from_slice<'a, T>(buf: &'a [u8]) -> T
    where T: Deserialize<'a>
{
    serde_json::from_slice(buf).unwrap()
}