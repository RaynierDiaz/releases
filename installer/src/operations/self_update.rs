use crate::prelude::*;
use utils::unsynced_err;



pub fn self_update(inner: Arc<Mutex<InnerApp>>) -> Result<DidFinish<()>> {
	match try_self_update(inner.clone()) {
		StdResult::Ok(()) => Ok(Some(())),
		StdResult::Err(err) => {
			background_thread::show_error_message(inner, &err)?;
			Ok(None)
		}
	}
}



pub fn try_self_update(inner: Arc<Mutex<InnerApp>>) -> Result<()> {
	
	// uninstall
	let revit_path = crate::operations::uninstall::try_uninstall(inner.clone(), None, true)?;
	
	// reinstall
	crate::operations::install::try_install(inner.clone(), true, Some(revit_path), true)?;
	
	// delete self
	self_replace::self_delete().with_context(|| format!("Failed to delete temporary installer. Once this is closed, please delete this file: {:?}", std::env::current_dir().unwrap()))?;
	
	let mut inner_locked = inner.lock().map_err_string()?;
	inner_locked.gui_elements.clear();
	inner_locked.gui_elements.push(GuiElement::Header (String::from("Self Update")));
	inner_locked.gui_elements.push(GuiElement::Separator);
	inner_locked.gui_elements.push(GuiElement::Label (String::from("Self-update finished successfully.")));
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
