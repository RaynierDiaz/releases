use reqwest::blocking::Client;
use anyhow::*;
use smart_read::prelude::*;
use zip::ZipArchive;
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;
use crate::*;



pub fn install() {
	
	let revit_dir = prompt!("Where is Revit located? "; ["C:\\ProgramData\\Autodesk\\Revit"] SimpleValidate (|input| {
		println!("Testing: '{input}'");
		if PathBuf::from(input).exists() {
			StdResult::Ok(())
		} else {
			StdResult::Err(String::from("That path does not exist"))
		}
	}));
	let revit_dir = PathBuf::from(revit_dir);
	let ext_dir = prompt!("Where would you like to install the extension? "; ["C:\\ProgramData\\TupeloWorkbenchExt"] SimpleValidate (|input| {
		if PathBuf::from(input).to_str().is_some() {
			StdResult::Ok(())
		} else {
			StdResult::Err(String::from("Unable to convert path back to string"))
		}
	}));
	let ext_dir = PathBuf::from(ext_dir);
	
	println!("Downloading assets...");
	let assets = match download_assets() {
		StdResult::Ok(v) => v,
		StdResult::Err(err) => {
			println!("Failed to download assets from GitHub. Please contact Tupelo Workbench with this error message: {err:?}");
			return;
		}
	};
	println!("Done.");
	
	println!("Installing...");
	
	let zip_cursor = Cursor::new(assets);
	let mut zip_data = match ZipArchive::new(zip_cursor) {
		StdResult::Ok(v) => v,
		StdResult::Err(err) => {
			println!("Failed to parse downloaded assets. Please contact Tupelo Workbench with this error message: {err:?}");
			return;
		}
	};
	
	let version = match get_version(&mut zip_data) {
		StdResult::Ok(v) => v,
		StdResult::Err(err) => {
			println!("Failed to retrieve assets data, attempting to continue...  (error message: {err:?})");
			settings::ASSETS_VERSION
		}
	};
	if version != settings::ASSETS_VERSION {
		println!("Installer is out of date, please re-download installer to continue\nNOTE: we suggest uninstalling the extension before updating the installer");
		return;
	}
	
	if let Err(err) = write_dll_files(&mut zip_data, &ext_dir) {
		println!("Failed to create dll files. Please contact Tupelo Workbench with this error message: {err:?}");
	}
	if let Err(err) = write_addin_files(&mut zip_data, &revit_dir, &ext_dir) {
		println!("Failed to create addin files. Please contact Tupelo Workbench with this error message: {err:?}");
	}
	
	println!("Done.");
	
	prompt!("Install successful, press enter to close the installer");
	
}



pub fn download_assets() -> Result<Vec<u8>> {
	
	// get release data
	let api_url = format!("https://api.github.com/repos/{}/{}/releases/latest", settings::REPO_OWNER, settings::REPO_NAME);
	let client = Client::new();
	let response = client
		.get(api_url)
		.header("User-Agent", "Mozilla/5.0")
		// .header("Accept", "application/vnd.github+json")
		.header("X-GitHub-Api-Version", "2022-11-28")
		.send().context("Attempted to send api request")?
		.text().context("Attempted to retrieve page text")?;
	let response: Release = serde_json::from_str(&response).context("Attempted to parse page json")?;
	
	// get assets data
	let asset = response
		.assets
		.iter()
		.find(|a| a.name == settings::ASSETS_NAME)
		.ok_or(Error::msg("Asset not found in the latest release"))?;
	
	// download assets
	let mut response = client.get(&asset.browser_download_url).send().context("Attempted to send download request")?;
	if response.status() != 200 {
		return Err(Error::msg(format!("Download of assets returned with status code {}.", response.status())));
	}
	
	// extract raw data
	let len = response.content_length().ok_or_else(|| 
		Error::msg("Could not get length of received assets data.")
	)?;
	let mut buffer = Vec::with_capacity(len as usize);
	response.read_to_end(&mut buffer).context("Attempted to read asset data")?;
	
	Ok(buffer)
}



pub fn get_version(zip_data: &mut ZipArchive<Cursor<Vec<u8>>>) -> Result<usize> {
	let file_contents = get_file_text(zip_data, "README.md")?;
	let first_line = file_contents.lines().next().ok_or_else(|| Error::msg("Could not prase the first line of README.md in assets."))?;
	if !first_line.starts_with("#### VERSION_") {return Err(Error::msg("Could not prase the first line of README.md in assets."));}
	let first_line = &first_line[13..];
	first_line.parse::<usize>().map_err(Error::from)
}



pub fn get_file_text(zip_data: &mut ZipArchive<Cursor<Vec<u8>>>, file_name: &'static str) -> Result<String> {
	let mut zip_file = zip_data.by_name(file_name).context(format!("Attempted to find file {file_name:?}"))?;
	let mut contents = String::with_capacity(zip_file.size() as usize);
	zip_file.read_to_string(&mut contents).context(format!("Attempted to read asset file {file_name:?}"))?;
	Ok(contents)
}

pub fn get_file_bytes(zip_data: &mut ZipArchive<Cursor<Vec<u8>>>, file_name: &'static str) -> Result<Vec<u8>> {
	let mut zip_file = zip_data.by_name(file_name).context(format!("Attempted to find file {file_name:?}"))?;
	let mut contents = Vec::with_capacity(zip_file.size() as usize);
	zip_file.read_to_end(&mut contents).context(format!("Attempted to read asset file {file_name:?}"))?;
	Ok(contents)
}



pub fn write_dll_files(zip_data: &mut ZipArchive<Cursor<Vec<u8>>>, ext_dir: &Path) -> Result<()> {
	
	fs::create_dir_all(ext_dir).context(format!("Attempted to create folders at {ext_dir:?}"))?;
	let frontend_dll_data = get_file_bytes(zip_data, "Frontend.dll")?;
	let backend_dll_data = get_file_bytes(zip_data, "Backend.dll")?;
	
	let frontend_dll_path = ext_dir.join("Frontend.dll");
	fs::write(&frontend_dll_path, frontend_dll_data).context(format!("Attempted to write file {frontend_dll_path:?}"))?;
	let backend_dll_path = ext_dir.join("Backend.dll");
	fs::write(&backend_dll_path, backend_dll_data).context(format!("Attempted to write file {backend_dll_path:?}"))?;
	
	Ok(())
}



pub fn write_addin_files(zip_data: &mut ZipArchive<Cursor<Vec<u8>>>, revit_dir: &Path, ext_dir: &Path) -> Result<()> {
	let addins_folder = revit_dir.join("Addins");
	let addin_file_contents = get_file_text(zip_data, "TupeloWorkbench.addin")?;
	let addin_file_contents = addin_file_contents.replace("EXTENSION_DIR", ext_dir.to_str().unwrap());
	
	for child in fs::read_dir(&addins_folder)? {
		let StdResult::Ok(child) = child else {
			println!("Warning: failed to read a child of {addins_folder:?}");
			continue;
		};
		fs::write(child.path().join("TupeloWorkbench.addin"), &addin_file_contents)?;
	}
	
	Ok(())
}
