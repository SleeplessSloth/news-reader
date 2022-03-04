/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use std::collections::HashMap;

use crate::{sink::Sink, source::Source};

pub type Tasks = HashMap<String, Task>;

#[derive(Debug)]
pub struct Task {
	pub disabled: Option<bool>,
	pub sink: Sink,
	pub source: Source,
	pub refresh: u64,
}
