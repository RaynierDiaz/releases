pub mod settings {
	pub const REPO_OWNER: &str = "RaynierDiaz";
	pub const REPO_NAME: &str = "releases";
	pub const ASSETS_NAME: &str = "Assets.zip";
	pub const ASSETS_VERSION: usize = 1;
}



use serde::Deserialize;
use smart_read::prelude::*;



pub mod install;
pub mod uninstall;



#[derive(Deserialize, Debug)]
struct Asset {
	name: String,
	browser_download_url: String,
}

#[derive(Deserialize)]
struct Release {
	assets: Vec<Asset>,
}



fn main() {
	
	let options = &[
		OptionWithData {display_name: String::from("1: install / update"), data: 1usize},
		OptionWithData {display_name: String::from("2: uninstall"), data: 2usize},
	];
	match prompt!("What would you like to do? "; options).data {
		1 => install::install(),
		2 => uninstall::uninstall(),
		_ => unreachable!(),
	}
	
}
