pub mod entities {
    use chrono::{DateTime, Utc};
    use gitops_lib::{GitopsEnum, GitopsResourcePart, GitopsResourceRoot};
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    #[gitops(key = "uid")]
    pub struct User {
        pub uid: String,
        pub password_hash: Option<String>,
        pub oauth: Option<String>,
        pub created_at: String,
        pub annotations: HashMap<String, String>,
        pub has_admin_status: bool,
    }
    impl From<User> for UserGitopsSerializable {
        fn from(resource: User) -> Self {
            Self {
                kind: "User".to_string(),
                api_version: "v1.0".to_string(),
                uid: resource.uid,
                password_hash: resource.password_hash,
                oauth: resource.oauth,
                created_at: resource.created_at,
                annotations: resource.annotations,
                has_admin_status: resource.has_admin_status,
            }
        }
    }
    impl From<UserGitopsSerializable> for User {
        fn from(serializable_resource: UserGitopsSerializable) -> Self {
            Self {
                uid: serializable_resource.uid,
                password_hash: serializable_resource.password_hash,
                oauth: serializable_resource.oauth,
                created_at: serializable_resource.created_at,
                annotations: serializable_resource.annotations,
                has_admin_status: serializable_resource.has_admin_status,
            }
        }
    }
    #[serde(rename_all = "camelCase")]
    pub struct UserGitopsUpdate {
        pub uid: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub password_hash: Option<Option<String>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub oauth: Option<Option<String>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub created_at: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub annotations: Option<HashMap<String, String>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub has_admin_status: Option<bool>,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UserGitopsUpdate {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "uid",
                "password_hash",
                "oauth",
                "created_at",
                "annotations",
                "has_admin_status",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.uid,
                &self.password_hash,
                &self.oauth,
                &self.created_at,
                &self.annotations,
                &&self.has_admin_status,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "UserGitopsUpdate",
                names,
                values,
            )
        }
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for UserGitopsUpdate {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "UserGitopsUpdate",
                    false as usize + 1
                        + if Option::is_none(&self.password_hash) { 0 } else { 1 }
                        + if Option::is_none(&self.oauth) { 0 } else { 1 }
                        + if Option::is_none(&self.created_at) { 0 } else { 1 }
                        + if Option::is_none(&self.annotations) { 0 } else { 1 }
                        + if Option::is_none(&self.has_admin_status) { 0 } else { 1 },
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "uid",
                    &self.uid,
                )?;
                if !Option::is_none(&self.password_hash) {
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "passwordHash",
                        &self.password_hash,
                    )?;
                } else {
                    _serde::ser::SerializeStruct::skip_field(
                        &mut __serde_state,
                        "passwordHash",
                    )?;
                }
                if !Option::is_none(&self.oauth) {
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "oauth",
                        &self.oauth,
                    )?;
                } else {
                    _serde::ser::SerializeStruct::skip_field(
                        &mut __serde_state,
                        "oauth",
                    )?;
                }
                if !Option::is_none(&self.created_at) {
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "createdAt",
                        &self.created_at,
                    )?;
                } else {
                    _serde::ser::SerializeStruct::skip_field(
                        &mut __serde_state,
                        "createdAt",
                    )?;
                }
                if !Option::is_none(&self.annotations) {
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "annotations",
                        &self.annotations,
                    )?;
                } else {
                    _serde::ser::SerializeStruct::skip_field(
                        &mut __serde_state,
                        "annotations",
                    )?;
                }
                if !Option::is_none(&self.has_admin_status) {
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "hasAdminStatus",
                        &self.has_admin_status,
                    )?;
                } else {
                    _serde::ser::SerializeStruct::skip_field(
                        &mut __serde_state,
                        "hasAdminStatus",
                    )?;
                }
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for UserGitopsUpdate {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __field4,
                    __field5,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            3u64 => _serde::__private::Ok(__Field::__field3),
                            4u64 => _serde::__private::Ok(__Field::__field4),
                            5u64 => _serde::__private::Ok(__Field::__field5),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "uid" => _serde::__private::Ok(__Field::__field0),
                            "passwordHash" => _serde::__private::Ok(__Field::__field1),
                            "oauth" => _serde::__private::Ok(__Field::__field2),
                            "createdAt" => _serde::__private::Ok(__Field::__field3),
                            "annotations" => _serde::__private::Ok(__Field::__field4),
                            "hasAdminStatus" => _serde::__private::Ok(__Field::__field5),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"uid" => _serde::__private::Ok(__Field::__field0),
                            b"passwordHash" => _serde::__private::Ok(__Field::__field1),
                            b"oauth" => _serde::__private::Ok(__Field::__field2),
                            b"createdAt" => _serde::__private::Ok(__Field::__field3),
                            b"annotations" => _serde::__private::Ok(__Field::__field4),
                            b"hasAdminStatus" => _serde::__private::Ok(__Field::__field5),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<UserGitopsUpdate>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = UserGitopsUpdate;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct UserGitopsUpdate",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct UserGitopsUpdate with 6 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            Option<Option<String>>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                _serde::__private::Default::default()
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            Option<Option<String>>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                _serde::__private::Default::default()
                            }
                        };
                        let __field3 = match _serde::de::SeqAccess::next_element::<
                            Option<String>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                _serde::__private::Default::default()
                            }
                        };
                        let __field4 = match _serde::de::SeqAccess::next_element::<
                            Option<HashMap<String, String>>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                _serde::__private::Default::default()
                            }
                        };
                        let __field5 = match _serde::de::SeqAccess::next_element::<
                            Option<bool>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                _serde::__private::Default::default()
                            }
                        };
                        _serde::__private::Ok(UserGitopsUpdate {
                            uid: __field0,
                            password_hash: __field1,
                            oauth: __field2,
                            created_at: __field3,
                            annotations: __field4,
                            has_admin_status: __field5,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field1: _serde::__private::Option<
                            Option<Option<String>>,
                        > = _serde::__private::None;
                        let mut __field2: _serde::__private::Option<
                            Option<Option<String>>,
                        > = _serde::__private::None;
                        let mut __field3: _serde::__private::Option<Option<String>> = _serde::__private::None;
                        let mut __field4: _serde::__private::Option<
                            Option<HashMap<String, String>>,
                        > = _serde::__private::None;
                        let mut __field5: _serde::__private::Option<Option<bool>> = _serde::__private::None;
                        while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("uid"),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "passwordHash",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Option<Option<String>>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private::Option::is_some(&__field2) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("oauth"),
                                        );
                                    }
                                    __field2 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Option<Option<String>>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private::Option::is_some(&__field3) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "createdAt",
                                            ),
                                        );
                                    }
                                    __field3 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Option<String>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field4 => {
                                    if _serde::__private::Option::is_some(&__field4) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "annotations",
                                            ),
                                        );
                                    }
                                    __field4 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Option<HashMap<String, String>>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field5 => {
                                    if _serde::__private::Option::is_some(&__field5) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "hasAdminStatus",
                                            ),
                                        );
                                    }
                                    __field5 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Option<bool>,
                                        >(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("uid")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                _serde::__private::Default::default()
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private::Some(__field2) => __field2,
                            _serde::__private::None => {
                                _serde::__private::Default::default()
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private::Some(__field3) => __field3,
                            _serde::__private::None => {
                                _serde::__private::Default::default()
                            }
                        };
                        let __field4 = match __field4 {
                            _serde::__private::Some(__field4) => __field4,
                            _serde::__private::None => {
                                _serde::__private::Default::default()
                            }
                        };
                        let __field5 = match __field5 {
                            _serde::__private::Some(__field5) => __field5,
                            _serde::__private::None => {
                                _serde::__private::Default::default()
                            }
                        };
                        _serde::__private::Ok(UserGitopsUpdate {
                            uid: __field0,
                            password_hash: __field1,
                            oauth: __field2,
                            created_at: __field3,
                            annotations: __field4,
                            has_admin_status: __field5,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "uid",
                    "passwordHash",
                    "oauth",
                    "createdAt",
                    "annotations",
                    "hasAdminStatus",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "UserGitopsUpdate",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<UserGitopsUpdate>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    impl gitops_lib::GitopsResourceRoot for User {
        type Serializable = UserGitopsSerializable;
        type Update = UserGitopsUpdate;
        fn as_serializable(&self) -> Self::Serializable {
            Self::Serializable {
                kind: "User".to_string(),
                api_version: "v1.0".to_string(),
                uid: self.uid.clone(),
                password_hash: self.password_hash.clone(),
                oauth: self.oauth.clone(),
                created_at: self.created_at.clone(),
                annotations: self.annotations.clone(),
                has_admin_status: self.has_admin_status.clone(),
            }
        }
        fn into_serializable(self) -> Self::Serializable {
            Self::Serializable {
                kind: "User".to_string(),
                api_version: "v1.0".to_string(),
                uid: self.uid,
                password_hash: self.password_hash,
                oauth: self.oauth,
                created_at: self.created_at,
                annotations: self.annotations,
                has_admin_status: self.has_admin_status,
            }
        }
        fn get_kind(&self) -> String {
            "User".to_string()
        }
        fn get_key(&self) -> String {
            self.uid.clone()
        }
        fn kind() -> &'static str {
            "User"
        }
        fn with_updates_from(self, updates: Self::Update) -> Self {
            if self.uid != updates.uid {
                {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "Attempted to merge updates from an object with a different key. Current key: {0}, Update key: {1}",
                            self.uid,
                            updates.uid,
                        ),
                    );
                };
            }
            let mut updated = self;
            if let Some(new_value) = updates.password_hash {
                updated.password_hash = new_value;
            }
            if let Some(new_value) = updates.oauth {
                updated.oauth = new_value;
            }
            if let Some(new_value) = updates.created_at {
                updated.created_at = new_value;
            }
            if let Some(new_value) = updates.annotations {
                updated.annotations = new_value;
            }
            if let Some(new_value) = updates.has_admin_status {
                updated.has_admin_status = new_value;
            }
            updated
        }
    }
    #[serde(rename_all = "camelCase")]
    #[allow(non_snake_case)]
    pub struct UserGitopsSerializable {
        pub kind: String,
        #[serde(rename = "apiVersion")]
        pub api_version: String,
        pub uid: String,
        pub password_hash: Option<String>,
        pub oauth: Option<String>,
        pub created_at: String,
        pub annotations: HashMap<String, String>,
        pub has_admin_status: bool,
    }
    #[automatically_derived]
    #[allow(non_snake_case)]
    impl ::core::fmt::Debug for UserGitopsSerializable {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "kind",
                "api_version",
                "uid",
                "password_hash",
                "oauth",
                "created_at",
                "annotations",
                "has_admin_status",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.kind,
                &self.api_version,
                &self.uid,
                &self.password_hash,
                &self.oauth,
                &self.created_at,
                &self.annotations,
                &&self.has_admin_status,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "UserGitopsSerializable",
                names,
                values,
            )
        }
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for UserGitopsSerializable {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "UserGitopsSerializable",
                    false as usize + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "kind",
                    &self.kind,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "apiVersion",
                    &self.api_version,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "uid",
                    &self.uid,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "passwordHash",
                    &self.password_hash,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "oauth",
                    &self.oauth,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "createdAt",
                    &self.created_at,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "annotations",
                    &self.annotations,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "hasAdminStatus",
                    &self.has_admin_status,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for UserGitopsSerializable {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __field4,
                    __field5,
                    __field6,
                    __field7,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            3u64 => _serde::__private::Ok(__Field::__field3),
                            4u64 => _serde::__private::Ok(__Field::__field4),
                            5u64 => _serde::__private::Ok(__Field::__field5),
                            6u64 => _serde::__private::Ok(__Field::__field6),
                            7u64 => _serde::__private::Ok(__Field::__field7),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "kind" => _serde::__private::Ok(__Field::__field0),
                            "apiVersion" => _serde::__private::Ok(__Field::__field1),
                            "uid" => _serde::__private::Ok(__Field::__field2),
                            "passwordHash" => _serde::__private::Ok(__Field::__field3),
                            "oauth" => _serde::__private::Ok(__Field::__field4),
                            "createdAt" => _serde::__private::Ok(__Field::__field5),
                            "annotations" => _serde::__private::Ok(__Field::__field6),
                            "hasAdminStatus" => _serde::__private::Ok(__Field::__field7),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"kind" => _serde::__private::Ok(__Field::__field0),
                            b"apiVersion" => _serde::__private::Ok(__Field::__field1),
                            b"uid" => _serde::__private::Ok(__Field::__field2),
                            b"passwordHash" => _serde::__private::Ok(__Field::__field3),
                            b"oauth" => _serde::__private::Ok(__Field::__field4),
                            b"createdAt" => _serde::__private::Ok(__Field::__field5),
                            b"annotations" => _serde::__private::Ok(__Field::__field6),
                            b"hasAdminStatus" => _serde::__private::Ok(__Field::__field7),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<UserGitopsSerializable>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = UserGitopsSerializable;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct UserGitopsSerializable",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct UserGitopsSerializable with 8 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct UserGitopsSerializable with 8 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct UserGitopsSerializable with 8 elements",
                                    ),
                                );
                            }
                        };
                        let __field3 = match _serde::de::SeqAccess::next_element::<
                            Option<String>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        3usize,
                                        &"struct UserGitopsSerializable with 8 elements",
                                    ),
                                );
                            }
                        };
                        let __field4 = match _serde::de::SeqAccess::next_element::<
                            Option<String>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        4usize,
                                        &"struct UserGitopsSerializable with 8 elements",
                                    ),
                                );
                            }
                        };
                        let __field5 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        5usize,
                                        &"struct UserGitopsSerializable with 8 elements",
                                    ),
                                );
                            }
                        };
                        let __field6 = match _serde::de::SeqAccess::next_element::<
                            HashMap<String, String>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        6usize,
                                        &"struct UserGitopsSerializable with 8 elements",
                                    ),
                                );
                            }
                        };
                        let __field7 = match _serde::de::SeqAccess::next_element::<
                            bool,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        7usize,
                                        &"struct UserGitopsSerializable with 8 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private::Ok(UserGitopsSerializable {
                            kind: __field0,
                            api_version: __field1,
                            uid: __field2,
                            password_hash: __field3,
                            oauth: __field4,
                            created_at: __field5,
                            annotations: __field6,
                            has_admin_status: __field7,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field1: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field2: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field3: _serde::__private::Option<Option<String>> = _serde::__private::None;
                        let mut __field4: _serde::__private::Option<Option<String>> = _serde::__private::None;
                        let mut __field5: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field6: _serde::__private::Option<
                            HashMap<String, String>,
                        > = _serde::__private::None;
                        let mut __field7: _serde::__private::Option<bool> = _serde::__private::None;
                        while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("kind"),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "apiVersion",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private::Option::is_some(&__field2) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("uid"),
                                        );
                                    }
                                    __field2 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private::Option::is_some(&__field3) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "passwordHash",
                                            ),
                                        );
                                    }
                                    __field3 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Option<String>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field4 => {
                                    if _serde::__private::Option::is_some(&__field4) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("oauth"),
                                        );
                                    }
                                    __field4 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Option<String>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field5 => {
                                    if _serde::__private::Option::is_some(&__field5) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "createdAt",
                                            ),
                                        );
                                    }
                                    __field5 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field6 => {
                                    if _serde::__private::Option::is_some(&__field6) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "annotations",
                                            ),
                                        );
                                    }
                                    __field6 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            HashMap<String, String>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field7 => {
                                    if _serde::__private::Option::is_some(&__field7) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "hasAdminStatus",
                                            ),
                                        );
                                    }
                                    __field7 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<bool>(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("kind")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("apiVersion")?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private::Some(__field2) => __field2,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("uid")?
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private::Some(__field3) => __field3,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("passwordHash")?
                            }
                        };
                        let __field4 = match __field4 {
                            _serde::__private::Some(__field4) => __field4,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("oauth")?
                            }
                        };
                        let __field5 = match __field5 {
                            _serde::__private::Some(__field5) => __field5,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("createdAt")?
                            }
                        };
                        let __field6 = match __field6 {
                            _serde::__private::Some(__field6) => __field6,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("annotations")?
                            }
                        };
                        let __field7 = match __field7 {
                            _serde::__private::Some(__field7) => __field7,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("hasAdminStatus")?
                            }
                        };
                        _serde::__private::Ok(UserGitopsSerializable {
                            kind: __field0,
                            api_version: __field1,
                            uid: __field2,
                            password_hash: __field3,
                            oauth: __field4,
                            created_at: __field5,
                            annotations: __field6,
                            has_admin_status: __field7,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "kind",
                    "apiVersion",
                    "uid",
                    "passwordHash",
                    "oauth",
                    "createdAt",
                    "annotations",
                    "hasAdminStatus",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "UserGitopsSerializable",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<UserGitopsSerializable>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::fmt::Debug for User {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "uid",
                "password_hash",
                "oauth",
                "created_at",
                "annotations",
                "has_admin_status",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.uid,
                &self.password_hash,
                &self.oauth,
                &self.created_at,
                &self.annotations,
                &&self.has_admin_status,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(f, "User", names, values)
        }
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for User {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "User",
                    false as usize + 1 + 1 + 1 + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "uid",
                    &self.uid,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "password_hash",
                    &self.password_hash,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "oauth",
                    &self.oauth,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "created_at",
                    &self.created_at,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "annotations",
                    &self.annotations,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "has_admin_status",
                    &self.has_admin_status,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for User {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __field4,
                    __field5,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            3u64 => _serde::__private::Ok(__Field::__field3),
                            4u64 => _serde::__private::Ok(__Field::__field4),
                            5u64 => _serde::__private::Ok(__Field::__field5),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "uid" => _serde::__private::Ok(__Field::__field0),
                            "password_hash" => _serde::__private::Ok(__Field::__field1),
                            "oauth" => _serde::__private::Ok(__Field::__field2),
                            "created_at" => _serde::__private::Ok(__Field::__field3),
                            "annotations" => _serde::__private::Ok(__Field::__field4),
                            "has_admin_status" => {
                                _serde::__private::Ok(__Field::__field5)
                            }
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"uid" => _serde::__private::Ok(__Field::__field0),
                            b"password_hash" => _serde::__private::Ok(__Field::__field1),
                            b"oauth" => _serde::__private::Ok(__Field::__field2),
                            b"created_at" => _serde::__private::Ok(__Field::__field3),
                            b"annotations" => _serde::__private::Ok(__Field::__field4),
                            b"has_admin_status" => {
                                _serde::__private::Ok(__Field::__field5)
                            }
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<User>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = User;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct User",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct User with 6 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            Option<String>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct User with 6 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            Option<String>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct User with 6 elements",
                                    ),
                                );
                            }
                        };
                        let __field3 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        3usize,
                                        &"struct User with 6 elements",
                                    ),
                                );
                            }
                        };
                        let __field4 = match _serde::de::SeqAccess::next_element::<
                            HashMap<String, String>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        4usize,
                                        &"struct User with 6 elements",
                                    ),
                                );
                            }
                        };
                        let __field5 = match _serde::de::SeqAccess::next_element::<
                            bool,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        5usize,
                                        &"struct User with 6 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private::Ok(User {
                            uid: __field0,
                            password_hash: __field1,
                            oauth: __field2,
                            created_at: __field3,
                            annotations: __field4,
                            has_admin_status: __field5,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field1: _serde::__private::Option<Option<String>> = _serde::__private::None;
                        let mut __field2: _serde::__private::Option<Option<String>> = _serde::__private::None;
                        let mut __field3: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field4: _serde::__private::Option<
                            HashMap<String, String>,
                        > = _serde::__private::None;
                        let mut __field5: _serde::__private::Option<bool> = _serde::__private::None;
                        while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("uid"),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "password_hash",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Option<String>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private::Option::is_some(&__field2) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("oauth"),
                                        );
                                    }
                                    __field2 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Option<String>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private::Option::is_some(&__field3) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "created_at",
                                            ),
                                        );
                                    }
                                    __field3 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field4 => {
                                    if _serde::__private::Option::is_some(&__field4) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "annotations",
                                            ),
                                        );
                                    }
                                    __field4 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            HashMap<String, String>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field5 => {
                                    if _serde::__private::Option::is_some(&__field5) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "has_admin_status",
                                            ),
                                        );
                                    }
                                    __field5 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<bool>(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("uid")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("password_hash")?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private::Some(__field2) => __field2,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("oauth")?
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private::Some(__field3) => __field3,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("created_at")?
                            }
                        };
                        let __field4 = match __field4 {
                            _serde::__private::Some(__field4) => __field4,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("annotations")?
                            }
                        };
                        let __field5 = match __field5 {
                            _serde::__private::Some(__field5) => __field5,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("has_admin_status")?
                            }
                        };
                        _serde::__private::Ok(User {
                            uid: __field0,
                            password_hash: __field1,
                            oauth: __field2,
                            created_at: __field3,
                            annotations: __field4,
                            has_admin_status: __field5,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "uid",
                    "password_hash",
                    "oauth",
                    "created_at",
                    "annotations",
                    "has_admin_status",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "User",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<User>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for User {
        #[inline]
        fn clone(&self) -> User {
            User {
                uid: ::core::clone::Clone::clone(&self.uid),
                password_hash: ::core::clone::Clone::clone(&self.password_hash),
                oauth: ::core::clone::Clone::clone(&self.oauth),
                created_at: ::core::clone::Clone::clone(&self.created_at),
                annotations: ::core::clone::Clone::clone(&self.annotations),
                has_admin_status: ::core::clone::Clone::clone(&self.has_admin_status),
            }
        }
    }
    pub struct VisibilityConfig {
        pub public_visible: bool,
        pub public_can_report: bool,
        pub public_can_see_tickets: Vec<String>,
    }
    #[serde(rename_all = "camelCase")]
    pub struct VisibilityConfigGitopsUpdate {
        pub public_visible: Option<bool>,
        pub public_can_report: Option<bool>,
        pub public_can_see_tickets: Option<Vec<String>>,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for VisibilityConfigGitopsUpdate {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "VisibilityConfigGitopsUpdate",
                "public_visible",
                &self.public_visible,
                "public_can_report",
                &self.public_can_report,
                "public_can_see_tickets",
                &&self.public_can_see_tickets,
            )
        }
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for VisibilityConfigGitopsUpdate {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "VisibilityConfigGitopsUpdate",
                    false as usize + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "publicVisible",
                    &self.public_visible,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "publicCanReport",
                    &self.public_can_report,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "publicCanSeeTickets",
                    &self.public_can_see_tickets,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for VisibilityConfigGitopsUpdate {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "publicVisible" => _serde::__private::Ok(__Field::__field0),
                            "publicCanReport" => _serde::__private::Ok(__Field::__field1),
                            "publicCanSeeTickets" => {
                                _serde::__private::Ok(__Field::__field2)
                            }
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"publicVisible" => _serde::__private::Ok(__Field::__field0),
                            b"publicCanReport" => {
                                _serde::__private::Ok(__Field::__field1)
                            }
                            b"publicCanSeeTickets" => {
                                _serde::__private::Ok(__Field::__field2)
                            }
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<VisibilityConfigGitopsUpdate>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = VisibilityConfigGitopsUpdate;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct VisibilityConfigGitopsUpdate",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            Option<bool>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct VisibilityConfigGitopsUpdate with 3 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            Option<bool>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct VisibilityConfigGitopsUpdate with 3 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            Option<Vec<String>>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct VisibilityConfigGitopsUpdate with 3 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private::Ok(VisibilityConfigGitopsUpdate {
                            public_visible: __field0,
                            public_can_report: __field1,
                            public_can_see_tickets: __field2,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<Option<bool>> = _serde::__private::None;
                        let mut __field1: _serde::__private::Option<Option<bool>> = _serde::__private::None;
                        let mut __field2: _serde::__private::Option<
                            Option<Vec<String>>,
                        > = _serde::__private::None;
                        while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "publicVisible",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Option<bool>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "publicCanReport",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Option<bool>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private::Option::is_some(&__field2) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "publicCanSeeTickets",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Option<Vec<String>>,
                                        >(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("publicVisible")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("publicCanReport")?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private::Some(__field2) => __field2,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("publicCanSeeTickets")?
                            }
                        };
                        _serde::__private::Ok(VisibilityConfigGitopsUpdate {
                            public_visible: __field0,
                            public_can_report: __field1,
                            public_can_see_tickets: __field2,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "publicVisible",
                    "publicCanReport",
                    "publicCanSeeTickets",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "VisibilityConfigGitopsUpdate",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<
                            VisibilityConfigGitopsUpdate,
                        >,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    impl gitops_lib::GitopsResourcePart for VisibilityConfig {
        fn with_updates_from_part(self, updates: Self::UpdatePart) -> Self {
            let mut updated = self;
            if let Some(new_value) = updates.public_visible {
                updated.public_visible = new_value;
            }
            if let Some(new_value) = updates.public_can_report {
                updated.public_can_report = new_value;
            }
            if let Some(new_value) = updates.public_can_see_tickets {
                updated.public_can_see_tickets = new_value;
            }
            updated
        }
        fn as_update(&self) -> Self::UpdatePart {
            VisibilityConfigGitopsUpdate {
                public_visible: Some(self.public_visible.clone()),
                public_can_report: Some(self.public_can_report.clone()),
                public_can_see_tickets: Some(self.public_can_see_tickets.clone()),
            }
        }
        type UpdatePart = VisibilityConfigGitopsUpdate;
    }
    impl VisibilityConfig {}
    #[automatically_derived]
    impl ::core::fmt::Debug for VisibilityConfig {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "VisibilityConfig",
                "public_visible",
                &self.public_visible,
                "public_can_report",
                &self.public_can_report,
                "public_can_see_tickets",
                &&self.public_can_see_tickets,
            )
        }
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for VisibilityConfig {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "public_visible" => _serde::__private::Ok(__Field::__field0),
                            "public_can_report" => {
                                _serde::__private::Ok(__Field::__field1)
                            }
                            "public_can_see_tickets" => {
                                _serde::__private::Ok(__Field::__field2)
                            }
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"public_visible" => _serde::__private::Ok(__Field::__field0),
                            b"public_can_report" => {
                                _serde::__private::Ok(__Field::__field1)
                            }
                            b"public_can_see_tickets" => {
                                _serde::__private::Ok(__Field::__field2)
                            }
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<VisibilityConfig>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = VisibilityConfig;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct VisibilityConfig",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            bool,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct VisibilityConfig with 3 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            bool,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct VisibilityConfig with 3 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            Vec<String>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct VisibilityConfig with 3 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private::Ok(VisibilityConfig {
                            public_visible: __field0,
                            public_can_report: __field1,
                            public_can_see_tickets: __field2,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<bool> = _serde::__private::None;
                        let mut __field1: _serde::__private::Option<bool> = _serde::__private::None;
                        let mut __field2: _serde::__private::Option<Vec<String>> = _serde::__private::None;
                        while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "public_visible",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<bool>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "public_can_report",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<bool>(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private::Option::is_some(&__field2) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "public_can_see_tickets",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Vec<String>,
                                        >(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("public_visible")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("public_can_report")?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private::Some(__field2) => __field2,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field(
                                    "public_can_see_tickets",
                                )?
                            }
                        };
                        _serde::__private::Ok(VisibilityConfig {
                            public_visible: __field0,
                            public_can_report: __field1,
                            public_can_see_tickets: __field2,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "public_visible",
                    "public_can_report",
                    "public_can_see_tickets",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "VisibilityConfig",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<VisibilityConfig>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for VisibilityConfig {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "VisibilityConfig",
                    false as usize + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "public_visible",
                    &self.public_visible,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "public_can_report",
                    &self.public_can_report,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "public_can_see_tickets",
                    &self.public_can_see_tickets,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for VisibilityConfig {
        #[inline]
        fn clone(&self) -> VisibilityConfig {
            VisibilityConfig {
                public_visible: ::core::clone::Clone::clone(&self.public_visible),
                public_can_report: ::core::clone::Clone::clone(&self.public_can_report),
                public_can_see_tickets: ::core::clone::Clone::clone(
                    &self.public_can_see_tickets,
                ),
            }
        }
    }
    pub struct ProjectLinks {
        pub github: String,
    }
    #[serde(rename_all = "camelCase")]
    pub struct ProjectLinksGitopsUpdate {
        pub github: Option<String>,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ProjectLinksGitopsUpdate {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "ProjectLinksGitopsUpdate",
                "github",
                &&self.github,
            )
        }
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for ProjectLinksGitopsUpdate {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "ProjectLinksGitopsUpdate",
                    false as usize + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "github",
                    &self.github,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for ProjectLinksGitopsUpdate {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "github" => _serde::__private::Ok(__Field::__field0),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"github" => _serde::__private::Ok(__Field::__field0),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<ProjectLinksGitopsUpdate>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = ProjectLinksGitopsUpdate;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct ProjectLinksGitopsUpdate",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            Option<String>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct ProjectLinksGitopsUpdate with 1 element",
                                    ),
                                );
                            }
                        };
                        _serde::__private::Ok(ProjectLinksGitopsUpdate {
                            github: __field0,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<Option<String>> = _serde::__private::None;
                        while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("github"),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Option<String>,
                                        >(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("github")?
                            }
                        };
                        _serde::__private::Ok(ProjectLinksGitopsUpdate {
                            github: __field0,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &["github"];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "ProjectLinksGitopsUpdate",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<
                            ProjectLinksGitopsUpdate,
                        >,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    impl gitops_lib::GitopsResourcePart for ProjectLinks {
        fn with_updates_from_part(self, updates: Self::UpdatePart) -> Self {
            let mut updated = self;
            if let Some(new_value) = updates.github {
                updated.github = new_value;
            }
            updated
        }
        fn as_update(&self) -> Self::UpdatePart {
            ProjectLinksGitopsUpdate {
                github: Some(self.github.clone()),
            }
        }
        type UpdatePart = ProjectLinksGitopsUpdate;
    }
    impl ProjectLinks {}
    #[automatically_derived]
    impl ::core::fmt::Debug for ProjectLinks {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "ProjectLinks",
                "github",
                &&self.github,
            )
        }
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for ProjectLinks {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "github" => _serde::__private::Ok(__Field::__field0),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"github" => _serde::__private::Ok(__Field::__field0),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<ProjectLinks>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = ProjectLinks;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct ProjectLinks",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct ProjectLinks with 1 element",
                                    ),
                                );
                            }
                        };
                        _serde::__private::Ok(ProjectLinks { github: __field0 })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                        while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("github"),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("github")?
                            }
                        };
                        _serde::__private::Ok(ProjectLinks { github: __field0 })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &["github"];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "ProjectLinks",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<ProjectLinks>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for ProjectLinks {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "ProjectLinks",
                    false as usize + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "github",
                    &self.github,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for ProjectLinks {
        #[inline]
        fn clone(&self) -> ProjectLinks {
            ProjectLinks {
                github: ::core::clone::Clone::clone(&self.github),
            }
        }
    }
    #[gitops(key = "name_id", owner = "owner_uid")]
    pub struct Project {
        pub name_id: String,
        pub public_name: String,
        pub owner_uid: String,
        pub admins_uid: Vec<String>,
        pub visibility: VisibilityConfig,
        pub links: ProjectLinks,
    }
    impl From<Project> for ProjectGitopsSerializable {
        fn from(resource: Project) -> Self {
            Self {
                kind: "Project".to_string(),
                api_version: "v1.0".to_string(),
                name_id: resource.name_id,
                public_name: resource.public_name,
                owner_uid: resource.owner_uid,
                admins_uid: resource.admins_uid,
                visibility: resource.visibility,
                links: resource.links,
            }
        }
    }
    impl From<ProjectGitopsSerializable> for Project {
        fn from(serializable_resource: ProjectGitopsSerializable) -> Self {
            Self {
                name_id: serializable_resource.name_id,
                public_name: serializable_resource.public_name,
                owner_uid: serializable_resource.owner_uid,
                admins_uid: serializable_resource.admins_uid,
                visibility: serializable_resource.visibility,
                links: serializable_resource.links,
            }
        }
    }
    #[serde(rename_all = "camelCase")]
    pub struct ProjectGitopsUpdate {
        pub name_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub public_name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub owner_uid: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub admins_uid: Option<Vec<String>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub visibility: Option<VisibilityConfigGitopsUpdate>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub links: Option<ProjectLinksGitopsUpdate>,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ProjectGitopsUpdate {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "name_id",
                "public_name",
                "owner_uid",
                "admins_uid",
                "visibility",
                "links",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.name_id,
                &self.public_name,
                &self.owner_uid,
                &self.admins_uid,
                &self.visibility,
                &&self.links,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "ProjectGitopsUpdate",
                names,
                values,
            )
        }
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for ProjectGitopsUpdate {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "ProjectGitopsUpdate",
                    false as usize + 1
                        + if Option::is_none(&self.public_name) { 0 } else { 1 }
                        + if Option::is_none(&self.owner_uid) { 0 } else { 1 }
                        + if Option::is_none(&self.admins_uid) { 0 } else { 1 }
                        + if Option::is_none(&self.visibility) { 0 } else { 1 }
                        + if Option::is_none(&self.links) { 0 } else { 1 },
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "nameId",
                    &self.name_id,
                )?;
                if !Option::is_none(&self.public_name) {
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "publicName",
                        &self.public_name,
                    )?;
                } else {
                    _serde::ser::SerializeStruct::skip_field(
                        &mut __serde_state,
                        "publicName",
                    )?;
                }
                if !Option::is_none(&self.owner_uid) {
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "ownerUid",
                        &self.owner_uid,
                    )?;
                } else {
                    _serde::ser::SerializeStruct::skip_field(
                        &mut __serde_state,
                        "ownerUid",
                    )?;
                }
                if !Option::is_none(&self.admins_uid) {
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "adminsUid",
                        &self.admins_uid,
                    )?;
                } else {
                    _serde::ser::SerializeStruct::skip_field(
                        &mut __serde_state,
                        "adminsUid",
                    )?;
                }
                if !Option::is_none(&self.visibility) {
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "visibility",
                        &self.visibility,
                    )?;
                } else {
                    _serde::ser::SerializeStruct::skip_field(
                        &mut __serde_state,
                        "visibility",
                    )?;
                }
                if !Option::is_none(&self.links) {
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "links",
                        &self.links,
                    )?;
                } else {
                    _serde::ser::SerializeStruct::skip_field(
                        &mut __serde_state,
                        "links",
                    )?;
                }
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for ProjectGitopsUpdate {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __field4,
                    __field5,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            3u64 => _serde::__private::Ok(__Field::__field3),
                            4u64 => _serde::__private::Ok(__Field::__field4),
                            5u64 => _serde::__private::Ok(__Field::__field5),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "nameId" => _serde::__private::Ok(__Field::__field0),
                            "publicName" => _serde::__private::Ok(__Field::__field1),
                            "ownerUid" => _serde::__private::Ok(__Field::__field2),
                            "adminsUid" => _serde::__private::Ok(__Field::__field3),
                            "visibility" => _serde::__private::Ok(__Field::__field4),
                            "links" => _serde::__private::Ok(__Field::__field5),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"nameId" => _serde::__private::Ok(__Field::__field0),
                            b"publicName" => _serde::__private::Ok(__Field::__field1),
                            b"ownerUid" => _serde::__private::Ok(__Field::__field2),
                            b"adminsUid" => _serde::__private::Ok(__Field::__field3),
                            b"visibility" => _serde::__private::Ok(__Field::__field4),
                            b"links" => _serde::__private::Ok(__Field::__field5),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<ProjectGitopsUpdate>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = ProjectGitopsUpdate;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct ProjectGitopsUpdate",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct ProjectGitopsUpdate with 6 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            Option<String>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                _serde::__private::Default::default()
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            Option<String>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                _serde::__private::Default::default()
                            }
                        };
                        let __field3 = match _serde::de::SeqAccess::next_element::<
                            Option<Vec<String>>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                _serde::__private::Default::default()
                            }
                        };
                        let __field4 = match _serde::de::SeqAccess::next_element::<
                            Option<VisibilityConfigGitopsUpdate>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                _serde::__private::Default::default()
                            }
                        };
                        let __field5 = match _serde::de::SeqAccess::next_element::<
                            Option<ProjectLinksGitopsUpdate>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                _serde::__private::Default::default()
                            }
                        };
                        _serde::__private::Ok(ProjectGitopsUpdate {
                            name_id: __field0,
                            public_name: __field1,
                            owner_uid: __field2,
                            admins_uid: __field3,
                            visibility: __field4,
                            links: __field5,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field1: _serde::__private::Option<Option<String>> = _serde::__private::None;
                        let mut __field2: _serde::__private::Option<Option<String>> = _serde::__private::None;
                        let mut __field3: _serde::__private::Option<
                            Option<Vec<String>>,
                        > = _serde::__private::None;
                        let mut __field4: _serde::__private::Option<
                            Option<VisibilityConfigGitopsUpdate>,
                        > = _serde::__private::None;
                        let mut __field5: _serde::__private::Option<
                            Option<ProjectLinksGitopsUpdate>,
                        > = _serde::__private::None;
                        while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("nameId"),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "publicName",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Option<String>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private::Option::is_some(&__field2) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "ownerUid",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Option<String>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private::Option::is_some(&__field3) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "adminsUid",
                                            ),
                                        );
                                    }
                                    __field3 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Option<Vec<String>>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field4 => {
                                    if _serde::__private::Option::is_some(&__field4) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "visibility",
                                            ),
                                        );
                                    }
                                    __field4 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Option<VisibilityConfigGitopsUpdate>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field5 => {
                                    if _serde::__private::Option::is_some(&__field5) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("links"),
                                        );
                                    }
                                    __field5 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Option<ProjectLinksGitopsUpdate>,
                                        >(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("nameId")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                _serde::__private::Default::default()
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private::Some(__field2) => __field2,
                            _serde::__private::None => {
                                _serde::__private::Default::default()
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private::Some(__field3) => __field3,
                            _serde::__private::None => {
                                _serde::__private::Default::default()
                            }
                        };
                        let __field4 = match __field4 {
                            _serde::__private::Some(__field4) => __field4,
                            _serde::__private::None => {
                                _serde::__private::Default::default()
                            }
                        };
                        let __field5 = match __field5 {
                            _serde::__private::Some(__field5) => __field5,
                            _serde::__private::None => {
                                _serde::__private::Default::default()
                            }
                        };
                        _serde::__private::Ok(ProjectGitopsUpdate {
                            name_id: __field0,
                            public_name: __field1,
                            owner_uid: __field2,
                            admins_uid: __field3,
                            visibility: __field4,
                            links: __field5,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "nameId",
                    "publicName",
                    "ownerUid",
                    "adminsUid",
                    "visibility",
                    "links",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "ProjectGitopsUpdate",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<ProjectGitopsUpdate>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    impl gitops_lib::GitopsResourceRoot for Project {
        type Serializable = ProjectGitopsSerializable;
        type Update = ProjectGitopsUpdate;
        fn as_serializable(&self) -> Self::Serializable {
            Self::Serializable {
                kind: "Project".to_string(),
                api_version: "v1.0".to_string(),
                name_id: self.name_id.clone(),
                public_name: self.public_name.clone(),
                owner_uid: self.owner_uid.clone(),
                admins_uid: self.admins_uid.clone(),
                visibility: self.visibility.clone(),
                links: self.links.clone(),
            }
        }
        fn into_serializable(self) -> Self::Serializable {
            Self::Serializable {
                kind: "Project".to_string(),
                api_version: "v1.0".to_string(),
                name_id: self.name_id,
                public_name: self.public_name,
                owner_uid: self.owner_uid,
                admins_uid: self.admins_uid,
                visibility: self.visibility,
                links: self.links,
            }
        }
        fn get_kind(&self) -> String {
            "Project".to_string()
        }
        fn get_key(&self) -> String {
            self.name_id.clone()
        }
        fn kind() -> &'static str {
            "Project"
        }
        fn with_updates_from(self, updates: Self::Update) -> Self {
            if self.name_id != updates.name_id {
                {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "Attempted to merge updates from an object with a different key. Current key: {0}, Update key: {1}",
                            self.name_id,
                            updates.name_id,
                        ),
                    );
                };
            }
            let mut updated = self;
            if let Some(new_value) = updates.public_name {
                updated.public_name = new_value;
            }
            if let Some(new_value) = updates.owner_uid {
                updated.owner_uid = new_value;
            }
            if let Some(new_value) = updates.admins_uid {
                updated.admins_uid = new_value;
            }
            if let Some(new_value) = updates.visibility {
                updated.visibility = updated
                    .visibility
                    .with_updates_from_part(new_value);
            }
            if let Some(new_value) = updates.links {
                updated.links = updated.links.with_updates_from_part(new_value);
            }
            updated
        }
    }
    #[serde(rename_all = "camelCase")]
    #[allow(non_snake_case)]
    pub struct ProjectGitopsSerializable {
        pub kind: String,
        #[serde(rename = "apiVersion")]
        pub api_version: String,
        pub name_id: String,
        pub public_name: String,
        pub owner_uid: String,
        pub admins_uid: Vec<String>,
        pub visibility: VisibilityConfig,
        pub links: ProjectLinks,
    }
    #[automatically_derived]
    #[allow(non_snake_case)]
    impl ::core::fmt::Debug for ProjectGitopsSerializable {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "kind",
                "api_version",
                "name_id",
                "public_name",
                "owner_uid",
                "admins_uid",
                "visibility",
                "links",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.kind,
                &self.api_version,
                &self.name_id,
                &self.public_name,
                &self.owner_uid,
                &self.admins_uid,
                &self.visibility,
                &&self.links,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "ProjectGitopsSerializable",
                names,
                values,
            )
        }
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for ProjectGitopsSerializable {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "ProjectGitopsSerializable",
                    false as usize + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "kind",
                    &self.kind,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "apiVersion",
                    &self.api_version,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "nameId",
                    &self.name_id,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "publicName",
                    &self.public_name,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "ownerUid",
                    &self.owner_uid,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "adminsUid",
                    &self.admins_uid,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "visibility",
                    &self.visibility,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "links",
                    &self.links,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for ProjectGitopsSerializable {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __field4,
                    __field5,
                    __field6,
                    __field7,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            3u64 => _serde::__private::Ok(__Field::__field3),
                            4u64 => _serde::__private::Ok(__Field::__field4),
                            5u64 => _serde::__private::Ok(__Field::__field5),
                            6u64 => _serde::__private::Ok(__Field::__field6),
                            7u64 => _serde::__private::Ok(__Field::__field7),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "kind" => _serde::__private::Ok(__Field::__field0),
                            "apiVersion" => _serde::__private::Ok(__Field::__field1),
                            "nameId" => _serde::__private::Ok(__Field::__field2),
                            "publicName" => _serde::__private::Ok(__Field::__field3),
                            "ownerUid" => _serde::__private::Ok(__Field::__field4),
                            "adminsUid" => _serde::__private::Ok(__Field::__field5),
                            "visibility" => _serde::__private::Ok(__Field::__field6),
                            "links" => _serde::__private::Ok(__Field::__field7),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"kind" => _serde::__private::Ok(__Field::__field0),
                            b"apiVersion" => _serde::__private::Ok(__Field::__field1),
                            b"nameId" => _serde::__private::Ok(__Field::__field2),
                            b"publicName" => _serde::__private::Ok(__Field::__field3),
                            b"ownerUid" => _serde::__private::Ok(__Field::__field4),
                            b"adminsUid" => _serde::__private::Ok(__Field::__field5),
                            b"visibility" => _serde::__private::Ok(__Field::__field6),
                            b"links" => _serde::__private::Ok(__Field::__field7),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<ProjectGitopsSerializable>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = ProjectGitopsSerializable;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct ProjectGitopsSerializable",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct ProjectGitopsSerializable with 8 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct ProjectGitopsSerializable with 8 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct ProjectGitopsSerializable with 8 elements",
                                    ),
                                );
                            }
                        };
                        let __field3 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        3usize,
                                        &"struct ProjectGitopsSerializable with 8 elements",
                                    ),
                                );
                            }
                        };
                        let __field4 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        4usize,
                                        &"struct ProjectGitopsSerializable with 8 elements",
                                    ),
                                );
                            }
                        };
                        let __field5 = match _serde::de::SeqAccess::next_element::<
                            Vec<String>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        5usize,
                                        &"struct ProjectGitopsSerializable with 8 elements",
                                    ),
                                );
                            }
                        };
                        let __field6 = match _serde::de::SeqAccess::next_element::<
                            VisibilityConfig,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        6usize,
                                        &"struct ProjectGitopsSerializable with 8 elements",
                                    ),
                                );
                            }
                        };
                        let __field7 = match _serde::de::SeqAccess::next_element::<
                            ProjectLinks,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        7usize,
                                        &"struct ProjectGitopsSerializable with 8 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private::Ok(ProjectGitopsSerializable {
                            kind: __field0,
                            api_version: __field1,
                            name_id: __field2,
                            public_name: __field3,
                            owner_uid: __field4,
                            admins_uid: __field5,
                            visibility: __field6,
                            links: __field7,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field1: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field2: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field3: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field4: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field5: _serde::__private::Option<Vec<String>> = _serde::__private::None;
                        let mut __field6: _serde::__private::Option<VisibilityConfig> = _serde::__private::None;
                        let mut __field7: _serde::__private::Option<ProjectLinks> = _serde::__private::None;
                        while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("kind"),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "apiVersion",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private::Option::is_some(&__field2) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("nameId"),
                                        );
                                    }
                                    __field2 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private::Option::is_some(&__field3) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "publicName",
                                            ),
                                        );
                                    }
                                    __field3 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field4 => {
                                    if _serde::__private::Option::is_some(&__field4) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "ownerUid",
                                            ),
                                        );
                                    }
                                    __field4 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field5 => {
                                    if _serde::__private::Option::is_some(&__field5) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "adminsUid",
                                            ),
                                        );
                                    }
                                    __field5 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Vec<String>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field6 => {
                                    if _serde::__private::Option::is_some(&__field6) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "visibility",
                                            ),
                                        );
                                    }
                                    __field6 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            VisibilityConfig,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field7 => {
                                    if _serde::__private::Option::is_some(&__field7) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("links"),
                                        );
                                    }
                                    __field7 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            ProjectLinks,
                                        >(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("kind")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("apiVersion")?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private::Some(__field2) => __field2,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("nameId")?
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private::Some(__field3) => __field3,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("publicName")?
                            }
                        };
                        let __field4 = match __field4 {
                            _serde::__private::Some(__field4) => __field4,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("ownerUid")?
                            }
                        };
                        let __field5 = match __field5 {
                            _serde::__private::Some(__field5) => __field5,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("adminsUid")?
                            }
                        };
                        let __field6 = match __field6 {
                            _serde::__private::Some(__field6) => __field6,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("visibility")?
                            }
                        };
                        let __field7 = match __field7 {
                            _serde::__private::Some(__field7) => __field7,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("links")?
                            }
                        };
                        _serde::__private::Ok(ProjectGitopsSerializable {
                            kind: __field0,
                            api_version: __field1,
                            name_id: __field2,
                            public_name: __field3,
                            owner_uid: __field4,
                            admins_uid: __field5,
                            visibility: __field6,
                            links: __field7,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "kind",
                    "apiVersion",
                    "nameId",
                    "publicName",
                    "ownerUid",
                    "adminsUid",
                    "visibility",
                    "links",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "ProjectGitopsSerializable",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<
                            ProjectGitopsSerializable,
                        >,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::fmt::Debug for Project {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "name_id",
                "public_name",
                "owner_uid",
                "admins_uid",
                "visibility",
                "links",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.name_id,
                &self.public_name,
                &self.owner_uid,
                &self.admins_uid,
                &self.visibility,
                &&self.links,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "Project",
                names,
                values,
            )
        }
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for Project {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "Project",
                    false as usize + 1 + 1 + 1 + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "name_id",
                    &self.name_id,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "public_name",
                    &self.public_name,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "owner_uid",
                    &self.owner_uid,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "admins_uid",
                    &self.admins_uid,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "visibility",
                    &self.visibility,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "links",
                    &self.links,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for Project {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                    __field4,
                    __field5,
                    __ignore,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            3u64 => _serde::__private::Ok(__Field::__field3),
                            4u64 => _serde::__private::Ok(__Field::__field4),
                            5u64 => _serde::__private::Ok(__Field::__field5),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "name_id" => _serde::__private::Ok(__Field::__field0),
                            "public_name" => _serde::__private::Ok(__Field::__field1),
                            "owner_uid" => _serde::__private::Ok(__Field::__field2),
                            "admins_uid" => _serde::__private::Ok(__Field::__field3),
                            "visibility" => _serde::__private::Ok(__Field::__field4),
                            "links" => _serde::__private::Ok(__Field::__field5),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"name_id" => _serde::__private::Ok(__Field::__field0),
                            b"public_name" => _serde::__private::Ok(__Field::__field1),
                            b"owner_uid" => _serde::__private::Ok(__Field::__field2),
                            b"admins_uid" => _serde::__private::Ok(__Field::__field3),
                            b"visibility" => _serde::__private::Ok(__Field::__field4),
                            b"links" => _serde::__private::Ok(__Field::__field5),
                            _ => _serde::__private::Ok(__Field::__ignore),
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<Project>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = Project;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct Project",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct Project with 6 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct Project with 6 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            String,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct Project with 6 elements",
                                    ),
                                );
                            }
                        };
                        let __field3 = match _serde::de::SeqAccess::next_element::<
                            Vec<String>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        3usize,
                                        &"struct Project with 6 elements",
                                    ),
                                );
                            }
                        };
                        let __field4 = match _serde::de::SeqAccess::next_element::<
                            VisibilityConfig,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        4usize,
                                        &"struct Project with 6 elements",
                                    ),
                                );
                            }
                        };
                        let __field5 = match _serde::de::SeqAccess::next_element::<
                            ProjectLinks,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        5usize,
                                        &"struct Project with 6 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private::Ok(Project {
                            name_id: __field0,
                            public_name: __field1,
                            owner_uid: __field2,
                            admins_uid: __field3,
                            visibility: __field4,
                            links: __field5,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field1: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field2: _serde::__private::Option<String> = _serde::__private::None;
                        let mut __field3: _serde::__private::Option<Vec<String>> = _serde::__private::None;
                        let mut __field4: _serde::__private::Option<VisibilityConfig> = _serde::__private::None;
                        let mut __field5: _serde::__private::Option<ProjectLinks> = _serde::__private::None;
                        while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "name_id",
                                            ),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "public_name",
                                            ),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private::Option::is_some(&__field2) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "owner_uid",
                                            ),
                                        );
                                    }
                                    __field2 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<String>(&mut __map)?,
                                    );
                                }
                                __Field::__field3 => {
                                    if _serde::__private::Option::is_some(&__field3) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "admins_uid",
                                            ),
                                        );
                                    }
                                    __field3 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Vec<String>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field4 => {
                                    if _serde::__private::Option::is_some(&__field4) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field(
                                                "visibility",
                                            ),
                                        );
                                    }
                                    __field4 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            VisibilityConfig,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field5 => {
                                    if _serde::__private::Option::is_some(&__field5) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("links"),
                                        );
                                    }
                                    __field5 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            ProjectLinks,
                                        >(&mut __map)?,
                                    );
                                }
                                _ => {
                                    let _ = _serde::de::MapAccess::next_value::<
                                        _serde::de::IgnoredAny,
                                    >(&mut __map)?;
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("name_id")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("public_name")?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private::Some(__field2) => __field2,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("owner_uid")?
                            }
                        };
                        let __field3 = match __field3 {
                            _serde::__private::Some(__field3) => __field3,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("admins_uid")?
                            }
                        };
                        let __field4 = match __field4 {
                            _serde::__private::Some(__field4) => __field4,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("visibility")?
                            }
                        };
                        let __field5 = match __field5 {
                            _serde::__private::Some(__field5) => __field5,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("links")?
                            }
                        };
                        _serde::__private::Ok(Project {
                            name_id: __field0,
                            public_name: __field1,
                            owner_uid: __field2,
                            admins_uid: __field3,
                            visibility: __field4,
                            links: __field5,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &[
                    "name_id",
                    "public_name",
                    "owner_uid",
                    "admins_uid",
                    "visibility",
                    "links",
                ];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "Project",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<Project>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for Project {
        #[inline]
        fn clone(&self) -> Project {
            Project {
                name_id: ::core::clone::Clone::clone(&self.name_id),
                public_name: ::core::clone::Clone::clone(&self.public_name),
                owner_uid: ::core::clone::Clone::clone(&self.owner_uid),
                admins_uid: ::core::clone::Clone::clone(&self.admins_uid),
                visibility: ::core::clone::Clone::clone(&self.visibility),
                links: ::core::clone::Clone::clone(&self.links),
            }
        }
    }
    pub enum UserStatus {
        Fired,
        Replaced,
        Normal,
    }
    impl gitops_lib::GitopsResourcePart for UserStatus {
        type UpdatePart = Self;
        fn with_updates_from_part(self, updates: Self::UpdatePart) -> Self {
            updates
        }
        fn as_update(&self) -> Self::UpdatePart {
            self.clone()
        }
    }
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for UserStatus {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    UserStatus::Fired => {
                        _serde::Serializer::serialize_unit_variant(
                            __serializer,
                            "UserStatus",
                            0u32,
                            "Fired",
                        )
                    }
                    UserStatus::Replaced => {
                        _serde::Serializer::serialize_unit_variant(
                            __serializer,
                            "UserStatus",
                            1u32,
                            "Replaced",
                        )
                    }
                    UserStatus::Normal => {
                        _serde::Serializer::serialize_unit_variant(
                            __serializer,
                            "UserStatus",
                            2u32,
                            "Normal",
                        )
                    }
                }
            }
        }
    };
    #[doc(hidden)]
    #[allow(
        non_upper_case_globals,
        unused_attributes,
        unused_qualifications,
        clippy::absolute_paths,
    )]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for UserStatus {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "variant identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            _ => {
                                _serde::__private::Err(
                                    _serde::de::Error::invalid_value(
                                        _serde::de::Unexpected::Unsigned(__value),
                                        &"variant index 0 <= i < 3",
                                    ),
                                )
                            }
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "Fired" => _serde::__private::Ok(__Field::__field0),
                            "Replaced" => _serde::__private::Ok(__Field::__field1),
                            "Normal" => _serde::__private::Ok(__Field::__field2),
                            _ => {
                                _serde::__private::Err(
                                    _serde::de::Error::unknown_variant(__value, VARIANTS),
                                )
                            }
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"Fired" => _serde::__private::Ok(__Field::__field0),
                            b"Replaced" => _serde::__private::Ok(__Field::__field1),
                            b"Normal" => _serde::__private::Ok(__Field::__field2),
                            _ => {
                                let __value = &_serde::__private::from_utf8_lossy(__value);
                                _serde::__private::Err(
                                    _serde::de::Error::unknown_variant(__value, VARIANTS),
                                )
                            }
                        }
                    }
                }
                #[automatically_derived]
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<UserStatus>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                #[automatically_derived]
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = UserStatus;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "enum UserStatus",
                        )
                    }
                    fn visit_enum<__A>(
                        self,
                        __data: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::EnumAccess<'de>,
                    {
                        match _serde::de::EnumAccess::variant(__data)? {
                            (__Field::__field0, __variant) => {
                                _serde::de::VariantAccess::unit_variant(__variant)?;
                                _serde::__private::Ok(UserStatus::Fired)
                            }
                            (__Field::__field1, __variant) => {
                                _serde::de::VariantAccess::unit_variant(__variant)?;
                                _serde::__private::Ok(UserStatus::Replaced)
                            }
                            (__Field::__field2, __variant) => {
                                _serde::de::VariantAccess::unit_variant(__variant)?;
                                _serde::__private::Ok(UserStatus::Normal)
                            }
                        }
                    }
                }
                #[doc(hidden)]
                const VARIANTS: &'static [&'static str] = &[
                    "Fired",
                    "Replaced",
                    "Normal",
                ];
                _serde::Deserializer::deserialize_enum(
                    __deserializer,
                    "UserStatus",
                    VARIANTS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<UserStatus>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    #[automatically_derived]
    impl ::core::clone::Clone for UserStatus {
        #[inline]
        fn clone(&self) -> UserStatus {
            match self {
                UserStatus::Fired => UserStatus::Fired,
                UserStatus::Replaced => UserStatus::Replaced,
                UserStatus::Normal => UserStatus::Normal,
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UserStatus {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    UserStatus::Fired => "Fired",
                    UserStatus::Replaced => "Replaced",
                    UserStatus::Normal => "Normal",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for UserStatus {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for UserStatus {
        #[inline]
        fn eq(&self, other: &UserStatus) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
}
