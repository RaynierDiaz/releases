use crate::prelude::*;
use utils::unsynced_err;



pub fn run(inner: Arc<Mutex<InnerApp>>) {
	let result = try_run(inner.clone());
	if let Err(err) = result {
		let result = show_error_message(inner, &err);
		if let Err(msg_err) = result {
			utils::fatal_error(format!("Failed to show error message in installer window. Error message error: {msg_err:#?}\nOriginal error: {err:#?}"));
		}
	}
}



pub fn show_error_message(inner: Arc<Mutex<InnerApp>>, err: &Error) -> Result<()> {
	let mut inner_locked = inner.lock().map_err_string()?;
	inner_locked.gui_elements.clear();
	inner_locked.gui_elements.push(GuiElement::Label (String::from("Error")));
	inner_locked.gui_elements.push(GuiElement::Separator);
	inner_locked.gui_elements.push(GuiElement::Label (format!("An error ocurred while running the installer. Please contact Tupelo Workbench with this error message: {err:#?}")));
	inner_locked.gui_elements.push(GuiElement::BottomElements (vec!(
		GuiElement::Button {text: String::from("Exit"), just_clicked: false},
	)));
	drop(inner_locked);
	loop {
		thread::sleep(Duration::from_millis(100));
		let mut inner_locked = inner.lock().map_err_string()?;
		let GuiElement::BottomElements (bottom_elements) = &mut inner_locked.gui_elements[3] else {return unsynced_err();};
		let GuiElement::Button {just_clicked,  ..} = &mut bottom_elements[0] else {return unsynced_err();};
		let just_clicked = mem::take(just_clicked);
		if just_clicked {
			inner_locked.should_close = true;
			break;
		}
	}
	Ok(())
}



pub fn try_run(inner: Arc<Mutex<InnerApp>>) -> Result<()> {
	
	let inner_locked = inner.lock().map_err_string()?;
	let is_self_update = inner_locked.is_self_update;
	drop(inner_locked);
	if is_self_update {
		operations::self_update::self_update(inner)?;
		return Ok(());
	}
	
	// select action
	loop {
		thread::sleep(Duration::from_millis(100));
		let mut inner_locked = inner.lock().map_err_string()?;
		let next_button_clicked = {
			let GuiElement::BottomElements (bottom_element) = &mut inner_locked.gui_elements[6] else {return unsynced_err();};
			let GuiElement::Button {just_clicked, ..} = &mut bottom_element[0] else {return unsynced_err();};
			mem::take(just_clicked)
		};
		if next_button_clicked {break;}
	}
	
	// run
	let inner_locked = inner.lock().map_err_string()?;
	let GuiElement::RadioButton {selected, ..} = &inner_locked.gui_elements[3] else {return unsynced_err();};
	let selected_action = *selected.lock().map_err_string()?;
	drop(inner_locked);
	
	match selected_action {
		0 => {let _ = operations::install::install(inner.clone(), false, None)?;}
		1 => {let _ = operations::install::install(inner.clone(), true, None)?;}
		2 => {let _ = operations::uninstall::uninstall(inner.clone(), None);}
		_ => unreachable!(),
	};
	
	let mut inner_locked = inner.lock().map_err_string()?;
	inner_locked.should_close = true;
	
	Ok(())
}
