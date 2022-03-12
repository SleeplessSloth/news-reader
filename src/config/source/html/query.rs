use serde::Deserialize;

use crate::source;

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum QueryKind {
	Tag { value: String },
	Class { value: String },
	Attr { name: String, value: String },
}

impl QueryKind {
	pub(crate) fn parse(self) -> source::html::query::QueryKind {
		use QueryKind::{Attr, Class, Tag};

		match self {
			Tag { value } => source::html::query::QueryKind::Tag { value },
			Class { value } => source::html::query::QueryKind::Class { value },
			Attr { name, value } => source::html::query::QueryKind::Attr { name, value },
		}
	}
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum DataLocation {
	Text,
	Attr { value: String },
}

impl DataLocation {
	fn parse(self) -> source::html::query::DataLocation {
		use DataLocation::{Attr, Text};

		match self {
			Text => source::html::query::DataLocation::Text,
			Attr { value } => source::html::query::DataLocation::Attr { value },
		}
	}
}

#[derive(Deserialize, Debug)]
pub(crate) struct Query {
	kind: Vec<QueryKind>,
	data_location: DataLocation,
}

impl Query {
	fn parse(self) -> source::html::query::Query {
		source::html::query::Query {
			kind: self.kind.into_iter().map(QueryKind::parse).collect(),
			data_location: self.data_location.parse(),
		}
	}
}

#[derive(Deserialize, Debug)]
pub(crate) struct TextQuery {
	prepend: Option<String>,
	#[serde(flatten)]
	inner: Query,
}

impl TextQuery {
	pub(crate) fn parse(self) -> source::html::query::TextQuery {
		source::html::query::TextQuery {
			prepend: self.prepend,
			inner: self.inner.parse(),
		}
	}
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum IdQueryKind {
	String,
	Date,
}

impl IdQueryKind {
	fn parse(self) -> source::html::query::IdQueryKind {
		match self {
			IdQueryKind::String => source::html::query::IdQueryKind::String,
			IdQueryKind::Date => source::html::query::IdQueryKind::Date,
		}
	}
}

#[derive(Deserialize, Debug)]
pub(crate) struct IdQuery {
	pub(crate) kind: IdQueryKind,
	#[serde(rename = "query")]
	pub(crate) inner: Query,
}

impl IdQuery {
	pub(crate) fn parse(self) -> source::html::query::IdQuery {
		source::html::query::IdQuery {
			kind: self.kind.parse(),
			inner: self.inner.parse(),
		}
	}
}

#[derive(Deserialize, Debug)]
pub(crate) struct LinkQuery {
	prepend: Option<String>,
	#[serde(flatten)]
	inner: Query,
}

impl LinkQuery {
	pub(crate) fn parse(self) -> source::html::query::LinkQuery {
		source::html::query::LinkQuery {
			prepend: self.prepend,
			inner: self.inner.parse(),
		}
	}
}

#[derive(Deserialize, Debug)]
pub(crate) struct ImageQuery {
	optional: Option<bool>,
	#[serde(flatten)]
	inner: LinkQuery,
}

impl ImageQuery {
	pub(crate) fn parse(self) -> source::html::query::ImageQuery {
		source::html::query::ImageQuery {
			optional: self.optional.unwrap_or(false),
			inner: self.inner.parse(),
		}
	}
}
