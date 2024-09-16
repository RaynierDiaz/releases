use crate::prelude::*;
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
	
	// agree to eula
	let mut app_locked = app.lock().map_err_string()?;
	app_locked.gui_elements.clear();
	app_locked.gui_elements.push(GuiElement::Header (String::from("Eula")));
	app_locked.gui_elements.push(GuiElement::Separator);
	app_locked.gui_elements.push(GuiElement::ScrollingLabel (include_str!("../eula.txt").to_string()));
	app_locked.gui_elements.push(GuiElement::BottomElements (vec!(
		GuiElement::Button {text: String::from("I Agree"), was_clicked: false},
		GuiElement::Button {text: String::from("I Don't Agree"), was_clicked: false},
	)));
	drop(app_locked);
	loop {
		thread::sleep(Duration::from_millis(100));
		let mut app_locked = app.lock().map_err_string()?;
		let GuiElement::BottomElements (bottom_elements) = &mut app_locked.gui_elements[3] else {return unsynced_err();};
		let GuiElement::Button {was_clicked: agree_just_clicked, ..} = &mut bottom_elements[0] else {return unsynced_err();};
		let agree_just_clicked = mem::take(agree_just_clicked);
		let GuiElement::Button {was_clicked: dont_agree_just_clicked, ..} = &mut bottom_elements[1] else {return unsynced_err();};
		let dont_agree_just_clicked = mem::take(dont_agree_just_clicked);
		drop(app_locked);
		if agree_just_clicked {break;}
		if dont_agree_just_clicked {
			let mut app_locked = app.lock().map_err_string()?;
			app_locked.gui_elements.clear();
			app_locked.gui_elements.push(GuiElement::Header (String::from("Eula Rejected")));
			app_locked.gui_elements.push(GuiElement::Separator);
			app_locked.gui_elements.push(GuiElement::Label (String::from("Affirmed, the installer will now close.")));
			app_locked.gui_elements.push(GuiElement::BottomElements (vec!(
				GuiElement::Button {text: String::from("Close"), was_clicked: false},
			)));
			drop(app_locked);
			loop {
				thread::sleep(Duration::from_millis(100));
				let mut app_locked = app.lock().map_err_string()?;
				let GuiElement::BottomElements (bottom_elements) = &mut app_locked.gui_elements[3] else {return unsynced_err();};
				let GuiElement::Button {was_clicked: close_just_clicked, ..} = &mut bottom_elements[0] else {return unsynced_err();};
				let close_just_clicked = mem::take(close_just_clicked);
				if close_just_clicked {
					return Ok(());
				}
			}
		}
	}
	
	let revit_path = operations::get_revit_path::get_revit_path(app.clone(), "Install", revit_path)?;
	
	// check if already installed
	match check_already_installed(&revit_path) {
		StdResult::Ok(true) => {
			let mut app_locked = app.lock().map_err_string()?;
			app_locked.gui_elements.clear();
			app_locked.gui_elements.push(GuiElement::Header (String::from("Installing")));
			app_locked.gui_elements.push(GuiElement::Separator);
			app_locked.gui_elements.push(GuiElement::Label (String::from("Note: the old version needs to be uninstalled before the new version is installed.")));
			app_locked.gui_elements.push(GuiElement::Label (String::from("In case you just want to cancel the installation, that option is also given.")));
			app_locked.gui_elements.push(GuiElement::BottomElements (vec!(
				GuiElement::Button {text: String::from("Uninstall"), was_clicked: false},
				GuiElement::Button {text: String::from("Exit"), was_clicked: false},
			)));
			drop(app_locked);
			loop {
				thread::sleep(Duration::from_millis(100));
				let mut app_locked = app.lock().map_err_string()?;
				let GuiElement::BottomElements (bottom_elements) = &mut app_locked.gui_elements[4] else {return unsynced_err();};
				let GuiElement::Button {was_clicked: uninstall_just_clicked, ..} = &mut bottom_elements[0] else {return unsynced_err();};
				let uninstall_just_clicked = mem::take(uninstall_just_clicked);
				let GuiElement::Button {was_clicked: exit_just_clicked, ..} = &mut bottom_elements[1] else {return unsynced_err();};
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
			app_locked.gui_elements.push(GuiElement::Label (format!("Please give Workbench LLC this error message: {err:#?}")));
			app_locked.gui_elements.push(GuiElement::BottomElements (vec!(
				GuiElement::Button {text: String::from("Continue"), was_clicked: false},
				GuiElement::Button {text: String::from("Exit"), was_clicked: false},
			)));
			drop(app_locked);
			loop {
				thread::sleep(Duration::from_millis(100));
				let mut app_locked = app.lock().map_err_string()?;
				let GuiElement::BottomElements (bottom_elements) = &mut app_locked.gui_elements[4] else {return unsynced_err();};
				let GuiElement::Button {was_clicked: continue_just_clicked, ..} = &mut bottom_elements[0] else {return unsynced_err();};
				let continue_just_clicked = mem::take(continue_just_clicked);
				let GuiElement::Button {was_clicked: exit_just_clicked, ..} = &mut bottom_elements[1] else {return unsynced_err();};
				let exit_just_clicked = mem::take(exit_just_clicked);
				drop(app_locked);
				if exit_just_clicked {return Ok(());}
				if continue_just_clicked {break;}
			}
		}
	}
	
	let zip_cursor = Cursor::new(&include_bytes!("../Assets.zip")[..]);
	let mut zip_data = ZipArchive::new(zip_cursor)?;
	
	let mut app_locked = app.lock().map_err_string()?;
	app_locked.gui_elements.clear();
	app_locked.gui_elements.push(GuiElement::Header (String::from("Installing")));
	app_locked.gui_elements.push(GuiElement::Separator);
	app_locked.gui_elements.push(GuiElement::Label (String::from("Installing, please wait...")));
	drop(app_locked);
	
	// install files
	if let Err(err) = write_files(&mut zip_data, &revit_path) {
		return Err(anyhow!("Failed to write addin files. Please contact Workbench LLC with this error message: {err:#?}"));
	}
	
	thread::sleep(Duration::SECOND);
	
	if !is_self_update {
		let mut app_locked = app.lock().map_err_string()?;
		app_locked.gui_elements.clear();
		app_locked.gui_elements.push(GuiElement::Header (String::from("Installing")));
		app_locked.gui_elements.push(GuiElement::Separator);
		app_locked.gui_elements.push(GuiElement::Label (String::from("Install finished successfully.")));
		app_locked.gui_elements.push(GuiElement::BottomElements (vec!(
			GuiElement::Button {text: String::from("Close"), was_clicked: false},
		)));
		drop(app_locked);
		loop {
			thread::sleep(Duration::from_millis(100));
			let mut app_locked = app.lock().map_err_string()?;
			let GuiElement::BottomElements (bottom_elements) = &mut app_locked.gui_elements[3] else {return unsynced_err();};
			let GuiElement::Button {was_clicked: just_clicked, ..} = &mut bottom_elements[0] else {return unsynced_err();};
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
	
	let replace_description_labels = |mut input: String| -> String {
		input = input.replace("EXTENSION_DIR", ext_dir.to_str().unwrap());
		input = input.replace("ADDIN_NAME", settings::ADDIN_NAME);
		input = input.replace("ADDIN_ID", settings::ADDIN_ID);
		input = input.replace("INSTALLER_URL", settings::INSTALLER_URL);
		input = input.replace("ASSEMBLY_NAME", settings::ASSEMBLY_NAME);
		input = input.replace("FULL_CLASS_NAME", settings::FULL_CLASS_NAME);
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
