/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod discord;
pub mod email_password;
pub mod google_oauth2;
pub mod runtime_external_save;
pub mod telegram;

use super::proj_dirs;

use color_eyre::Result;
use std::{
	io::{self, Write},
	path::PathBuf,
};

pub fn prompt_user_for(prompt: &str) -> io::Result<String> {
	print!("{prompt}");
	io::stdout().flush()?;

	let mut input = String::new();
	io::stdin().read_line(&mut input)?;

	Ok(input.trim().to_owned())
}

pub fn default_data_path() -> Result<PathBuf> {
	#[cfg(target_os = "linux")]
	{
		if nix::unistd::Uid::effective().is_root() {
			return Ok(PathBuf::from("/var/lib/fetcher"));
		}
	}

	Ok(proj_dirs()?.data_dir().to_path_buf())
}
