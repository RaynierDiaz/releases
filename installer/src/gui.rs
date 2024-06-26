use crate::prelude::*;



pub fn app_update(app: &mut App, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Result<()> {
	
	for command in app.gui_commands.try_iter() {
		match command {
			
			GuiCommand::ShowWorkError (err) => {
				app.state = AppState::WorkError (err);
			}
			
			GuiCommand::GoToInstalling => {
				app.state = AppState::Installing (InstallingState::None);
			}
			
			GuiCommand::GoToUninstalling => {
				app.state = AppState::Uninstalling;
			}
			
			GuiCommand::ChooseRevitPath => {
				app.state = AppState::Installing (InstallingState::ChooseRevitPath);
			}
			
		}
	}
	
	match &mut app.state {
		
		
		
		AppState::ChooseAction {selected_action} => {
			
			let result = egui::CentralPanel::default().show(ctx, |ui| {
				
				ui.spacing_mut().item_spacing.y = 5.0;
				ui.heading("Tupelo Workbench Installer");
				ui.spacing_mut().item_spacing.y = 15.0;
				ui.separator();
				ui.label("What would you like to do?");
				ui.spacing_mut().item_spacing.y = 5.0;
				ui.radio_value(selected_action, SelectedAction::Install, "Install");
				ui.radio_value(selected_action, SelectedAction::OfflineInstall, "Offline Install");
				ui.radio_value(selected_action, SelectedAction::Uninstall, "Uninstall");
				
				let result = ui.with_layout(Layout::bottom_up(egui::Align::Max), |ui| {
					ui.spacing_mut().item_spacing.x = 20.0;
					ui.spacing_mut().item_spacing.y = 20.0;
					if ui.add_sized(Vec2::new(90.0, 35.0), egui::Button::new("Start")).clicked() {
						match selected_action {
							SelectedAction::Install => app.gui_results.send(GuiResult::StartInstall {is_offline: false})?,
							SelectedAction::OfflineInstall => app.gui_results.send(GuiResult::StartInstall {is_offline: true})?,
							SelectedAction::Uninstall => app.gui_results.send(GuiResult::StartUninstall)?,
						}
					}
					Ok(())
				});
				if result.inner.is_err() {return result.inner;}
				
				Ok(())
			});
			if result.inner.is_err() {return result.inner;}
			
		}
		
		
		
		AppState::Installing (installing_state) => {
			
			let result = egui::CentralPanel::default().show(ctx, |ui| {
				
				ui.spacing_mut().item_spacing.y = 5.0;
				ui.heading("Installing");
				ui.spacing_mut().item_spacing.y = 15.0;
				ui.separator();
				
				match installing_state {
					InstallingState::None => {
						ui.label("Please wait...");
					}
					InstallingState::ChooseRevitPath => {
						ui.label("Choose Revit Path");
						
					}
				}
				
				Ok(())
			});
			if result.inner.is_err() {return result.inner;}
			
		}
		
		
		
		AppState::Uninstalling => {
			
		}
		
		
		
		AppState::WorkError (err) => {
			let result = egui::CentralPanel::default().show(ctx, |ui| {
				
				ui.heading("Error ocurred while running installer");
				
				ui.label(format!("Please contact Tupelo Workbench with this error message: {err:?}"));
				
				let result = ui.with_layout(Layout::bottom_up(egui::Align::Max), |ui| {
					ui.spacing_mut().item_spacing.x = 20.0;
					ui.spacing_mut().item_spacing.y = 20.0;
					if ui.add_sized(Vec2::new(90.0, 35.0), egui::Button::new("Close")).clicked() {
						ctx.send_viewport_cmd(egui::ViewportCommand::Close);
					}
					Ok(())
				});
				if result.inner.is_err() {return result.inner;}
				
				Ok(())
			});
			if result.inner.is_err() {return result.inner;}
		}
		
		
		
	}
	Ok(())
}
