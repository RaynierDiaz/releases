use crate::prelude::*;
use crate::operations::uninstallers;
use utils::unsynced_err;



pub fn uninstall(inner: Arc<Mutex<InnerApp>>, revit_path: Option<PathBuf>, is_self_update: bool) -> Result<DidFinish<PathBuf>> {
	match try_uninstall(inner.clone(), revit_path, is_self_update) {
		StdResult::Ok(revit_dir) => Ok(Some(revit_dir)),
		StdResult::Err(err) => {
			background_thread::show_error_message(inner, &err)?;
			Ok(None)
		}
	}
}



pub fn try_uninstall(inner: Arc<Mutex<InnerApp>>, revit_path: Option<PathBuf>, is_self_update: bool) -> Result<PathBuf> {
	
	let revit_path = operations::get_revit_path::get_revit_path(inner.clone(), "Uninstall", revit_path)?;
	
	let mut inner_locked = inner.lock().map_err_string()?;
	inner_locked.gui_elements.clear();
	inner_locked.gui_elements.push(GuiElement::Header (String::from("Uninstall")));
	inner_locked.gui_elements.push(GuiElement::Separator);
	inner_locked.gui_elements.push(GuiElement::Label (String::from("Determining addin format version, please wait...")));
	drop(inner_locked);
	
	thread::sleep(Duration::SECOND / 3);
	
	let format_version = get_format_version(&revit_path)?;
	match format_version {
		1 => uninstallers::uninstall_format_1::uninstall(inner.clone(), &revit_path)?,
		2 => uninstallers::uninstall_format_2::uninstall(inner.clone(), &revit_path)?,
		3 => uninstallers::uninstall_format_3::uninstall(inner.clone(), &revit_path)?,
		_ => return Err(Error::msg(format!("Unknown format version: {format_version}"))),
	}
	
	if !is_self_update {
		let mut inner_locked = inner.lock().map_err_string()?;
		inner_locked.gui_elements.clear();
		inner_locked.gui_elements.push(GuiElement::Header (String::from("Uninstall")));
		inner_locked.gui_elements.push(GuiElement::Separator);
		inner_locked.gui_elements.push(GuiElement::Label (String::from("Uninstall finished successfully.")));
		inner_locked.gui_elements.push(GuiElement::BottomElements (vec!(
			GuiElement::Button {text: String::from("Close"), just_clicked: false},
		)));
		drop(inner_locked);
		loop {
			thread::sleep(Duration::from_millis(100));
			let mut inner_locked = inner.lock().map_err_string()?;
			let GuiElement::BottomElements (bottom_elements) = &mut inner_locked.gui_elements[3] else {return unsynced_err();};
			let GuiElement::Button {just_clicked: close_just_clicked, ..} = &mut bottom_elements[0] else {return unsynced_err();};
			let close_just_clicked = mem::take(close_just_clicked);
			if close_just_clicked {break;}
		}
	}
	
	Ok(revit_path)
}



pub fn get_format_version(revit_path: &Path) -> Result<usize> {
	let addin_file_path = 'addin_path: {
		for entry in fs::read_dir(revit_path.join("Addins"))? {
			let StdResult::Ok(entry) = entry else {continue;};
			let entry = entry.path();
			let addin_file_path = entry.join("TupeloWorkbench.addin");
			if !addin_file_path.exists() {continue;}
			break 'addin_path addin_file_path;
		}
		return Err(Error::msg("Could not find any .addin files for Tupelo Workbench"));
	};
	let format_version = {
		let addin_contents = fs::read_to_string(addin_file_path)?;
		let format_line =
			addin_contents.lines()
			.find(|line| line.starts_with("<!--FORMAT_VERSION_"))
			.unwrap_or_else(|| {
				println!("Warning: could not find format version within files, assuming installed version if format 1...");
				"<!--FORMAT_VERSION_1"
			});
		let format_num = &format_line[19..];
		format_num.parse::<usize>().map_err(Error::from)?
	};
	Ok(format_version)
}
