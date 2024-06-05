pub mod settings {
	pub const REPO_OWNER: &str = "RaynierDiaz";
	pub const REPO_NAME: &str = "releases";
	pub const ASSET_NAME: &str = "TupeloWorkbenchExt.dll";
}



use reqwest::blocking::Client;
use serde::Deserialize;
use std::fs::File;
use std::io::copy;
use anyhow::*;



#[derive(Deserialize, Debug)]
struct Asset {
	name: String,
	browser_download_url: String,
}

#[derive(Deserialize)]
struct Release {
	assets: Vec<Asset>,
}



fn main() -> Result<()> {
	const DEST_PATH: &str = "C:/ProgramData/TupeloWorkbenchExt.dll";
	
	let api_url = format!("https://api.github.com/repos/{}/{}/releases/latest", settings::REPO_OWNER, settings::REPO_NAME);
	println!("{api_url}");
	let client = Client::new();
	let response = client
		.get(&api_url)
		.header("User-Agent", "Mozilla/5.0")
		.header("Accept", "application/vnd.github+json")
		.header("X-GitHub-Api-Version", "2022-11-28")
		.send()?;
	println!("{}", response.status());
	let response = response.text()?;
	println!("{response}");
	let response: Release = serde_json::from_str(&response)?;
	println!("{:#?}", &response.assets);
	
	let asset = response
		.assets
		.iter()
		.find(|a| a.name == settings::ASSET_NAME)
		.ok_or(Error::msg("Asset not found in the latest release"))?;
	
	let mut resp = client.get(&asset.browser_download_url).send()?;
	let mut out = File::create(DEST_PATH)?;
	copy(&mut resp, &mut out)?;
	
	println!("Download complete.");
	Ok(())
}
