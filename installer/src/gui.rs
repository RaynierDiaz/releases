use crate::prelude::*;
use egui::{Layout, Ui, Vec2};



pub fn app_update(app: &mut OuterApp, ctx: &egui::Context) -> Result<()> {
	let mut app_locked = app.app.lock().map_err_string()?;
	if app_locked.should_close {
		ctx.send_viewport_cmd(egui::ViewportCommand::Close);
	}
	let result = egui::CentralPanel::default().show(ctx, |ui| {
		ui.spacing_mut().item_spacing.y = 5.0;
		for gui_element in &mut app_locked.gui_elements {
			draw_gui_element(gui_element, ui)?;
		}
		Ok(())
	});
	if result.inner.is_err() {return result.inner};
	Ok(())
}



pub fn draw_gui_element(gui_element: &mut GuiElement, ui: &mut Ui) -> Result<()> {
	match gui_element {
		
		GuiElement::Header (text) => {let _ = ui.heading(&**text);}
		
		GuiElement::Separator => {
			ui.spacing_mut().item_spacing.y = 15.0;
			let _ = ui.separator();
			ui.spacing_mut().item_spacing.y = 5.0;
		}
		
		GuiElement::Label (text) => {let _ = ui.label(&**text);}
		
		GuiElement::ScrollingLabel (text) => {
			ui.group(|ui| {
				ui.set_max_height(ui.available_size().y - 40.0);
				egui::scroll_area::ScrollArea::vertical().show(ui, |ui| {
					let _ = ui.label(&**text);
				});
			});
		}
		
		GuiElement::TextBox (text) => {let _ = ui.text_edit_singleline(text);}
		
		GuiElement::Button {text, was_clicked} => {
			ui.spacing_mut().item_spacing.x = 20.0;
			ui.spacing_mut().item_spacing.y = 20.0;
			*was_clicked |= ui.add_sized(Vec2::new(80.0, 30.0), egui::Button::new(&**text)).clicked();
		}
		
		GuiElement::RadioButton {selected, value, text} => {
			let old_value = *selected.lock().map_err_string()?;
			let mut new_value = old_value;
			ui.radio_value(&mut new_value, *value, &**text);
			if new_value != old_value {*selected.lock().map_err_string()? = new_value;}
		}
		
		GuiElement::BottomElements (elements) => {
			let result = ui.with_layout(Layout::right_to_left(egui::Align::Max), |ui| {
				for gui_element in elements {
					draw_gui_element(gui_element, ui)?;
				}
				Ok(())
			});
			if result.inner.is_err() {return result.inner;}
		}
		
	}
	Ok(())
}
