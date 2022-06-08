use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::ops::Deref;
use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Status<'a> {
    #[serde(borrow)]
    pub version: Bytes<'a>,
    pub mtu: Option<u32>,
    pub correlation_id: Option<u32>,
    pub update: Option<UpdateStatus<'a>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct UpdateStatus<'a> {
    #[serde(borrow)]
    pub version: Bytes<'a>,
    pub offset: u32,
}

impl<'a> Status<'a> {
    pub fn first(version: &'a [u8], mtu: Option<u32>, correlation_id: Option<u32>) -> Self {
        Self {
            version: Bytes::new(version),
            mtu,
            correlation_id,
            update: None,
        }
    }

    pub fn update(
        version: &'a [u8],
        mtu: Option<u32>,
        offset: u32,
        next_version: &'a [u8],
        correlation_id: Option<u32>,
    ) -> Self {
        Self {
            version: Bytes::new(version),
            mtu,
            correlation_id,
            update: Some(UpdateStatus {
                offset,
                version: Bytes::new(next_version),
            }),
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Command<'a> {
    Wait {
        correlation_id: Option<u32>,
        poll: Option<u32>,
    },
    Sync {
        #[serde(borrow)]
        version: Bytes<'a>,
        correlation_id: Option<u32>,
        poll: Option<u32>,
    },
    Write {
        #[serde(borrow)]
        version: Bytes<'a>,
        correlation_id: Option<u32>,
        offset: u32,
        #[serde(borrow)]
        data: Bytes<'a>,
    },
    Swap {
        #[serde(borrow)]
        version: Bytes<'a>,
        correlation_id: Option<u32>,
        #[serde(borrow)]
        checksum: Bytes<'a>,
    },
}

impl<'a> Command<'a> {
    pub fn new_wait(poll: Option<u32>, correlation_id: Option<u32>) -> Self {
        Self::Wait {
            correlation_id,
            poll,
        }
    }

    pub fn new_sync(version: &'a [u8], poll: Option<u32>, correlation_id: Option<u32>) -> Self {
        Self::Sync {
            version: Bytes::new(version),
            correlation_id,
            poll,
        }
    }

    pub fn new_swap(version: &'a [u8], checksum: &'a [u8], correlation_id: Option<u32>) -> Self {
        Self::Swap {
            version: Bytes::new(version),
            correlation_id,
            checksum: Bytes::new(checksum),
        }
    }

    pub fn new_write(
        version: &'a [u8],
        offset: u32,
        data: &'a [u8],
        correlation_id: Option<u32>,
    ) -> Self {
        Self::Write {
            version: Bytes::new(version),
            correlation_id,
            offset,
            data: Bytes::new(data),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Bytes<'a> {
    data: &'a [u8],
}

impl<'a> Bytes<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl<'a> Serialize for Bytes<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(self.data)
    }
}

impl<'a, 'de: 'a> Deserialize<'de> for Bytes<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(BytesVisitor)
    }
}

impl<'a> AsRef<[u8]> for Bytes<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl<'a> Deref for Bytes<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a> Default for Bytes<'a> {
    fn default() -> Self {
        Bytes::new(&[])
    }
}

impl<'a, Rhs> PartialEq<Rhs> for Bytes<'a>
where
    Rhs: ?Sized + AsRef<[u8]>,
{
    fn eq(&self, other: &Rhs) -> bool {
        self.as_ref().eq(other.as_ref())
    }
}

impl<'a, Rhs> PartialOrd<Rhs> for Bytes<'a>
where
    Rhs: ?Sized + AsRef<[u8]>,
{
    fn partial_cmp(&self, other: &Rhs) -> Option<Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl<'a> Hash for Bytes<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

struct BytesVisitor;

impl<'de> Visitor<'de> for BytesVisitor {
    type Value = Bytes<'de>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("a byte slice")
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Bytes::new(v))
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::println;
    use std::vec::Vec;

    #[test]
    fn deserialize_ref() {
        let s = Command::new_write(b"1234", 0, &[1, 2, 3, 4], None);
        let out = serde_cbor::to_vec(&s).unwrap();

        let s: Command = serde_cbor::from_slice(&out).unwrap();
        println!("Out: {:?}", s);
    }

    #[test]
    fn serialized_status_size() {
        // 1 byte version, 4 byte payload, 4 byte checksum
        let version = &[1];
        let mtu = Some(4);
        let cid = None;
        let offset = 0;
        let next_version = &[2];

        let s = Status::first(version, mtu, cid);
        let first = encode(&s);

        let s = Status::update(version, mtu, offset, next_version, cid);
        let update = encode(&s);
        println!(
            "Serialized size:\n FIRST:\t{}\nUPDATE:\t{}",
            first.len(),
            update.len(),
        );
    }

    #[test]
    fn serialized_command_size() {
        // 1 byte version, 4 byte payload, 4 byte checksum
        let version = &[1];
        let payload = &[1, 2, 3, 4];
        let checksum = &[1, 2, 3, 4];

        let s = Command::new_write(version, 0, payload, None);
        let write = encode(&s);

        let s = Command::new_wait(Some(1), None);
        let wait = encode(&s);

        let s = Command::new_sync(version, Some(1), None);
        let sync = encode(&s);

        let s = Command::new_swap(version, checksum, None);
        let swap = encode(&s);
        println!(
            "Serialized size:\n WRITE:\t{}\nWAIT:\t{}\nSYNC:\t{}\nSWAP:\t{}",
            write.len(),
            wait.len(),
            sync.len(),
            swap.len()
        );
    }

    fn encode<T>(value: &T) -> Vec<u8>
    where
        T: serde::Serialize,
    {
        serde_cbor::ser::to_vec_packed(value).unwrap()
    }
}
