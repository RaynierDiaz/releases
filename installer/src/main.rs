#![warn(clippy::all)]
#![feature(duration_constants)]



pub mod settings {
	pub const REPO_OWNER: &str = "RaynierDiaz";
	pub const REPO_NAME: &str = "releases";
	pub const ASSETS_NAME: &str = "Assets.zip";
	pub const INSTALLER_NAME: &str = "Installer.exe";
	pub const LATEST_ASSETS_VERSION: usize = 3;
}



use crate::prelude::*;
use std::{sync::mpsc::{channel, Receiver, Sender}, thread};
use egui::{Layout, Vec2};



pub mod operations;
pub mod gui;
pub mod background_thread;
pub mod data;
pub mod utils;

pub mod prelude {
	pub use crate::{*, data::*};
	pub use std::{fs, path::{Path, PathBuf}};
	pub use std::result::Result as StdResult;
	pub use serde::{Serialize, Deserialize};
	pub use anyhow::*;
}



fn main() {
	
	let mut args = std::env::args();
	args.next();
	let first_arg = args.next();
	if first_arg.as_deref() == Some("--self-update") {
		operations::self_update::self_update();
		return;
	}
	
	let (commands_tx, commands_rx) = channel();
	let (results_tx, results_rx) = channel();
	
	thread::spawn(|| background_thread::run(commands_tx, results_rx));
	
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
		Box::new(|cc| Box::new(App::new(cc, commands_rx, results_tx))),
	);
	if let Err(err) = result {
		utils::fatal_error(format!("Fatal error while running installer: {err}"));
	}
	
}



pub enum GuiCommand {
	
	ShowWorkError (Error),
	
	GoToInstalling,
	ChooseRevitPath,
	
	GoToUninstalling,
	
}

#[derive(Debug)]
pub enum GuiResult {
	
	StartInstall {is_offline: bool},
	RevitPathChosen (PathBuf),
	
	StartUninstall,
	
}
