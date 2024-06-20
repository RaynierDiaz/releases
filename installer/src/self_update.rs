use crate::{settings, Release};
use anyhow::*;
use reqwest::blocking::Client;
use std::{fs, io::Read, process::Command, result::Result as StdResult};
use smart_read::prelude::*;



pub fn self_update() {
	
	// uninstall
	let uninstall_succeeded  = crate::uninstall::uninstall(true);
	if !uninstall_succeeded {return;}
	
	println!("Downloading new installer...");
	
	// get new installer
	let new_installer_data = match download_new_installer() {
		StdResult::Ok(v) => v,
		StdResult::Err(err) => {
			prompt!(format!("Failed to download new installer. Please contact Tupelo Workbench with this error message: {err:?} "));
			return;
		}
	};
	
	// save new installer
	let mut new_installer_path = std::env::current_exe().expect("Attempted to get path of current exe");
	new_installer_path.pop();
	new_installer_path.push("new_installer.exe");
	if let Err(err) = fs::write(&new_installer_path, new_installer_data) {
		prompt!(format!("Failed to create file for new installer. Please contact Tupelo Workbench with this error message: {err:?} "));
		return;
	};
	
	println!("Done. Running new installer...");
	
	let child = Command::new("cmd")
        .arg("/C")
        .arg("start")
        .arg("powershell")
        .arg("-Command")
        .arg(format!("& '{}' --auto-install", new_installer_path.to_str().expect("Attempted to parse path into string")))
        .spawn();
	if let Err(err) = child {
		prompt!(format!("Failed to run new installer. Please contact Tupelo Workbench with this error message: {err:?} "));
		return;
	}
	
	// delete self
	if let Err(err) = self_replace::self_delete() {
		prompt!(format!("Failed to delete this installer. Please contact Tupelo Workbench with this error message: {err:?} "));
		return;
	}
	
	println!("Auto uninstall complete. The latest installer should be running, this installer should be deleted soon, and this window should close before you even see this message.");
	
}



pub fn download_new_installer() -> Result<Vec<u8>> {
	// get release data
	let api_url = format!("https://api.github.com/repos/{}/{}/releases/latest", settings::REPO_OWNER, settings::REPO_NAME);
	let client = Client::new();
	let response = client
		.get(api_url)
		.header("User-Agent", "Mozilla/5.0")
		.header("X-GitHub-Api-Version", "2022-11-28")
		.send().context("Attempted to send api request")?
		.text().context("Attempted to retrieve page text")?;
	let response: Release = serde_json::from_str(&response).context("Attempted to parse page json")?;
	
	// get installer data
	let installer_asset = response
		.assets
		.iter()
		.find(|a| a.name == settings::INSTALLER_NAME)
		.ok_or(Error::msg("Installer not found in the latest release"))?;
	
	// download installer
	let mut response = client.get(&installer_asset.browser_download_url).send().context("Attempted to send download request")?;
	if response.status() != 200 {
		return Err(Error::msg(format!("Download of installer returned error code {}", response.status())))
	}
	
	// extract raw data
	let Some(len) = response.content_length() else {
		return Err(Error::msg("Could not get length of received installer data."));
	};
	let mut buffer = Vec::with_capacity(len as usize);
	response.read_to_end(&mut buffer).context("Attempted to read installer data")?;
	
	Ok(buffer)
}
