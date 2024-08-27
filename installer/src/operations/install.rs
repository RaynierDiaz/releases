use crate::prelude::*;
use reqwest::blocking::Client;
use utils::unsynced_err;
use zip::ZipArchive;
use std::io::{Cursor, Read};



pub fn install(app: Arc<Mutex<App>>, revit_path: Option<PathBuf>, is_self_update: bool) -> Result<DidFinish<()>> {
	match try_install(app.clone(), revit_path, is_self_update) {
		StdResult::Ok(()) => Ok(Some(())),
		StdResult::Err(err) => {
			background_thread::show_error_message(app, &err)?;
			Ok(None)
		}
	}
}



pub fn try_install(app: Arc<Mutex<App>>, revit_path: Option<PathBuf>, is_self_update: bool) -> Result<()> {
	
	let revit_path = operations::get_revit_path::get_revit_path(app.clone(), "Install", revit_path)?;
	
	// check if already installed
	match check_already_installed(&revit_path) {
		StdResult::Ok(true) => {
			let mut app_locked = app.lock().map_err_string()?;
			app_locked.gui_elements.clear();
			app_locked.gui_elements.push(GuiElement::Header (String::from("Installing")));
			app_locked.gui_elements.push(GuiElement::Separator);
			app_locked.gui_elements.push(GuiElement::Label (String::from("Error: cannot install addin because it is already installed.")));
			app_locked.gui_elements.push(GuiElement::Label (String::from("Please uninstall before continuing.")));
			app_locked.gui_elements.push(GuiElement::BottomElements (vec!(
				GuiElement::Button {text: String::from("Uninstall"), just_clicked: false},
				GuiElement::Button {text: String::from("Exit"), just_clicked: false},
			)));
			drop(app_locked);
			loop {
				thread::sleep(Duration::from_millis(100));
				let mut app_locked = app.lock().map_err_string()?;
				let GuiElement::BottomElements (bottom_elements) = &mut app_locked.gui_elements[4] else {return unsynced_err();};
				let GuiElement::Button {just_clicked: uninstall_just_clicked, ..} = &mut bottom_elements[0] else {return unsynced_err();};
				let uninstall_just_clicked = mem::take(uninstall_just_clicked);
				let GuiElement::Button {just_clicked: exit_just_clicked, ..} = &mut bottom_elements[1] else {return unsynced_err();};
				let exit_just_clicked = mem::take(exit_just_clicked);
				drop(app_locked);
				if exit_just_clicked {return Ok(());}
				if uninstall_just_clicked {
					operations::uninstall::try_uninstall(app.clone(), Some(revit_path.clone()), is_self_update, true)?;
					break;
				}
			}
		}
		StdResult::Ok(false) => {}
		StdResult::Err(err) => {
			let mut app_locked = app.lock().map_err_string()?;
			app_locked.gui_elements.clear();
			app_locked.gui_elements.push(GuiElement::Header (String::from("Installing")));
			app_locked.gui_elements.push(GuiElement::Separator);
			app_locked.gui_elements.push(GuiElement::Label (String::from("Error: failed to check if this addin is already installed. Continue anyways?")));
			app_locked.gui_elements.push(GuiElement::Label (format!("Please give Tupelo Workbench this error message: {err:#?}")));
			app_locked.gui_elements.push(GuiElement::BottomElements (vec!(
				GuiElement::Button {text: String::from("Continue"), just_clicked: false},
				GuiElement::Button {text: String::from("Exit"), just_clicked: false},
			)));
			drop(app_locked);
			loop {
				thread::sleep(Duration::from_millis(100));
				let mut app_locked = app.lock().map_err_string()?;
				let GuiElement::BottomElements (bottom_elements) = &mut app_locked.gui_elements[4] else {return unsynced_err();};
				let GuiElement::Button {just_clicked: continue_just_clicked, ..} = &mut bottom_elements[0] else {return unsynced_err();};
				let continue_just_clicked = mem::take(continue_just_clicked);
				let GuiElement::Button {just_clicked: exit_just_clicked, ..} = &mut bottom_elements[1] else {return unsynced_err();};
				let exit_just_clicked = mem::take(exit_just_clicked);
				drop(app_locked);
				if exit_just_clicked {return Ok(());}
				if continue_just_clicked {break;}
			}
		}
	}
	
	let zip_cursor = Cursor::new(&include_bytes!("../Assets.zip")[..]);
	let mut zip_data = ZipArchive::new(zip_cursor)?;
	
	let version = get_format_version(&mut zip_data).unwrap_or(LATEST_ASSETS_VERSION);
	if version != LATEST_ASSETS_VERSION {
		return Err(Error::msg(format!("Installer is out of date, please re-download installer to continue. If this is the latest version, please submit a bug report.")));
	}
	
	let mut app_locked = app.lock().map_err_string()?;
	app_locked.gui_elements.clear();
	app_locked.gui_elements.push(GuiElement::Header (String::from("Installing")));
	app_locked.gui_elements.push(GuiElement::Separator);
	app_locked.gui_elements.push(GuiElement::Label (String::from("Installing, please wait...")));
	drop(app_locked);
	
	// install files
	if let Err(err) = write_files(&mut zip_data, &revit_path) {
		return Err(Error::msg(format!("Failed to write addin files. Please contact Tupelo Workbench with this error message: {err:#?}")));
	}
	
	thread::sleep(Duration::SECOND);
	
	if !is_self_update {
		let mut app_locked = app.lock().map_err_string()?;
		app_locked.gui_elements.clear();
		app_locked.gui_elements.push(GuiElement::Header (String::from("Installing")));
		app_locked.gui_elements.push(GuiElement::Separator);
		app_locked.gui_elements.push(GuiElement::Label (String::from("Install finished successfully.")));
		app_locked.gui_elements.push(GuiElement::BottomElements (vec!(
			GuiElement::Button {text: String::from("Close"), just_clicked: false},
		)));
		drop(app_locked);
		loop {
			thread::sleep(Duration::from_millis(100));
			let mut app_locked = app.lock().map_err_string()?;
			let GuiElement::BottomElements (bottom_elements) = &mut app_locked.gui_elements[3] else {return unsynced_err();};
			let GuiElement::Button {just_clicked, ..} = &mut bottom_elements[0] else {return unsynced_err();};
			let just_clicked = mem::take(just_clicked);
			if just_clicked {break;}
		}
	}
	
	Ok(())
}



