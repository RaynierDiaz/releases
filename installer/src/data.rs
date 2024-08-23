use crate::prelude::*;
use egui::{Color32, Visuals};



pub type DidFinish<T> = Option<T>;



pub struct OuterApp {
	pub app: Arc<Mutex<App>>,
}

impl OuterApp {
	pub fn new(cc: &eframe::CreationContext<'_>, app: Arc<Mutex<App>>) -> Self {
		
		let mut visuals = Visuals::light();
		visuals.override_text_color = Some(Color32::from_gray(0));
		visuals.widgets.inactive.bg_fill = Color32::from_gray(220);
		visuals.widgets.inactive.weak_bg_fill = Color32::from_gray(220);
		visuals.widgets.hovered.bg_fill = Color32::from_gray(200);
		visuals.widgets.active.bg_fill = Color32::from_gray(180);
		cc.egui_ctx.set_visuals(visuals);
		cc.egui_ctx.set_zoom_factor(1.333);
		
		Self {
			app,
		}
	}
}

impl eframe::App for OuterApp {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		let result = gui::app_update(self, ctx);
		ctx.request_repaint();
		if let Err(err) = result {
			utils::fatal_error( format!("Fatal error while running installer: {err:#?}"));
		}
	}
}



pub struct App {
	pub gui_elements: Vec<GuiElement>,
	pub should_close: bool,
	pub is_self_update: bool,
}

pub enum GuiElement {
	Header (String),
	Separator,
	Label (String),
	TextBox (String),
	Button {text: String, just_clicked: bool},
	RadioButton {selected: Arc<Mutex<usize>>, value: usize, text: String},
	BottomElements (Vec<GuiElement>),
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
