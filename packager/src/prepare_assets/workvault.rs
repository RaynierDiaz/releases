use crate::*;
use std::{fs::File, path::Path};
use anyhow::*;
use zip::{write::FileOptions, ZipWriter};

pub const EXTENSION_DIR: &str = "C:\\ProgramData\\WorkVault";



pub fn prepare_assets_and_installer(zip: &mut ZipWriter<File>, options: FileOptions<()>, releases_dir: &Path) -> Result<()> {
	
	fs::write(releases_dir.join("installer/src/settings/addin_name.txt"), "WorkVault")?;
	fs::write(releases_dir.join("installer/src/settings/addin_id.txt"), "8110dcaa-2e2f-4696-a89f-cfbee4cc4c27")?;
	fs::write(releases_dir.join("installer/src/settings/installer_url.txt"), "http://158.120.176.164:8080/Installers/WorkVault.exe")?;
	fs::write(releases_dir.join("installer/src/settings/assembly_name.txt"), "WorkVault.dll")?;
	fs::write(releases_dir.join("installer/src/settings/full_class_name.txt"), "WorkVault.RevitApp")?;
	
	// frontend dll
	zip.start_file("Program/WorkVault.dll", options)?;
	let file_contents = fs::read(PathBuf::from(EXTENSION_DIR).join("WorkVault/bin/release/net48/WorkVault.dll"))?;
	zip.write_all(&file_contents)?;
	
	// wpf app
	zip.start_file("Program/WpfWindow.exe", options)?;
	let file_contents = fs::read(PathBuf::from(EXTENSION_DIR).join("WpfWindow/bin/release/net48/WpfWindow.exe"))?;
	zip.write_all(&file_contents)?;
	
	// settings file
	zip.start_file("Program/settings.txt", options)?;
	let file_contents = fs::read(PathBuf::from(EXTENSION_DIR).join("settings.txt"))?;
	zip.write_all(&file_contents)?;
	
	// helix dlls
	zip.start_file("Program/HelixToolkit.dll", options)?;
	let file_contents = fs::read(PathBuf::from(EXTENSION_DIR).join("WpfWindow/bin/release/net48/HelixToolkit.dll"))?;
	zip.write_all(&file_contents)?;
	zip.start_file("Program/HelixToolkit.Wpf.dll", options)?;
	let file_contents = fs::read(PathBuf::from(EXTENSION_DIR).join("WpfWindow/bin/release/net48/HelixToolkit.Wpf.dll"))?;
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
