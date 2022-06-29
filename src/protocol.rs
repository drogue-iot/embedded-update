use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::ops::Deref;
use serde::{de::Visitor, Deserialize, Serialize};

/// Represents the current state of firmware and firmware being written on a device.
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Status<'a> {
    /// The current version of the firmware.
    #[serde(borrow)]
    pub version: Bytes<'a>,
    /// The max firmware block size to be sent back. The update service must ensure it does not sent larger blocks.
    pub mtu: Option<u32>,
    /// A correlation id which the update service will use when sending commands back. Used mainly when you need to multiplex multiple devices (in a gateway).
    pub correlation_id: Option<u32>,
    /// The status of the firmware being written to a device.
    pub update: Option<UpdateStatus<'a>>,
}

/// The status of the firmware being written to a device.
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct UpdateStatus<'a> {
    /// The version of the firmware being written to the device.
    #[serde(borrow)]
    pub version: Bytes<'a>,
    /// The expected next block offset to be written.
    pub offset: u32,
}

impl<'a> Status<'a> {
    /// Create an initial status update where no firmware have been written yet.
    pub fn first(version: &'a [u8], mtu: Option<u32>, correlation_id: Option<u32>) -> Self {
        Self {
            version: Bytes::new(version),
            mtu,
            correlation_id,
            update: None,
        }
    }

    /// Create a status update containing information about the firmware being written in addition to the existing firmware.
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

/// Represents a command issued from the update service to a device.
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Command<'a> {
    /// Instruct the device to wait and send its status update at a later time.
    Wait {
        /// Correlation id matching the id sent in the status update.
        correlation_id: Option<u32>,
        /// The number of seconds the device should wait before sending another status update.
        poll: Option<u32>,
    },
    /// Tell the device that it is up to date and that it can send its status update at a later time.
    Sync {
        /// The version that was used for deciding the device was up to date. The device should check it matches its own version.
        #[serde(borrow)]
        version: Bytes<'a>,
        /// Correlation id matching the id sent in the status update.
        correlation_id: Option<u32>,
        /// The number of seconds the device should wait before sending another status update.
        poll: Option<u32>,
    },
    /// A block of firmware data that should be written to the device at a given offset.
    Write {
        /// The firmware version that this block corresponds to. The device should check that this matches version it has been writing so far.
        #[serde(borrow)]
        version: Bytes<'a>,
        /// Correlation id matching the id sent in the status update.
        correlation_id: Option<u32>,
        /// The offset where this block should be written.
        offset: u32,
        /// The firmware data to write.
        #[serde(borrow)]
        data: Bytes<'a>,
    },
    /// Tell the device that it has now written all of the firmware and that it can commence the swap/update operation.
    Swap {
        /// The version that was used for deciding the device is ready to swap. The device should check it matches the version being written.
        #[serde(borrow)]
        version: Bytes<'a>,
        /// Correlation id matching the id sent in the status update.
        correlation_id: Option<u32>,
        /// The full checksum of the firmware being written. The device should compare this with the checksum of the firmware it has written before swapping.
        #[serde(borrow)]
        checksum: Bytes<'a>,
    },
}

impl<'a> Command<'a> {
    /// Create a new Wait command
    pub fn new_wait(poll: Option<u32>, correlation_id: Option<u32>) -> Self {
        Self::Wait { correlation_id, poll }
    }

    /// Create a new Sync command.
    pub fn new_sync(version: &'a [u8], poll: Option<u32>, correlation_id: Option<u32>) -> Self {
        Self::Sync {
            version: Bytes::new(version),
            correlation_id,
            poll,
        }
    }

    /// Create a new Swap command
    pub fn new_swap(version: &'a [u8], checksum: &'a [u8], correlation_id: Option<u32>) -> Self {
        Self::Swap {
            version: Bytes::new(version),
            correlation_id,
            checksum: Bytes::new(checksum),
        }
    }

    /// Create a new Write command.
    pub fn new_write(version: &'a [u8], offset: u32, data: &'a [u8], correlation_id: Option<u32>) -> Self {
        Self::Write {
            version: Bytes::new(version),
            correlation_id,
            offset,
            data: Bytes::new(data),
        }
    }
}

/// Represents a serde serializeable byte slice.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Bytes<'a> {
    data: &'a [u8],
}

impl<'a> Bytes<'a> {
    fn new(data: &'a [u8]) -> Self {
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
        self.data
    }
}

impl<'a> Deref for Bytes<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.data
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
        println!("Serialized size:\n FIRST:\t{}\nUPDATE:\t{}", first.len(), update.len(),);
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
