pub mod settings {
	pub const REPO_OWNER: &str = "RaynierDiaz";
	pub const REPO_NAME: &str = "releases";
	pub const ASSETS_NAME: &str = "Assets.zip";
	pub const LATEST_ASSETS_VERSION: usize = 2;
}



use serde::Deserialize;
use smart_read::prelude::*;



pub mod install;
pub mod uninstall;
pub mod uninstallers;



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
		InputOption::new("install / update", vec!("1"), 1),
		InputOption::new("uninstall", vec!("2"), 2),
	];
	match prompt!("What would you like to do? "; options).1.data {
		1 => install::install(),
		2 => uninstall::uninstall(),
		_ => unreachable!(),
	}
	
}
