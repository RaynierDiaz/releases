use crate::*;
use std::{fs::File, path::Path};
use anyhow::*;
use zip::{write::FileOptions, ZipWriter};

pub const EXTENSION_DIR: &str = "C:\\ProgramData\\WorkVisionAddin";



pub fn prepare_assets_and_installer(zip: &mut ZipWriter<File>, options: FileOptions<()>, releases_dir: &Path) -> Result<()> {
	
	fs::write(releases_dir.join("installer/src/settings/addin_name.txt"), "WorkVision")?;
	fs::write(releases_dir.join("installer/src/settings/addin_id.txt"), "967efb81-ca19-4a6c-82bb-f11b2bb2c3d1")?;
	fs::write(releases_dir.join("installer/src/settings/installer_url.txt"), "http://158.120.176.164:8080/Installers/WorkVision.exe")?;
	fs::write(releases_dir.join("installer/src/settings/assembly_name.txt"), "Frontend.dll")?;
	fs::write(releases_dir.join("installer/src/settings/full_class_name.txt"), "WorkVisionFrontend.Main")?;
	
	// frontend dll
	zip.start_file("Program/Frontend.dll", options)?;
	let file_contents = fs::read(PathBuf::from(EXTENSION_DIR).join("Frontend/bin/Release/net48/WorkVision.dll"))?;
	zip.write_all(&file_contents)?;
	
	// backend dll
	zip.start_file("Program/Backend.dll", options)?;
	let file_contents = fs::read(PathBuf::from(EXTENSION_DIR).join("Backend/bin/Release/net48/WorkVision.dll"))?;
	zip.write_all(&file_contents)?;
	
	// addin file
	zip.start_file("AddinFile.addin", options)?;
	let file_contents = fs::read(releases_dir.join("assets/AddinFile.addin"))?;
	zip.write_all(&file_contents)?;
	
	// readme file
	zip.start_file("Program/readme.txt", options)?;
	let file_contents = fs::read(releases_dir.join("assets/Program/readme.txt"))?;
	zip.write_all(&file_contents)?;
	
	// eula file
	zip.start_file("Program/eula.txt", options)?;
	let file_contents = fs::read(releases_dir.join("assets/Program/eula.txt"))?;
	zip.write_all(&file_contents)?;
	
	Ok(())
}
