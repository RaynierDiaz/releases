use crate::prelude::*;
use utils::unsynced_err;



pub fn get_revit_path(inner: Arc<Mutex<InnerApp>>, header: &'static str) -> Result<PathBuf> {
	
	let mut inner_locked = inner.lock().map_err_string()?;
	inner_locked.gui_elements.clear();
	inner_locked.gui_elements.push(GuiElement::Header (String::from(header)));
	inner_locked.gui_elements.push(GuiElement::Separator);
	inner_locked.gui_elements.push(GuiElement::Label (String::from("Revit could not be found, please enter the path to Revit:")));
	inner_locked.gui_elements.push(GuiElement::TextBox (String::from("C:\\ProgramData\\Autodesk\\Revit")));
	inner_locked.gui_elements.push(GuiElement::BottomElements (vec!(
		GuiElement::Button {text: String::from("Next"), just_clicked: false},
	)));
	drop(inner_locked);
	let mut has_error_msg = false;
	loop {
		thread::sleep(Duration::from_millis(100));
		let mut inner_locked = inner.lock().map_err_string()?;
		let GuiElement::BottomElements (bottom_elements) = &mut inner_locked.gui_elements[4 + has_error_msg as usize] else {return unsynced_err();};
		let GuiElement::Button {just_clicked, ..} = &mut bottom_elements[0] else {return unsynced_err();};
		let just_clicked = mem::take(just_clicked);
		if just_clicked {
			let GuiElement::TextBox (path_text) = &inner_locked.gui_elements[3] else {return unsynced_err();};
			let path = PathBuf::from(path_text);
			if !path.exists() {
				if !has_error_msg {
					has_error_msg = true;
					inner_locked.gui_elements.insert(4, GuiElement::Label (String::new()));
				}
				let GuiElement::Label (error_msg_text) = &mut inner_locked.gui_elements[4] else {return unsynced_err();};
				*error_msg_text = String::from("That path does not exist");
				continue;
			}
			if !path.join("Addins").exists() {
				if !has_error_msg {
					has_error_msg = true;
					inner_locked.gui_elements.insert(4, GuiElement::Label (String::new()));
				}
				let GuiElement::Label (error_msg_text) = &mut inner_locked.gui_elements[4] else {return unsynced_err();};
				*error_msg_text = String::from("That path is not valid, please enter the folder which encloses the \"Addins\" folder");
				continue;
			}
			return Ok(path);
		}
	}
}
