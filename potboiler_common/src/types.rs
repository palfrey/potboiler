use hybrid_clocks::{Timestamp, WallT};
use serde_derive::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

#[macro_export]
macro_rules! enum_str {
    ($name:ident { $($variant:ident($str:expr), )* }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name {
            $($variant,)*
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: ::serde::Serializer,
            {
                // Serialize the enum as a string.
                serializer.serialize_str(match *self {
                    $( $name::$variant => $str, )*
                })
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: ::serde::Deserializer<'de>,
            {
                struct Visitor;

                impl<'de> ::serde::de::Visitor<'de> for Visitor {
                    type Value = $name;

                    fn visit_str<E>(self, value: &str) -> Result<$name, E>
                        where E: ::serde::de::Error,
                    {
                        match value {
                            $( $str => Ok($name::$variant), )*
                            _ => Err(E::invalid_value(::serde::de::Unexpected::Str(&value), &self))
                        }
                    }

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str(&format!("{} enumeration", stringify!($name)))
                    }
                }

                // Deserialize the enum from a string.
                deserializer.deserialize_str(Visitor)
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum CRDT {
    LWW,
    GSET,
    ORSET,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Log {
    pub id: Uuid,
    pub owner: Uuid,
    pub prev: Option<Uuid>,
    pub next: Option<Uuid>,
    pub when: Timestamp<WallT>,
    pub data: serde_json::Value,
    pub dependencies: Vec<Uuid>,
}
