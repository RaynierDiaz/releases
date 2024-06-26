use crate::prelude::*;
use egui::{Color32, Visuals};



pub struct App {
	pub state: AppState,
	pub gui_commands: Receiver<GuiCommand>,
	pub gui_results: Sender<GuiResult>,
}

impl App {
	pub fn new(cc: &eframe::CreationContext<'_>, gui_commands: Receiver<GuiCommand>, gui_results: Sender<GuiResult>) -> Self {
		
		let mut visuals = Visuals::light();
		visuals.override_text_color = Some(Color32::from_gray(0));
		visuals.widgets.inactive.bg_fill = Color32::from_gray(220);
		visuals.widgets.inactive.weak_bg_fill = Color32::from_gray(220);
		visuals.widgets.hovered.bg_fill = Color32::from_gray(200);
		visuals.widgets.active.bg_fill = Color32::from_gray(180);
		cc.egui_ctx.set_visuals(visuals);
		cc.egui_ctx.set_zoom_factor(1.333);
		
		Self {
			state: AppState::ChooseAction {
				selected_action: SelectedAction::Install,
			},
			gui_commands,
			gui_results,
		}
	}
}

impl eframe::App for App {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		let result = gui::app_update(self, ctx, frame);
		if let Err(err) = result {
			utils::fatal_error( format!("Fatal error while running installer: {err}"));
		}
	}
}



pub enum AppState {
	ChooseAction {selected_action: SelectedAction},
	Installing (InstallingState),
	Uninstalling,
	WorkError (Error), // different from fatal error because a fatal error would force close the installer
}



#[derive(PartialEq)]
pub enum SelectedAction {
	Install,
	OfflineInstall,
	Uninstall,
}



pub enum InstallingState {
	None,
	ChooseRevitPath,
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
