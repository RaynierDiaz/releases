use crate::prelude::*;
use utils::unsynced_err;



pub fn run(app: Arc<Mutex<App>>) {
	let result = try_run(app.clone());
	if let Err(err) = result {
		let result = show_error_message(app, &err);
		if let Err(msg_err) = result {
			utils::fatal_error(format!("Failed to show error message in installer window. Error message error: {msg_err:#?}\nOriginal error: {err:#?}"));
		}
	}
}



pub fn show_error_message(app: Arc<Mutex<App>>, err: &Error) -> Result<()> {
	let mut app_locked = app.lock().map_err_string()?;
	app_locked.gui_elements.clear();
	app_locked.gui_elements.push(GuiElement::Label (String::from("Error")));
	app_locked.gui_elements.push(GuiElement::Separator);
	app_locked.gui_elements.push(GuiElement::Label (format!("An error ocurred while running the installer. Please contact Tupelo Workbench with this error message: {err:#?}")));
	app_locked.gui_elements.push(GuiElement::BottomElements (vec!(
		GuiElement::Button {text: String::from("Exit"), was_clicked: false},
	)));
	drop(app_locked);
	loop {
		thread::sleep(Duration::from_millis(100));
		let mut app_locked = app.lock().map_err_string()?;
		let GuiElement::BottomElements (bottom_elements) = &mut app_locked.gui_elements[3] else {return unsynced_err();};
		let GuiElement::Button {was_clicked: just_clicked,  ..} = &mut bottom_elements[0] else {return unsynced_err();};
		let just_clicked = mem::take(just_clicked);
		if just_clicked {
			app_locked.should_close = true;
			break;
		}
	}
	Ok(())
}



pub fn try_run(app: Arc<Mutex<App>>) -> Result<()> {
	
	let mut app_locked = app.lock().map_err_string()?;
	let select_action_rc = Arc::new(Mutex::new(0));
	app_locked.gui_elements = vec!(
		GuiElement::Header (format!("{}  {}  Installer", settings::ADDIN_NAME, settings::ADDIN_VERSION)),
		GuiElement::Separator,
		GuiElement::Label (String::from("What would you like to do?")),
		GuiElement::RadioButton {selected: select_action_rc.clone(), value: 0, text: String::from("Install")},
		GuiElement::RadioButton {selected: select_action_rc.clone(), value: 1, text: String::from("Uninstall")},
		GuiElement::BottomElements (vec!(
			GuiElement::Button {text: String::from("Next"), was_clicked: false}
		)),
	);
	let is_self_update = app_locked.is_self_update;
	drop(app_locked);
	if is_self_update {
		operations::self_update::self_update(app.clone())?;
		let mut app_locked = app.lock().map_err_string()?;
		app_locked.should_close = true;
		return Ok(());
	}
	
	// select action
	loop {
		thread::sleep(Duration::from_millis(100));
		let mut app_locked = app.lock().map_err_string()?;
		let next_button_clicked = {
			let GuiElement::BottomElements (bottom_element) = &mut app_locked.gui_elements[5] else {return unsynced_err();};
			let GuiElement::Button {was_clicked: just_clicked, ..} = &mut bottom_element[0] else {return unsynced_err();};
			mem::take(just_clicked)
		};
		if next_button_clicked {break;}
	}
	
	// run
	let app_locked = app.lock().map_err_string()?;
	let GuiElement::RadioButton {selected, ..} = &app_locked.gui_elements[3] else {return unsynced_err();};
	let selected_action = *selected.lock().map_err_string()?;
	drop(app_locked);
	
	match selected_action {
		0 => {let _ = operations::install::install(app.clone(), None, false)?;}
		1 => {let _ = operations::uninstall::uninstall(app.clone(), None, false, false)?;}
		_ => unreachable!(),
	};
	
	let mut app_locked = app.lock().map_err_string()?;
	app_locked.should_close = true;
	
	Ok(())
}
