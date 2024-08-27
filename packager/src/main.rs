pub mod prepare_assets;



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
	
	let addin = prompt!("Which addin?"; ["WorkHorse", "WorkVault", "WorkVision"]);
	
	println!();
	let version = prompt!("Version number: ");
	fs::write(releases_dir.join("installer/src/settings/version.txt"), version)?;
	
	println!();
	let confirm = prompt!("Checklist: do you have the latest version of the addin repo(s)? "; YesNoInput);
	if !confirm {println!("Please do this first, canceling package"); return Ok(());}
	let confirm = prompt!("Checklist: do you have the latest version of the Releases repo? "; YesNoInput);
	if !confirm {println!("Please do this first, canceling package"); return Ok(());}
	let confirm = prompt!("Checklist: have you built everything in release mode? "; YesNoInput);
	if !confirm {println!("Please do this first, canceling package"); return Ok(());}
	
	let output_dir = releases_dir.join("output");
	if output_dir.exists() {
		fs::remove_dir_all(output_dir).context("Failed to delete output folder")?;
	}
	fs::create_dir(releases_dir.join("output")).context("Failed to create output folder")?;
	
	// assets.zip
	let output_file = File::create(releases_dir.join("output/Assets.zip")).context("Failed to create assets.zip file")?;
	let mut zip = ZipWriter::new(output_file);
	let options: zip::write::FileOptions<()> = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
	
	println!("Starting packaging, creating Assets.zip...");
	match addin.1 {
		"WorkHorse" => prepare_assets::workhorse::prepare_assets_and_installer(&mut zip, options, &releases_dir)?,
		"WorkVault" => prepare_assets::workvault::prepare_assets_and_installer(&mut zip, options, &releases_dir)?,
		"WorkVision" => prepare_assets::workvision::prepare_assets_and_installer(&mut zip, options, &releases_dir)?,
		_ => unreachable!(),
	}
	
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
