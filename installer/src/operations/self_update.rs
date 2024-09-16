use crate::prelude::*;
use utils::unsynced_err;



pub fn self_update(app: Arc<Mutex<App>>) -> Result<DidFinish<()>> {
	match try_self_update(app.clone()) {
		StdResult::Ok(()) => Ok(Some(())),
		StdResult::Err(err) => {
			background_thread::show_error_message(app, &err)?;
			Ok(None)
		}
	}
}



pub fn try_self_update(app: Arc<Mutex<App>>) -> Result<()> {
	
	// uninstall
	let revit_path = crate::operations::uninstall::try_uninstall(app.clone(), None, true, true)?;
	
	// reinstall
	crate::operations::install::try_install(app.clone(), Some(revit_path), true)?;
	
	// delete self
	self_replace::self_delete().with_context(|| format!("Failed to delete temporary installer. Once this is closed, please delete this file: {:?}", std::env::current_dir().unwrap()))?;
	
	let mut app_locked = app.lock().map_err_string()?;
	app_locked.gui_elements.clear();
	app_locked.gui_elements.push(GuiElement::Header (String::from("Self Update")));
	app_locked.gui_elements.push(GuiElement::Separator);
	app_locked.gui_elements.push(GuiElement::Label (String::from("Self-update finished successfully.")));
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
	
	Ok(())
}
