pub mod settings {
	pub const REPO_OWNER: &str = "RaynierDiaz";
	pub const REPO_NAME: &str = "releases";
	pub const ASSETS_NAME: &str = "Assets.zip";
	pub const INSTALLER_NAME: &str = "Installer.exe";
	pub const LATEST_ASSETS_VERSION: usize = 3;
}



use serde::Deserialize;
use smart_read::prelude::*;



pub mod install;
pub mod self_update;
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
	
	let mut args = std::env::args();
	args.next();
	let first_arg = args.next();
	
	if first_arg.as_deref() == Some("--self-update") {
		self_update::self_update();
		return;
	}
	
	let options = &[
		InputOption::new("install / update (uses latest version)", vec!("1"), 1),
		InputOption::new("offline install", vec!("2"), 2),
		InputOption::new("uninstall", vec!("3"), 3),
	];
	match prompt!("What would you like to do? "; options).1.data {
		1 => {let _ = install::install(false, None, false);},
		2 => {let _ = install::install(true, None, false);},
		3 => {let _ = uninstall::uninstall(false);},
		_ => unreachable!(),
	}
	
}
