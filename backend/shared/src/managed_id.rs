pub trait ManagedId: Sized + Clone + Eq + PartialEq {
    fn get_specifier() -> &'static str;
    fn id(&self) -> &str;
    fn from_id(id: String) -> Self;

    fn full(&self) -> String {
        format!("{}:{}", Self::get_specifier(), self.id())
    }

    fn parse(s: &str) -> Self {
        let mut parts = s.splitn(2, ':');
        let spec = parts.next().unwrap_or("");
        let id = parts.next().unwrap_or("");
        if spec != Self::get_specifier() {
            panic!(
                "Invalid specifier: expected '{}', got '{}'",
                Self::get_specifier(),
                spec
            );
        }
        Self::from_id(id.to_string())
    }
}

#[macro_export]
macro_rules! define_managed_id {
    ($name:ident, $specifier:literal) => {
        #[derive(Clone, PartialEq, Eq, Hash)]
        pub struct $name(pub String);

        impl $crate::ManagedId for $name {
            fn get_specifier() -> &'static str {
                $specifier
            }

            fn id(&self) -> &str {
                &self.0
            }

            fn from_id(id: String) -> Self {
                $name(id)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}:{}", $specifier, self.0)
            }
        }

        impl std::string::ToString for $name {
            fn to_string(&self) -> String {
                format!("{}:{}", $specifier, self.0)
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                <$name as $crate::ManagedId>::parse(&s)
            }
        }

        impl<'a> From<&'a str> for $name {
            fn from(s: &'a str) -> Self {
                <$name as $crate::ManagedId>::parse(s)
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&self.to_string())
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                struct Visitor;

                impl<'de> serde::de::Visitor<'de> for Visitor {
                    type Value = $name;

                    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(f, "a string in format '{}:id'", $specifier)
                    }

                    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<$name, E> {
                        Ok(<$name as $crate::ManagedId>::parse(value))
                    }
                }

                deserializer.deserialize_str(Visitor)
            }
        }
    };
}
