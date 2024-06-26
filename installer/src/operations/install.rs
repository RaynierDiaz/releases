use crate::prelude::*;
use reqwest::blocking::Client;
use zip::ZipArchive;
use std::io::{Cursor, Read};
use smart_read::prelude::*;



pub type InstallSucceeded = bool;

pub fn install(inner: Arc<Mutex<InnerApp>>, is_offline: bool, revit_path: Option<PathBuf>, is_self_update: bool) -> Result<InstallSucceeded> {
	
	const DEFAULT_REVIT_PATH: &str = "C:\\ProgramData\\Autodesk\\Revit";
	let revit_path = revit_path.unwrap_or_else(|| PathBuf::from(DEFAULT_REVIT_PATH));
	let revit_path = if revit_path.exists() {revit_path} else {
		
		todo!()
	};
	
	// check if already installed
	match check_already_installed(&revit_path) {
		StdResult::Ok(true) => {
			prompt!("Error: extension is already installed. Please uninstall before attempting to install / update. ");
			return Ok(false);
		}
		StdResult::Ok(false) => {}
		StdResult::Err(err) => {
			let should_continue = prompt!(format!("Warning: failed to check if extension is already installed (error: {err:?}), do you want to continue with installation? "); YesNoInput);
			if !should_continue {
				prompt!("Affirmed, canceling install. ");
				return Ok(false);
			}
		}
	}
	
	// get / download assets
	#[allow(unused_assignments)]
	let mut assets_owned = vec!();
	let assets = if is_offline {
		const ASSETS_DATA: &[u8] = include_bytes!("../Assets.zip");
		ASSETS_DATA
	} else {
		println!("Downloading assets...");
		let assets = match download_assets() {
			StdResult::Ok(v) => v,
			StdResult::Err(err) => {
				prompt!(format!("Failed to download assets from GitHub. Please contact Tupelo Workbench with this error message: {err:?} "));
				return Ok(false);
			}
		};
		println!("Done.");
		assets_owned = assets;
		&assets_owned
	};
	
	println!("Installing...");
	
	let zip_cursor = Cursor::new(assets);
	let mut zip_data = match ZipArchive::new(zip_cursor) {
		StdResult::Ok(v) => v,
		StdResult::Err(err) => {
			prompt!(format!("Failed to parse downloaded assets. Please contact Tupelo Workbench with this error message: {err:?} "));
			return Ok(false);
		}
	};
	
	let version = match get_format_version(&mut zip_data) {
		StdResult::Ok(v) => v,
		StdResult::Err(err) => {
			prompt!(format!("Failed to retrieve assets version, attempting to continue...  (error message: {err:?}) "));
			settings::LATEST_ASSETS_VERSION
		}
	};
	if version != settings::LATEST_ASSETS_VERSION {
		prompt!(format!("Installer is out of date, please re-download installer to continue. If this is the latest version, please submit a bug report (https://github.com/{}/{}/issues). ", settings::REPO_OWNER, settings::REPO_NAME));
		return Ok(false);
	}
	
	if let Err(err) = write_files(&mut zip_data, &revit_path) {
		prompt!(format!("Failed to write extension files. Please contact Tupelo Workbench with this error message: {err:?} "));
		return Ok(false);
	}
	
	println!("Done.");
	
	if !is_self_update {
		prompt!("Install successful, press enter to close the installer");
	}
	
	Ok(true)
}



pub fn check_already_installed(revit_path: &Path) -> Result<bool> {
	let addins_folder = revit_path.join("Addins");
	for child in fs::read_dir(&addins_folder)? {
		let StdResult::Ok(child) = child else {
			println!("Warning: failed to read a child of {addins_folder:?}");
			continue;
		};
		if child.path().join("TupeloWorkbench.addin").exists() {
			return Ok(true)
		}
	}
	Ok(false)
}



pub fn download_assets() -> Result<Vec<u8>> {
	
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
	let Some(len) = response.content_length() else {
		return Err(Error::msg("Could not get length of received assets data."));
	};
	let mut buffer = Vec::with_capacity(len as usize);
	response.read_to_end(&mut buffer).context("Attempted to read asset data")?;
	
	Ok(buffer)
}



pub fn get_format_version(zip_data: &mut ZipArchive<Cursor<&[u8]>>) -> Result<usize> {
	let file_contents = get_file_text(zip_data, "TupeloWorkbench.addin")?;
	let format_line =
		file_contents.lines()
		.find(|line| line.starts_with("<!--FORMAT_VERSION_"))
		.ok_or_else(|| Error::msg(format!("Could find format version in asset files, you may need to re-download the installer. If this is the latest version, please submit a bug report (https://github.com/{}/{}/issues).", settings::REPO_OWNER, settings::REPO_NAME)))?;
	let format_num = &format_line[19..];
	format_num.parse::<usize>().map_err(Error::from)
}



pub fn get_file_text(zip_data: &mut ZipArchive<Cursor<&[u8]>>, file_name: &'static str) -> Result<String> {
	let mut zip_file = zip_data.by_name(file_name).context(format!("Attempted to find file {file_name:?}"))?;
	let mut contents = String::with_capacity(zip_file.size() as usize);
	zip_file.read_to_string(&mut contents).context(format!("Attempted to read asset file {file_name:?}"))?;
	Ok(contents)
}

pub fn get_file_bytes(zip_data: &mut ZipArchive<Cursor<&[u8]>>, file_name: &'static str) -> Result<Vec<u8>> {
	let mut zip_file = zip_data.by_name(file_name).context(format!("Attempted to find file {file_name:?}"))?;
	let mut contents = Vec::with_capacity(zip_file.size() as usize);
	zip_file.read_to_end(&mut contents).context(format!("Attempted to read asset file {file_name:?}"))?;
	Ok(contents)
}



pub fn write_files(zip_data: &mut ZipArchive<Cursor<&[u8]>>, revit_path: &Path) -> Result<()> {
	let ext_dir = revit_path.join("Tupelo Workbench");
	
	// dlls
	fs::create_dir_all(&ext_dir).context(format!("Attempted to create folders at {ext_dir:?}"))?;
	let frontend_dll_data = get_file_bytes(zip_data, "Frontend.dll")?;
	let backend_dll_data = get_file_bytes(zip_data, "Backend.dll")?;
	
	let frontend_dll_path = ext_dir.join("Frontend.dll");
	fs::write(&frontend_dll_path, frontend_dll_data).context(format!("Attempted to write file {frontend_dll_path:?}"))?;
	let backend_dll_path = ext_dir.join("Backend.dll");
	fs::write(&backend_dll_path, backend_dll_data).context(format!("Attempted to write file {backend_dll_path:?}"))?;
	
	// readme
	let readme_file_contents = get_file_text(zip_data, "readme.txt")?;
	let readme_path = ext_dir.join("readme.txt");
	fs::write(&readme_path, readme_file_contents).context(format!("Attempted to write file {readme_path:?}"))?;
	
	// .addin
	let addins_folder = revit_path.join("Addins");
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