/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`Query`] that contains everything needed to check if an HTML tag fits all the provided requirements
//! and [`QueryData`] that has everything needed to traverse an entire HTML document in search for a tag,
//! as well as a way to parse the data contained in it

use crate::action::regex::{action::Replace, Regex};

/// The type of item that should be queried
#[derive(Clone, Debug)]
pub enum QueryKind {
	/// An HTML tag
	Tag(String),
	/// An HTML class
	Class(String),
	/// An HTML attribute
	Attr {
		/// Name of the attr
		name: String,
		/// Value of the attr
		value: String,
	},
}

/// The location of the data in the quiried tag
#[derive(Debug)]
pub enum DataLocation {
	/// In the text part of the tag
	Text,
	/// In an attribute
	Attr(String),
}

/// A query for an HTML tag
#[derive(Debug)]
pub struct Query {
	/// Query the tag should match against
	pub kind: QueryKind,
	/// Query the tag should never match
	pub ignore: Option<Vec<QueryKind>>,
}

/// A query for a complete HTML tag. Traverses all queries one by one and extracts the data from it's [`DataLocation`], optionally transforming the data via regex
/// Example:
/// ```text
/// QueryData {
///     query: [Tag("div"), Attr { name: "id", value: "this-attr" }],
///     data_location: text,
///     regex: { re: ".*", replace_with: "hello, ${1}!"
/// }
/// ```
/// will match
/// ```text
/// <div>
///     <b id="this-attr">
///         world
///     </b>
/// </div>
/// ```
/// and return "hello, world!"
#[derive(Debug)]
pub struct QueryData {
	/// The queries to match against, one by one
	pub query: Vec<Query>,
	/// location of the data to extract
	pub data_location: DataLocation,
	/// optional regex to match against and replace with if it matches
	pub regex: Option<Regex<Replace>>,
}

// TODO: make query data optional instead
#[allow(missing_docs)]
#[derive(Debug)]
pub struct ImageQuery {
	pub inner: QueryData,
	pub optional: bool,
}