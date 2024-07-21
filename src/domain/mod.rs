// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).
pub mod github;
pub mod rfcbot;
pub mod schema;

fn unsigned<S>(v: &i32, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::ser::Serializer,
{
    s.serialize_u32(*v as u32)
}
