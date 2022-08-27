use std::hash::Hasher;
use std::ops::Deref;

use siphasher::sip128::{Hasher128, SipHasher24};

use crate::StrId;

#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash)]
#[repr(transparent)]
pub struct StrHash {
    hash: u128,
}

impl StrHash {
    pub fn new(value: &str) -> Self {
        let mut hasher = SipHasher24::new();
        hasher.write(value.as_bytes());
        Self {
            hash: hasher.finish128().into(),
        }
    }

    #[inline]
    pub fn from_be_bytes(bytes: [u8; 16]) -> Self {
        Self {
            hash: u128::from_be_bytes(bytes),
        }
    }

    #[inline]
    pub fn to_be_bytes(self) -> [u8; 16] {
        self.hash.to_be_bytes()
    }
}

impl StrId for StrHash {}

impl Deref for StrHash {
    type Target = u128;

    fn deref(&self) -> &Self::Target {
        &self.hash
    }
}
