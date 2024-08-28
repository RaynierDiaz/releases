#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#![warn(clippy::all)]
#![feature(duration_constants)]



use crate::prelude::*;
use std::thread;



pub(crate) mod operations;
pub(crate) mod gui;
pub(crate) mod background_thread;
pub(crate) mod data;
pub(crate) mod utils;
pub(crate) mod custom_impls;

pub(crate) mod settings {
	pub const ADDIN_NAME: &str = include_str!("settings/addin_name.txt");
	pub const ADDIN_ID: &str = include_str!("settings/addin_id.txt");
	pub const ADDIN_VERSION: &str = include_str!("settings/addin_version.txt");
	pub const VENDOR_DESCRIPTION: &str = include_str!("settings/vendor_description.txt");
	pub const INSTALLER_URL: &str = include_str!("settings/installer_url.txt");
	pub const ASSEMBLY_NAME: &str = include_str!("settings/assembly_name.txt");
	pub const FULL_CLASS_NAME: &str = include_str!("settings/full_class_name.txt");
}

pub(crate) mod prelude {
	pub(crate) use crate::{*, data::*, custom_impls::*};
	pub(crate) use std::{mem, fs, path::{Path, PathBuf}, sync::{Arc, Mutex}, time::Duration};
	pub(crate) use std::result::Result as StdResult;
	pub(crate) use anyhow::*;
}



fn main() {
	
	let mut args = std::env::args();
	args.next();
	let first_arg = args.next();
	let is_self_update = first_arg.as_deref() == Some("--self-update");
	
	let app = Arc::new(Mutex::new(App {
		gui_elements: vec!(),
		should_close: false,
		is_self_update,
	}));
	
	let app_clone = app.clone();
	thread::spawn(|| background_thread::run(app_clone));
	
	let eframe_options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default()
			.with_inner_size([500.0, 350.0])
			.with_min_inner_size([350.0, 200.0])
			.with_icon(
				eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon 256.png"))
					.expect("Failed to load icon"),
			),
		multisampling: 8,
		centered: true,
		..Default::default()
	};
	let result = eframe::run_native(
		"Tupelo Workbench Installer",
		eframe_options,
		Box::new(|cc| StdResult::Ok(Box::new(OuterApp::new(cc, app)))),
	);
	if let Err(err) = result {
		utils::fatal_error(format!("Fatal error while running installer: {err}"));
	}
	
}
