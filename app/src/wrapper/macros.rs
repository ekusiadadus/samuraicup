/// Derive the serde instance for newtype struct.
///
/// example:
/// ```
/// pub struct Id(pub String);
///
/// derive_newtype_serde!(Id, String);
/// ```
#[macro_export]
macro_rules! derive_newtype_serde {
    ($t1:ty, $t2:ty; $fn:expr, $err_msg:expr) => {
        impl Serialize for $t1 {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                self.0.serialize(serializer)
            }
        }

        impl<'de> Deserialize<'de> for $t1 {
            fn deserialize<D>(deserializer: D) -> std::result::Result<$t1, D::Error>
            where
                D: Deserializer<'de>,
            {
                let s = <$t2>::deserialize(deserializer)?;
                let ret = $fn(s).map_err(|_| de::Error::custom($err_msg))?;
                Ok(ret)
            }
        }
    };

    ($t1:tt, $t2:ty) => {
        derive_newtype_serde!(_aux; $t1, $t2; $t1);
    };

    (_aux; $t1:ty, $t2:ty; $fn:expr) => {
        impl Serialize for $t1 {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                self.0.serialize(serializer)
            }
        }

        impl<'de> Deserialize<'de> for $t1 {
            fn deserialize<D>(deserializer: D) -> std::result::Result<$t1, D::Error>
            where
                D: Deserializer<'de>,
            {
                Ok($fn(<$t2>::deserialize(deserializer)?))
            }
        }
    };
}
