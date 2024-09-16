use crate::prelude::*;
use utils::unsynced_err;



pub fn get_revit_path(app: Arc<Mutex<App>>, header: &'static str, given_revit_path: Option<PathBuf>) -> Result<PathBuf> {
	
	let mut app_locked = app.lock().map_err_string()?;
	app_locked.gui_elements.clear();
	app_locked.gui_elements.push(GuiElement::Header (String::from(header)));
	app_locked.gui_elements.push(GuiElement::Separator);
	app_locked.gui_elements.push(GuiElement::Label (String::from("Location Revit, please wait...")));
	drop(app_locked);
	
	thread::sleep(Duration::SECOND / 2);
	
	const DEFAULT_REVIT_PATH: &str = "C:\\ProgramData\\Autodesk\\Revit";
	let revit_path = given_revit_path.unwrap_or_else(|| PathBuf::from(DEFAULT_REVIT_PATH));
	if revit_path.exists() && revit_path.join("Addins").exists() {return Ok(revit_path);}
	
	let mut app_locked = app.lock().map_err_string()?;
	app_locked.gui_elements.clear();
	app_locked.gui_elements.push(GuiElement::Header (String::from(header)));
	app_locked.gui_elements.push(GuiElement::Separator);
	app_locked.gui_elements.push(GuiElement::Label (String::from("Revit could not be found, please enter the path to Revit:")));
	app_locked.gui_elements.push(GuiElement::TextBox (String::from("C:\\ProgramData\\Autodesk\\Revit")));
	app_locked.gui_elements.push(GuiElement::BottomElements (vec!(
		GuiElement::Button {text: String::from("Next"), was_clicked: false},
	)));
	drop(app_locked);
	let mut has_error_msg = false;
	loop {
		thread::sleep(Duration::from_millis(100));
		let mut app_locked = app.lock().map_err_string()?;
		let GuiElement::BottomElements (bottom_elements) = &mut app_locked.gui_elements[4 + has_error_msg as usize] else {return unsynced_err();};
		let GuiElement::Button {was_clicked: just_clicked, ..} = &mut bottom_elements[0] else {return unsynced_err();};
		let just_clicked = mem::take(just_clicked);
		if just_clicked {
			let GuiElement::TextBox (path_text) = &app_locked.gui_elements[3] else {return unsynced_err();};
			let path = PathBuf::from(path_text);
			if !path.exists() {
				if !has_error_msg {
					has_error_msg = true;
					app_locked.gui_elements.insert(4, GuiElement::Label (String::new()));
				}
				let GuiElement::Label (error_msg_text) = &mut app_locked.gui_elements[4] else {return unsynced_err();};
				*error_msg_text = String::from("That path does not exist");
				continue;
			}
			if !path.join("Addins").exists() {
				if !has_error_msg {
					has_error_msg = true;
					app_locked.gui_elements.insert(4, GuiElement::Label (String::new()));
				}
				let GuiElement::Label (error_msg_text) = &mut app_locked.gui_elements[4] else {return unsynced_err();};
				*error_msg_text = String::from("That path is not valid, please enter the folder which encloses the \"Addins\" folder");
				continue;
			}
			return Ok(path);
		}
	}
}
