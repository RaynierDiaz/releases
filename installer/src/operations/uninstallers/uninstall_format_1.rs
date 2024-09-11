use crate::prelude::*;
use sysinfo::System;
use utils::unsynced_err;



pub fn uninstall(app: Arc<Mutex<App>>, revit_path: &Path, is_reinstall: bool) -> Result<()> {
	
	wait_for_revit_close(app.clone())?;
	
	let keep_settings = is_reinstall || ask_keep_settings(app.clone())?;
	
	let result = delete_addin_folder(app.clone(), revit_path, keep_settings);
	if let Err(err) = result {
		return Err(anyhow!("Failed to delete addin folder, located at {:?}. Please contact Workbench LLC with this error message: {err:#?}.", revit_path.join(settings::ADDIN_NAME)));
	}
	
	let result = delete_addin_files(app, revit_path);
	if let Err(err) = result {
		return Err(anyhow!("Failed to delete addin files, located at {:?}. Please contact Workbench LLC with this error message: {err:#?}.", revit_path.join("Addins")));
	}
	
	Ok(())
}



pub fn wait_for_revit_close(app: Arc<Mutex<App>>) -> Result<()> {
	if !is_revit_running() {return Ok(());}
	let mut app_locked = app.lock().map_err_string()?;
	app_locked.gui_elements.clear();
	app_locked.gui_elements.push(GuiElement::Header (String::from("Waiting for Revit to close...")));
	app_locked.gui_elements.push(GuiElement::Separator);
	app_locked.gui_elements.push(GuiElement::Label (String::from("Please close Revit so the uninstall can continue")));
	app_locked.gui_elements.push(GuiElement::Label (String::from("(It will takes several seconds for the installer to respond)")));
	drop(app_locked);
	while is_revit_running() {thread::sleep(Duration::from_millis(100));}
	Ok(())
}

pub fn is_revit_running() -> bool {
	System::new_all().processes().values().any(|p| p.name() == "Revit.exe")
}



pub fn ask_keep_settings(app: Arc<Mutex<App>>) -> Result<bool> {
	let mut app_locked = app.lock().map_err_string()?;
	app_locked.gui_elements.clear();
	app_locked.gui_elements.push(GuiElement::Header (String::from("Uninstall")));
	app_locked.gui_elements.push(GuiElement::Separator);
	app_locked.gui_elements.push(GuiElement::Label (String::from("Do you want to keep any settings you have?")));
	app_locked.gui_elements.push(GuiElement::BottomElements (vec!(
		GuiElement::Button {text: String::from("Yes"), was_clicked: false},
		GuiElement::Button {text: String::from("No"), was_clicked: false},
	)));
	drop(app_locked);
	loop {
		thread::sleep(Duration::from_millis(100));
		let mut app_locked = app.lock().map_err_string()?;
		let GuiElement::BottomElements (bottom_elements) = &mut app_locked.gui_elements[3] else {return unsynced_err();};
		let GuiElement::Button {was_clicked: keep_just_clicked, ..} = &mut bottom_elements[0] else {return unsynced_err();};
		let keep_just_clicked = mem::take(keep_just_clicked);
		let GuiElement::Button {was_clicked: delete_just_clicked, ..} = &mut bottom_elements[1] else {return unsynced_err();};
		let delete_just_clicked = mem::take(delete_just_clicked);
		if keep_just_clicked {return Ok(true);}
		if delete_just_clicked {return Ok(false);}
	}
}



pub fn delete_addin_folder(app: Arc<Mutex<App>>, revit_path: &Path, keep_settings: bool) -> Result<()> {
	
	let mut app_locked = app.lock().map_err_string()?;
	app_locked.gui_elements.clear();
	app_locked.gui_elements.push(GuiElement::Header (String::from("Uninstall")));
	app_locked.gui_elements.push(GuiElement::Separator);
	app_locked.gui_elements.push(GuiElement::Label (String::from("Removing addin folder...")));
	drop(app_locked);
	thread::sleep(Duration::SECOND);
	
	let delete_folder = || -> Result<()> {
		for entry in fs::read_dir(revit_path.join(settings::ADDIN_NAME))? {
			let entry = entry?.path();
			if keep_settings && entry.file_name() == Some("settings.txt".as_ref()) {
				continue;
			}
			if entry.is_dir() {
				fs::remove_dir_all(entry)?;
			} else {
				fs::remove_file(entry)?;
			}
		}
		Ok(())
	};
	
	// delete
	'outer: loop {
		match delete_folder() {
			StdResult::Ok(()) => break,
			StdResult::Err(err) => {
				let mut app_locked = app.lock().map_err_string()?;
				app_locked.gui_elements.clear();
				app_locked.gui_elements.push(GuiElement::Header (String::from("Uninstall")));
				app_locked.gui_elements.push(GuiElement::Separator);
				app_locked.gui_elements.push(GuiElement::Label (format!("Failed to delete addin folder (NOTE: If Revit is open, please close it and wait a few seconds before continuing). Error message: {err:#?}")));
				app_locked.gui_elements.push(GuiElement::BottomElements (vec!(
					GuiElement::Button {text: String::from("Retry"), was_clicked: false},
					GuiElement::Button {text: String::from("Exit"), was_clicked: false},
				)));
				drop(app_locked);
				loop {
					thread::sleep(Duration::from_millis(100));
					let mut app_locked = app.lock().map_err_string()?;
					let GuiElement::BottomElements (bottom_elements) = &mut app_locked.gui_elements[3] else {return unsynced_err();};
					let GuiElement::Button {was_clicked: retry_just_clicked, ..} = &mut bottom_elements[0] else {return unsynced_err();};
					let retry_just_clicked = mem::take(retry_just_clicked);
					let GuiElement::Button {was_clicked: exit_just_clicked, ..} = &mut bottom_elements[1] else {return unsynced_err();};
					let exit_just_clicked = mem::take(exit_just_clicked);
					if retry_just_clicked {continue 'outer;}
					if exit_just_clicked {return Err(err);}
				}
			}
		};
	}
	
	Ok(())
}



pub fn delete_addin_files(app: Arc<Mutex<App>>, revit_path: &Path) -> Result<()> {
	
	let mut app_locked = app.lock().map_err_string()?;
	app_locked.gui_elements.clear();
	app_locked.gui_elements.push(GuiElement::Header (String::from("Uninstall")));
	app_locked.gui_elements.push(GuiElement::Separator);
	app_locked.gui_elements.push(GuiElement::Label (String::from("Removing .addin files...")));
	drop(app_locked);
	thread::sleep(Duration::SECOND);
	
	let addin_file_name = format!("{}.addin", settings::ADDIN_NAME);
	let addins_path = revit_path.join("Addins");
	for entry in fs::read_dir(&addins_path).context(format!("Attempted to read contents of {addins_path:?}"))? {
		let StdResult::Ok(entry) = entry else {continue;};
		let addin_file_path = entry.path().join(&addin_file_name);
		if addin_file_path.exists() {
			fs::remove_file(addin_file_path)?;
		}
	}
	
	Ok(())
}
