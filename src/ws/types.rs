use beef::lean::Cow;
use serde::de::{self, Deserializer, Unexpected, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::fmt;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct JsonRpcRequest<'a> {
	pub jsonrpc: TwoPointZero,

	#[serde(borrow)]
	pub id: Option<&'a RawValue>,

	#[serde(borrow)]
	pub method: Cow<'a, str>,

	#[serde(borrow)]
	pub params: &'a RawValue,
}

#[derive(Serialize, Debug)]
pub struct JsonRpcResponse<'a, T> {
	pub jsonrpc: TwoPointZero,
	pub id: Option<&'a RawValue>,
	pub result: T,
}

#[derive(Debug, Default)]
pub struct TwoPointZero;

struct TwoPointZeroVisitor;

impl<'de> Visitor<'de> for TwoPointZeroVisitor {
	type Value = TwoPointZero;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str(r#"a string "2.0""#)
	}

	fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
	where
		E: de::Error,
	{
		match s {
			"2.0" => Ok(TwoPointZero),
			_ => Err(de::Error::invalid_value(Unexpected::Str(s), &self)),
		}
	}
}

impl<'de> Deserialize<'de> for TwoPointZero {
	fn deserialize<D>(deserializer: D) -> Result<TwoPointZero, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_str(TwoPointZeroVisitor)
	}
}

impl Serialize for TwoPointZero {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str("2.0")
	}
}
