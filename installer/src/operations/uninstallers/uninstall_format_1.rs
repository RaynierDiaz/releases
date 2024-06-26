use crate::prelude::*;
use utils::unsynced_err;



pub fn uninstall(inner: Arc<Mutex<InnerApp>>, revit_path: &Path) -> Result<()> {
	
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



pub fn delete_addin_folder(inner: Arc<Mutex<InnerApp>>, revit_path: &Path) -> Result<()> {
	
	let mut inner_locked = inner.lock().map_err_string()?;
	inner_locked.gui_elements.clear();
	inner_locked.gui_elements.push(GuiElement::Header (String::from("Uninstall")));
	inner_locked.gui_elements.push(GuiElement::Separator);
	inner_locked.gui_elements.push(GuiElement::Label (String::from("Determining addin path, please wait...")));
	drop(inner_locked);
	thread::sleep(Duration::SECOND / 3);
	
	// read addins file
	let addins_path = revit_path.join("Addins");
	let addin_file_path =
		fs::read_dir(&addins_path).context(format!("Attempted to read contents of {addins_path:?}"))?
		.find(|entry| {
			let StdResult::Ok(entry) = entry else {return false;};
			entry.path().join("TupeloWorkbench.addin").exists()
		});
	let Some(addin_file_path) = addin_file_path else {
		return Err(Error::msg("Warning: Could not find any .addin file for this addin, so the path of the dlls is unknown and cannot be removed automatically. They are usually stored at C:\\ProgramData\\TupeloWorkbenchExt, though the install path could be customized"));
	};
	let addin_file_path = addin_file_path?.path().join("TupeloWorkbench.addin");
	
	// get addin path
	let addin_contents = fs::read_to_string(&addin_file_path).context(format!("Attempted to read contents of {addin_file_path:?}"))?;
	let assembly_line = addin_contents.lines()
		.find(|line| {
			line.trim().starts_with("<Assembly>")
		})
		.expect("Could not find \"<Assembly>\" line in addin file.")
		.trim();
	
	let addin_path = PathBuf::from(&assembly_line.trim()[10..assembly_line.len()-23]);
	
	let mut inner_locked = inner.lock().map_err_string()?;
	inner_locked.gui_elements.clear();
	inner_locked.gui_elements.push(GuiElement::Header (String::from("Uninstall")));
	inner_locked.gui_elements.push(GuiElement::Separator);
	inner_locked.gui_elements.push(GuiElement::Label (format!("The detected addin path is {addin_path:?}, does that sound right?")));
	inner_locked.gui_elements.push(GuiElement::BottomElements (vec!(
		GuiElement::Button {text: String::from("Yes, Continue"), just_clicked: false},
	)));
	drop(inner_locked);
	loop {
		let mut inner_locked = inner.lock().map_err_string()?;
		let GuiElement::BottomElements (bottom_elements) = &mut inner_locked.gui_elements[3] else {return unsynced_err();};
		let GuiElement::Button {just_clicked, ..} = &mut bottom_elements[0] else {return unsynced_err();};
		let just_clicked = mem::take(just_clicked);
		if just_clicked {break;}
	}
	
	let mut inner_locked = inner.lock().map_err_string()?;
	inner_locked.gui_elements.clear();
	inner_locked.gui_elements.push(GuiElement::Header (String::from("Uninstall")));
	inner_locked.gui_elements.push(GuiElement::Separator);
	inner_locked.gui_elements.push(GuiElement::Label (String::from("Removing addin folder...")));
	drop(inner_locked);
	thread::sleep(Duration::SECOND);
	
	// delete
	loop {
		match fs::remove_dir_all(&addin_path) {
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
	inner_locked.gui_elements.push(GuiElement::Label (String::from("Removing addin files...")));
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
