use crate::prelude::*;
use egui::{Color32, Visuals};



pub struct App {
	pub state: AppState,
}

impl App {
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		
		let mut visuals = Visuals::light();
		visuals.override_text_color = Some(Color32::from_gray(0));
		cc.egui_ctx.set_visuals(visuals);
		cc.egui_ctx.set_zoom_factor(1.333);
		
		Self {
			state: AppState::ChooseAction {
				selected_action: SelectedAction::Install,
			},
		}
	}
}



pub enum AppState {
	ChooseAction {selected_action: SelectedAction},
}



#[derive(PartialEq)]
pub enum SelectedAction {
	Install,
	OfflineInstall,
	Uninstall,
}



#[derive(Deserialize, Debug)]
pub struct Asset {
	pub name: String,
	pub browser_download_url: String,
}

#[derive(Deserialize)]
pub struct Release {
	pub assets: Vec<Asset>,
}
