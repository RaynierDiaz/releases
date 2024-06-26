use crate::prelude::*;
use utils::unsynced_err;



pub fn run(inner: Arc<Mutex<InnerApp>>) {
	let result = try_run(inner);
	if let Err(err) = result {
		utils::fatal_error(format!("Fatal error while running background thread: {err:?}"));
	}
}



pub fn try_run(inner: Arc<Mutex<InnerApp>>) -> Result<()> {
	
	// select action
	loop {
		thread::sleep(Duration::from_millis(100));
		let inner_locked = inner.lock().map_err_string()?;
		let next_button_clicked = {
			let GuiElement::BottomElements (bottom_element) = &inner_locked.gui_elements[6] else {return unsynced_err();};
			let GuiElement::Button {just_clicked, ..} = &bottom_element[0] else {return unsynced_err();};
			*just_clicked
		};
		if next_button_clicked {break;}
	}
	
	// run
	let inner_locked = inner.lock().map_err_string()?;
	let GuiElement::RadioButton {selected, ..} = &inner_locked.gui_elements[3] else {return unsynced_err();};
	let selected_action = *selected.lock().map_err_string()?;
	drop(inner_locked);
	
	match selected_action {
		0 => {let _ = operations::install::install(inner, false, None, false);}
		1 => {let _ = operations::install::install(inner, true, None, false);}
		2 => {let _ = operations::uninstall::uninstall(false);}
		_ => unreachable!(),
	};
	
	Ok(())
}
