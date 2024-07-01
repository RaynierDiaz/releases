pub mod settings {
	pub const EXTENSION_DIR: &str = "C:\\ProgramData\\WorkbenchRevitExt";
}



use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use smart_read::prelude::*;
use anyhow::*;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;



fn main() -> Result<()> {
	
	let mut releases_dir = std::env::current_exe()?;
	while !releases_dir.ends_with("releases") {
		let did_pop = releases_dir.pop();
		if !did_pop {
			return Err(Error::msg("Could not find 'releases' as parent folder"));
		}
	}
	
	let confirm = prompt!("Checklist: do you have the latest version of the Workbench repo? "; YesNoInput);
	if !confirm {println!("Please do this first, canceling package"); return Ok(());}
	let confirm = prompt!("Checklist: do you have the latest version of the Releases repo? "; YesNoInput);
	if !confirm {println!("Please do this first, canceling package"); return Ok(());}
	let confirm = prompt!("Checklist: have you built the frontend in release mode? "; YesNoInput);
	if !confirm {println!("Please do this first, canceling package"); return Ok(());}
	let confirm = prompt!("Checklist: have you built the backend in release mode? "; YesNoInput);
	if !confirm {println!("Please do this first, canceling package"); return Ok(());}
	
	let output_dir = releases_dir.join("output");
	if output_dir.exists() {
		fs::remove_dir_all(output_dir).context("Failed to delete output folder")?;
	}
	fs::create_dir(releases_dir.join("output")).context("Failed to create output folder")?;
	
	println!("Starting packaging, creating Assets.zip...");
	
	// assets.zip
	let output_file = File::create(releases_dir.join("output/Assets.zip")).context("Failed to create assets.zip file")?;
	let mut zip = ZipWriter::new(output_file);
	let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
	
	// frontend dll
	zip.start_file("Frontend.dll", options)?;
	let frontend_file_contents = fs::read(PathBuf::from(settings::EXTENSION_DIR).join("C#Frontend/bin/release/net48/TupeloWorkbenchExt.dll"))?;
	zip.write_all(&frontend_file_contents)?;
	
	// backend dll
	zip.start_file("Backend.dll", options)?;
	let backend_file_contents = fs::read(PathBuf::from(settings::EXTENSION_DIR).join("C#Backend/bin/release/net48/TupeloWorkbenchExt.dll"))?;
	zip.write_all(&backend_file_contents)?;
	
	// addin file
	zip.start_file("TupeloWorkbench.addin", options)?;
	let addin_file_contents = fs::read(releases_dir.join("assets/TupeloWorkbench.addin"))?;
	zip.write_all(&addin_file_contents)?;
	
	// readme file
	zip.start_file("readme.txt", options)?;
	let readme_file_contents = fs::read(releases_dir.join("assets/readme.txt"))?;
	zip.write_all(&readme_file_contents)?;
	
	zip.finish()?;
	
	println!("Done. Building installer...");
	
	// installer's Assets copy
	fs::copy(releases_dir.join("output/Assets.zip"), releases_dir.join("installer/src/Assets.zip")).context("Failed to copy Assets.zip from output to installer/src")?;
	
	let result = Command::new("cargo")
		.current_dir(releases_dir.join("installer"))
		.arg("build")
		.arg("--release")
		.status()
		.context("Failed to run `cargo build --release` on installer")?;
	if !result.success() {println!("Failed to build installer, canceling package"); return Ok(());}
	
	// installer file
	fs::copy(releases_dir.join("installer/target/release/installer.exe"), releases_dir.join("output/Installer.exe")).context("Failed to copy installer exe")?;
	
	println!("Done.");
	
	println!();
	println!();
	println!();
	println!("Finished packaging extension, files to release are in /output.");
	println!("NOTE: make sure you test the addin and installer before releasing");
	println!();
	prompt!("Packing finished, press enter to exit ");
	
	Ok(())
}
