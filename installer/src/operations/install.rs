use crate::prelude::*;
use reqwest::blocking::Client;
use utils::unsynced_err;
use zip::ZipArchive;
use std::io::{Cursor, Read};



pub fn install(inner: Arc<Mutex<InnerApp>>, is_offline: bool, revit_path: Option<PathBuf>) -> Result<DidFinish<()>> {
	match try_install(inner.clone(), is_offline, revit_path) {
		StdResult::Ok(()) => Ok(Some(())),
		StdResult::Err(err) => {
			background_thread::show_error_message(inner, &err)?;
			Ok(None)
		}
	}
}



pub fn try_install(inner: Arc<Mutex<InnerApp>>, is_offline: bool, revit_path: Option<PathBuf>) -> Result<()> {
	
	let revit_path = operations::get_revit_path::get_revit_path(inner.clone(), "Install", revit_path)?;
	
	// check if already installed
	match check_already_installed(&revit_path) {
		StdResult::Ok(true) => {
			let mut inner_locked = inner.lock().map_err_string()?;
			inner_locked.gui_elements.clear();
			inner_locked.gui_elements.push(GuiElement::Header (String::from("Installing")));
			inner_locked.gui_elements.push(GuiElement::Separator);
			inner_locked.gui_elements.push(GuiElement::Label (String::from("Error: cannot install addin because it is already installed.")));
			inner_locked.gui_elements.push(GuiElement::Label (String::from("Please uninstall before continuing.")));
			inner_locked.gui_elements.push(GuiElement::BottomElements (vec!(
				GuiElement::Button {text: String::from("Uninstall"), just_clicked: false},
				GuiElement::Button {text: String::from("Exit"), just_clicked: false},
			)));
			drop(inner_locked);
			loop {
				thread::sleep(Duration::from_millis(100));
				let mut inner_locked = inner.lock().map_err_string()?;
				let GuiElement::BottomElements (bottom_elements) = &mut inner_locked.gui_elements[4] else {return unsynced_err();};
				let GuiElement::Button {just_clicked: uninstall_just_clicked, ..} = &mut bottom_elements[0] else {return unsynced_err();};
				let uninstall_just_clicked = mem::take(uninstall_just_clicked);
				let GuiElement::Button {just_clicked: exit_just_clicked, ..} = &mut bottom_elements[1] else {return unsynced_err();};
				let exit_just_clicked = mem::take(exit_just_clicked);
				drop(inner_locked);
				if exit_just_clicked {return Ok(());}
				if uninstall_just_clicked {
					operations::uninstall::uninstall(inner.clone(), Some(revit_path.clone()));
					break;
				}
			}
		}
		StdResult::Ok(false) => {}
		StdResult::Err(err) => {
			let mut inner_locked = inner.lock().map_err_string()?;
			inner_locked.gui_elements.clear();
			inner_locked.gui_elements.push(GuiElement::Header (String::from("Installing")));
			inner_locked.gui_elements.push(GuiElement::Separator);
			inner_locked.gui_elements.push(GuiElement::Label (String::from("Error: failed to check if this addin is already installed. Continue anyways?")));
			inner_locked.gui_elements.push(GuiElement::Label (format!("Please give Tupelo Workbench this error message: {err:#?}")));
			inner_locked.gui_elements.push(GuiElement::BottomElements (vec!(
				GuiElement::Button {text: String::from("Continue"), just_clicked: false},
				GuiElement::Button {text: String::from("Exit"), just_clicked: false},
			)));
			drop(inner_locked);
			loop {
				thread::sleep(Duration::from_millis(100));
				let mut inner_locked = inner.lock().map_err_string()?;
				let GuiElement::BottomElements (bottom_elements) = &mut inner_locked.gui_elements[4] else {return unsynced_err();};
				let GuiElement::Button {just_clicked: continue_just_clicked, ..} = &mut bottom_elements[0] else {return unsynced_err();};
				let continue_just_clicked = mem::take(continue_just_clicked);
				let GuiElement::Button {just_clicked: exit_just_clicked, ..} = &mut bottom_elements[1] else {return unsynced_err();};
				let exit_just_clicked = mem::take(exit_just_clicked);
				drop(inner_locked);
				if exit_just_clicked {return Ok(());}
				if continue_just_clicked {break;}
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
		let mut inner_locked = inner.lock().map_err_string()?;
		inner_locked.gui_elements.clear();
		inner_locked.gui_elements.push(GuiElement::Header (String::from("Installing")));
		inner_locked.gui_elements.push(GuiElement::Separator);
		inner_locked.gui_elements.push(GuiElement::Label (String::from("Downloading assets, please wait...")));
		drop(inner_locked);
		let assets = download_assets()?;
		thread::sleep(Duration::SECOND);
		assets_owned = assets;
		&assets_owned
	};
	
	let mut inner_locked = inner.lock().map_err_string()?;
	inner_locked.gui_elements.clear();
	inner_locked.gui_elements.push(GuiElement::Header (String::from("Installing")));
	inner_locked.gui_elements.push(GuiElement::Separator);
	inner_locked.gui_elements.push(GuiElement::Label (String::from("Installing, please wait...")));
	drop(inner_locked);
	
	let zip_cursor = Cursor::new(assets);
	let mut zip_data = ZipArchive::new(zip_cursor)?;
	
	let version = get_format_version(&mut zip_data).unwrap_or(settings::LATEST_ASSETS_VERSION);
	if version != settings::LATEST_ASSETS_VERSION {
		return Err(Error::msg(format!("Installer is out of date, please re-download installer to continue. If this is the latest version, please submit a bug report (https://github.com/{}/{}/issues). ", settings::REPO_OWNER, settings::REPO_NAME)));
	}
	
	if let Err(err) = write_files(&mut zip_data, &revit_path) {
		return Err(Error::msg(format!("Failed to write addin files. Please contact Tupelo Workbench with this error message: {err:#?} ")));
	}
	
	thread::sleep(Duration::SECOND);
	
	let mut inner_locked = inner.lock().map_err_string()?;
	inner_locked.gui_elements.clear();
	inner_locked.gui_elements.push(GuiElement::Header (String::from("Installing")));
	inner_locked.gui_elements.push(GuiElement::Separator);
	inner_locked.gui_elements.push(GuiElement::Label (String::from("Install finished successfully.")));
	inner_locked.gui_elements.push(GuiElement::BottomElements (vec!(
		GuiElement::Button {text: String::from("Close"), just_clicked: false},
	)));
	drop(inner_locked);
	loop {
		let mut inner_locked = inner.lock().map_err_string()?;
		let GuiElement::BottomElements (bottom_elements) = &mut inner_locked.gui_elements[3] else {return unsynced_err();};
		let GuiElement::Button {just_clicked, ..} = &mut bottom_elements[0] else {return unsynced_err();};
		let just_clicked = mem::take(just_clicked);
		if just_clicked {break;}
	}
	
	Ok(())
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
	let addin_file_contents = addin_file_contents.replace("addin_DIR", ext_dir.to_str().unwrap());
	
	for child in fs::read_dir(&addins_folder)? {
		let StdResult::Ok(child) = child else {
			println!("Warning: failed to read a child of {addins_folder:?}");
			continue;
		};
		fs::write(child.path().join("TupeloWorkbench.addin"), &addin_file_contents)?;
	}
	
	Ok(())
}
