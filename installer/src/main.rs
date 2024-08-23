#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#![warn(clippy::all)]
#![feature(duration_constants)]

pub const LATEST_ASSETS_VERSION: usize = 1;



use crate::prelude::*;
use std::thread;



pub mod operations;
pub mod gui;
pub mod background_thread;
pub mod data;
pub mod utils;
pub mod custom_impls;

pub mod settings {
	pub const ADDIN_NAME: &str = include_str!("settings/addin_name.txt");
	pub const ADDIN_ID: &str = include_str!("settings/addin_id.txt");
	pub const VENDOR_DESCRIPTION: &str = include_str!("settings/vendor_description.txt");
	pub const ASSETS_URL: &str = include_str!("settings/assets_url.txt");
	pub const INSTALLER_URL: &str = include_str!("settings/installer_url.txt");
	pub const ASSEMBLY_NAME: &str = include_str!("settings/assembly_name.txt");
	pub const FULL_CLASS_NAME: &str = include_str!("settings/full_class_name.txt");
}

pub mod prelude {
	pub use crate::{*, data::*, custom_impls::*};
	pub use std::{mem, fs, path::{Path, PathBuf}, sync::{Arc, Mutex}, rc::Rc, time::Duration};
	pub use std::result::Result as StdResult;
	pub use serde::{Serialize, Deserialize};
	pub use anyhow::*;
}



fn main() {
	
	let mut args = std::env::args();
	args.next();
	let first_arg = args.next();
	let is_self_update = first_arg.as_deref() == Some("--self-update");
	
	let select_action_rc = Arc::new(Mutex::new(0));
	let app = Arc::new(Mutex::new(App {
		gui_elements: vec!(
			GuiElement::Header (format!("{} Installer", settings::ADDIN_NAME)),
			GuiElement::Separator,
			GuiElement::Label (String::from("What would you like to do?")),
			GuiElement::RadioButton {selected: select_action_rc.clone(), value: 0, text: String::from("Install (uses latest version)")},
			GuiElement::RadioButton {selected: select_action_rc.clone(), value: 1, text: String::from("Offline Install")},
			GuiElement::RadioButton {selected: select_action_rc.clone(), value: 2, text: String::from("Uninstall")},
			GuiElement::BottomElements (vec!(
				GuiElement::Button {text: String::from("Next"), just_clicked: false}
			)),
		),
		should_close: false,
		is_self_update,
	}));
	
	let app_clone = app.clone();
	thread::spawn(|| background_thread::run(app_clone));
	
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
		Box::new(|cc| Box::new(OuterApp::new(cc, app))),
	);
	if let Err(err) = result {
		utils::fatal_error(format!("Fatal error while running installer: {err:#?}"));
	}
	
}
