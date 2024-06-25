pub mod settings {
	pub const REPO_OWNER: &str = "RaynierDiaz";
	pub const REPO_NAME: &str = "releases";
	pub const ASSETS_NAME: &str = "Assets.zip";
	pub const INSTALLER_NAME: &str = "Installer.exe";
	pub const LATEST_ASSETS_VERSION: usize = 3;
}



use egui::{Layout, Vec2};

use crate::prelude::*;



pub mod operations;
pub mod data;
pub mod utils;

pub mod prelude {
	pub use crate::{*, data::*};
	pub use std::{fs, path::{Path, PathBuf}};
	pub use std::result::Result as StdResult;
	pub use serde::{Serialize, Deserialize};
	pub use anyhow::*;
	pub use smart_read::prelude::*;
}



fn main() {
	
	let mut args = std::env::args();
	args.next();
	let first_arg = args.next();
	if first_arg.as_deref() == Some("--self-update") {
		operations::self_update::self_update();
		return;
	}
	
	let eframe_options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default()
			.with_inner_size([500.0, 350.0])
			.with_min_inner_size([300.0, 220.0])
			.with_icon(
				eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon 256.png"))
					.expect("Failed to load icon"),
			),
		multisampling: 8,
		..Default::default()
	};
	let result = eframe::run_native(
		"Tupelo Workbench Installer",
		eframe_options,
		Box::new(|cc| Box::new(App::new(cc))),
	);
	if let Err(err) = result {
		utils::show_message_box("Error", format!("Fatal error while running installer: {err}"));
	}
	
}



impl eframe::App for App {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		match &mut self.state {
			
			AppState::ChooseAction {selected_action} => {
				
				let mut new_state = None;
				egui::CentralPanel::default().show(ctx, |ui| {
					
					ui.spacing_mut().item_spacing.y = 5.0;
					ui.heading("Tupelo Workbench Installer");
					ui.spacing_mut().item_spacing.y = 15.0;
					ui.separator();
					ui.label("What would you like to do?");
					ui.spacing_mut().item_spacing.y = 5.0;
					ui.radio_value(selected_action, SelectedAction::Install, "Install");
					ui.radio_value(selected_action, SelectedAction::OfflineInstall, "Offline Install");
					ui.radio_value(selected_action, SelectedAction::Uninstall, "Uninstall");
					
					ui.with_layout(Layout::bottom_up(egui::Align::Max), |ui| {
						ui.spacing_mut().item_spacing.x = 20.0;
						ui.spacing_mut().item_spacing.y = 20.0;
						if ui.add_sized(Vec2::new(90.0, 35.0), egui::Button::new("Start")).clicked() {
							new_state = Some(match selected_action {
								SelectedAction::Install => AppState::Installing {is_offline: false},
								SelectedAction::OfflineInstall => AppState::Installing {is_offline: true},
								SelectedAction::Uninstall => AppState::Uninstalling,
							});
						}
					})
					
				});
				
				if let Some(new_state) = new_state {
					self.state = new_state;
				}
			}
			
			AppState::Installing {is_offline} => {
				
			}
			
			AppState::Uninstalling => {
				
			}
			
		}
	}
}
