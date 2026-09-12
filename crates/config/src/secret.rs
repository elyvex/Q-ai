use std::fmt;
use std::ops::Deref;
use zeroize::{Zeroize, DefaultIsZeroes};

/// Wrapper that redacts secrets in Debug, Display, and Serialize.
pub struct Secret<T: Zeroize + DefaultIsZeroes> {
    inner: T,
}

impl<T: Zeroize + DefaultIsZeroes> Secret<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn expose(&self) -> &T {
        &self.inner
    }
}

impl<T: Zeroize + DefaultIsZeroes> Drop for Secret<T> {
    fn drop(&mut self) {
        self.inner.zeroize();
    }
}

impl<T: Zeroize + DefaultIsZeroes + Default> Default for Secret<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Zeroize + DefaultIsZeroes> From<T> for Secret<T> {
    fn from(inner: T) -> Self {
        Self::new(inner)
    }
}

impl<T: Zeroize + DefaultIsZeroes + Clone> Clone for Secret<T> {
    fn clone(&self) -> Self {
        Self::new(self.inner.clone())
    }
}

impl<T: Zeroize + DefaultIsZeroes> Deref for Secret<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Zeroize + DefaultIsZeroes> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(*** )")
    }
}

impl<T: Zeroize + DefaultIsZeroes> fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}

impl<T: Zeroize + DefaultIsZeroes + serde::Serialize> serde::Serialize for Secret<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("***")
    }
}

impl<'de, T: Zeroize + DefaultIsZeroes + serde::Deserialize<'de>> serde::Deserialize<'de> for Secret<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        T::deserialize(deserializer).map(Secret::new)
    }
}