pub fn check_already_installed(revit_path: &Path) -> Result<bool> {
	let addin_file_name = format!("{}.addin", settings::ADDIN_NAME);
	let addins_folder = revit_path.join("Addins");
	for child in fs::read_dir(&addins_folder)? {
		let StdResult::Ok(child) = child else {
			println!("Warning: failed to read a child of {addins_folder:?}");
			continue;
		};
		if child.path().join(&addin_file_name).exists() {
			return Ok(true)
		}
	}
	Ok(false)
}



pub fn download_assets() -> Result<Vec<u8>> {
	
	// download assets
	let mut response =
		Client::new()
		.get(settings::ASSETS_URL)
		.send()
		.context("Attempted to send download request")?;
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
	let file_contents = get_file_text(zip_data, "AddinFile.addin")?;
	let format_line =
		file_contents.lines()
		.find(|line| line.starts_with("<!--FORMAT_VERSION_"))
		.ok_or_else(|| Error::msg(format!("Could find format version in asset files, you may need to re-download the installer. If this is the latest version, please submit a bug report.")))?;
	let format_num = &format_line[19..];
	format_num.parse::<usize>().map_err(Error::from)
}



pub fn get_file_text(zip_data: &mut ZipArchive<Cursor<&[u8]>>, file_name: &str) -> Result<String> {
	let mut zip_file = zip_data.by_name(file_name).context(format!("Attempted to find file {file_name:?}"))?;
	let mut contents = String::with_capacity(zip_file.size() as usize);
	zip_file.read_to_string(&mut contents).context(format!("Attempted to read asset file {file_name:?}"))?;
	Ok(contents)
}

pub fn get_file_bytes(zip_data: &mut ZipArchive<Cursor<&[u8]>>, file_name: &str) -> Result<Vec<u8>> {
	let mut zip_file = zip_data.by_name(file_name).context(format!("Attempted to find file {file_name:?}"))?;
	let mut contents = Vec::with_capacity(zip_file.size() as usize);
	zip_file.read_to_end(&mut contents).context(format!("Attempted to read asset file {file_name:?}"))?;
	Ok(contents)
}



pub fn write_files(zip_data: &mut ZipArchive<Cursor<&[u8]>>, revit_path: &Path) -> Result<()> {
	let ext_dir = revit_path.join(settings::ADDIN_NAME);
	
	let replace_description_labels = |input: String| -> String {
		let input = input.replace("EXTENSION_DIR", ext_dir.to_str().unwrap());
		let input = input.replace("ADDIN_NAME", settings::ADDIN_NAME);
		let input = input.replace("ADDIN_ID", settings::ADDIN_ID);
		let input = input.replace("VENDOR_DESCRIPTION", settings::VENDOR_DESCRIPTION);
		let input = input.replace("INSTALLER_URL", settings::INSTALLER_URL);
		let input = input.replace("ASSEMBLY_NAME", settings::ASSEMBLY_NAME);
		let input = input.replace("FULL_CLASS_NAME", settings::FULL_CLASS_NAME);
		input
	};
	
	// dlls
	fs::create_dir_all(&ext_dir).context(format!("Attempted to create folders at {ext_dir:?}"))?;
	for i in 0..zip_data.len() {
		let file = zip_data.by_index(i)?;
		let file_path = file.name().to_string();
		drop(file);
		if file_path.starts_with("Program") && !file_path.ends_with('/') {
			let mut file_data = get_file_bytes(zip_data, &file_path)?;
			let file_name = &file_path[8..];
			if file_name == "readme.txt" {
				file_data = replace_description_labels(String::from_utf8(file_data)?).into();
			}
			let new_file_path = ext_dir.join(file_name);
			fs::create_dir_all(new_file_path.parent().expect("Attempted to write file that has no parent"))?;
			fs::write(&new_file_path, file_data).context(format!("Attempted to write file {new_file_path:?}"))?;
		}
	}
	
	// .addin
	let addins_folder = revit_path.join("Addins");
	let addin_file_contents = get_file_text(zip_data, "AddinFile.addin")?;
	let addin_file_contents = replace_description_labels(addin_file_contents);
	
	for child in fs::read_dir(&addins_folder)? {
		let StdResult::Ok(child) = child else {
			println!("Warning: failed to read a child of {addins_folder:?}");
			continue;
		};
		fs::write(child.path().join(format!("{}.addin", settings::ADDIN_NAME)), &addin_file_contents)?;
	}
	
	Ok(())
}
