use crate::prelude::*;
use egui::{Layout, Ui, Vec2};



pub fn app_update(app: &mut App, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Result<()> {
	let mut inner = app.inner.lock().map_err_string()?;
	let result = egui::CentralPanel::default().show(ctx, |ui| {
		ui.spacing_mut().item_spacing.y = 5.0;
		for gui_element in &mut inner.gui_elements {
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
		
		GuiElement::Button {text, just_clicked: was_clicked} => {
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
			let result = ui.with_layout(Layout::bottom_up(egui::Align::Max), |ui| {
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
