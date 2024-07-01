use crate::prelude::*;
use sysinfo::System;
use utils::unsynced_err;



pub fn uninstall(inner: Arc<Mutex<InnerApp>>, revit_path: &Path) -> Result<()> {
	
	wait_for_revit_close(inner.clone())?;
	
	let result = delete_addin_folder(inner.clone(), revit_path);
	if let Err(err) = result {
		return Err(Error::msg(format!("Failed to delete addin folder, located at {:?}. Please contact Tupelo Workbench with this error message: {err:#?}.", revit_path.join("Tupelo Workbench"))));
	}
	
	let result = delete_addin_files(inner, revit_path);
	if let Err(err) = result {
		return Err(Error::msg(format!("Failed to delete addin files, located at {:?}. Please contact Tupelo Workbench with this error message: {err:#?}.", revit_path.join("Addins"))));
	}
	
	Ok(())
}



pub fn wait_for_revit_close(inner: Arc<Mutex<InnerApp>>) -> Result<()> {
	if !is_revit_running() {return Ok(());}
	let mut inner_locked = inner.lock().map_err_string()?;
	inner_locked.gui_elements.clear();
	inner_locked.gui_elements.push(GuiElement::Header (String::from("Waiting for Revit to close...")));
	inner_locked.gui_elements.push(GuiElement::Separator);
	inner_locked.gui_elements.push(GuiElement::Label (String::from("Please close Revit so the uninstall can continue")));
	inner_locked.gui_elements.push(GuiElement::Label (String::from("(It will takes several seconds for the installer to respond)")));
	drop(inner_locked);
	while is_revit_running() {thread::sleep(Duration::from_millis(100));}
	Ok(())
}

pub fn is_revit_running() -> bool {
	System::new_all().processes().values().any(|p| p.name() == "Revit.exe")
}



pub fn delete_addin_folder(inner: Arc<Mutex<InnerApp>>, revit_path: &Path) -> Result<()> {
	
	let mut inner_locked = inner.lock().map_err_string()?;
	inner_locked.gui_elements.clear();
	inner_locked.gui_elements.push(GuiElement::Header (String::from("Uninstall")));
	inner_locked.gui_elements.push(GuiElement::Separator);
	inner_locked.gui_elements.push(GuiElement::Label (String::from("Removing addin folder...")));
	drop(inner_locked);
	thread::sleep(Duration::SECOND);
	
	// delete
	loop {
		match fs::remove_dir_all(&revit_path.join("Tupelo Workbench")) {
			StdResult::Ok(()) => break,
			StdResult::Err(err) => {
				let mut inner_locked = inner.lock().map_err_string()?;
				inner_locked.gui_elements.clear();
				inner_locked.gui_elements.push(GuiElement::Header (String::from("Uninstall")));
				inner_locked.gui_elements.push(GuiElement::Separator);
				inner_locked.gui_elements.push(GuiElement::Label (format!("Failed to delete addin folder (NOTE: If Revit is open, please close it and wait a few seconds before continuing). Error message: {err:#?}")));
				inner_locked.gui_elements.push(GuiElement::BottomElements (vec!(
					GuiElement::Button {text: String::from("Retry"), just_clicked: false},
					GuiElement::Button {text: String::from("Exit"), just_clicked: false},
				)));
				drop(inner_locked);
				loop {
					let mut inner_locked = inner.lock().map_err_string()?;
					let GuiElement::BottomElements (bottom_elements) = &mut inner_locked.gui_elements[3] else {return unsynced_err();};
					let GuiElement::Button {just_clicked: retry_just_clicked, ..} = &mut bottom_elements[0] else {return unsynced_err();};
					let retry_just_clicked = mem::take(retry_just_clicked);
					let GuiElement::Button {just_clicked: exit_just_clicked, ..} = &mut bottom_elements[1] else {return unsynced_err();};
					let exit_just_clicked = mem::take(exit_just_clicked);
					if retry_just_clicked {continue;}
					if exit_just_clicked {return Err(err.into());}
				}
			}
		};
	}
	
	Ok(())
}



pub fn delete_addin_files(inner: Arc<Mutex<InnerApp>>, revit_path: &Path) -> Result<()> {
	
	let mut inner_locked = inner.lock().map_err_string()?;
	inner_locked.gui_elements.clear();
	inner_locked.gui_elements.push(GuiElement::Header (String::from("Uninstall")));
	inner_locked.gui_elements.push(GuiElement::Separator);
	inner_locked.gui_elements.push(GuiElement::Label (String::from("Removing .addin files...")));
	drop(inner_locked);
	thread::sleep(Duration::SECOND);
	
	let addins_path = revit_path.join("Addins");
	for entry in fs::read_dir(&addins_path).context(format!("Attempted to read contents of {addins_path:?}"))? {
		let StdResult::Ok(entry) = entry else {continue;};
		let addin_file_path = entry.path().join("TupeloWorkbench.addin");
		if addin_file_path.exists() {
			fs::remove_file(addin_file_path)?;
		}
	}
	
	Ok(())
}
