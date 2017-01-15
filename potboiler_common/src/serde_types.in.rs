use serde_json;
use hybrid_clocks::{Timestamp, WallT};
use uuid::Uuid;

// from https://serde.rs/enum-str.html
#[macro_export]
macro_rules! enum_str {
    ($name:ident { $($variant:ident($str:expr), )* }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name {
            $($variant,)*
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                where S: ::serde::Serializer,
            {
                // Serialize the enum as a string.
                serializer.serialize_str(match *self {
                    $( $name::$variant => $str, )*
                })
            }
        }

        impl ::serde::Deserialize for $name {
            fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
                where D: ::serde::Deserializer,
            {
                struct Visitor;

                impl ::serde::de::Visitor for Visitor {
                    type Value = $name;

                    fn visit_str<E>(&mut self, value: &str) -> Result<$name, E>
                        where E: ::serde::de::Error,
                    {
                        match value {
                            $( $str => Ok($name::$variant), )*
                            _ => Err(E::invalid_value(
                                &format!("unknown {} variant: {}",
                                stringify!($name), value))),
                        }
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
    pub data: serde_json::Value
}
