use crate::*;
use std::{fs::File, path::Path};
use anyhow::*;
use walkdir::WalkDir;
use zip::{write::FileOptions, ZipWriter};

pub const EXTENSION_DIR: &str = "C:\\ProgramData\\WorkVault";



pub fn prepare_assets_and_installer(zip: &mut ZipWriter<File>, options: FileOptions<()>, releases_dir: &Path) -> Result<()> {
	
	fs::write(releases_dir.join("installer/src/settings/addin_name.txt"), "WorkVault")?;
	fs::write(releases_dir.join("installer/src/settings/addin_id.txt"), "8110dcaa-2e2f-4696-a89f-cfbee4cc4c27")?;
	fs::write(releases_dir.join("installer/src/settings/assets_url.txt"), "https://github.com/RaynierDiaz/WorkVault/releases/download/latest/Assets.zip")?;
	fs::write(releases_dir.join("installer/src/settings/installer_url.txt"), "https://github.com/RaynierDiaz/WorkVault/releases/download/latest/Installer.zip")?;
	fs::write(releases_dir.join("installer/src/settings/assembly_name.txt"), "WorkVault.dll")?;
	fs::write(releases_dir.join("installer/src/settings/full_class_name.txt"), "WorkVault.RevitApp")?;
	
	// frontend dll
	zip.start_file("Program/WorkVault.dll", options)?;
	let frontend_file_contents = fs::read(PathBuf::from(EXTENSION_DIR).join("WorkVault/bin/release/net48/WorkVault.dll"))?;
	zip.write_all(&frontend_file_contents)?;
	
	// wpf app
	zip.start_file("Program/WpfWindow.exe", options)?;
	let apf_file_contents = fs::read(PathBuf::from(EXTENSION_DIR).join("WpfWindow/bin/release/net48/WpfWindow.exe"))?;
	zip.write_all(&apf_file_contents)?;
	
	// assets
	for entry in WalkDir::new(PathBuf::from(EXTENSION_DIR).join("asset")) {
		let entry = entry?;
		let entry = entry.path();
		if entry.is_dir() {continue;}
		let zip_path = PathBuf::from("Program").join(entry.strip_prefix(EXTENSION_DIR)?);
		zip.start_file(zip_path.to_string_lossy(), options)?;
		let asset_file_contents = fs::read(entry)?;
		zip.write_all(&asset_file_contents)?;
	}
	
	// helix dlls
	zip.start_file("Program/HelixToolkit.dll", options)?;
	let helix_file_contents = fs::read(PathBuf::from(EXTENSION_DIR).join("WpfWindow/bin/release/net48/HelixToolkit.dll"))?;
	zip.write_all(&helix_file_contents)?;
	zip.start_file("Program/HelixToolkit.Wpf.dll", options)?;
	let helix_wpf_file_contents = fs::read(PathBuf::from(EXTENSION_DIR).join("WpfWindow/bin/release/net48/HelixToolkit.Wpf.dll"))?;
	zip.write_all(&helix_wpf_file_contents)?;
	
	// addin file
	zip.start_file("AddinFile.addin", options)?;
	let addin_file_contents = fs::read(releases_dir.join("assets/AddinFile.addin"))?;
	zip.write_all(&addin_file_contents)?;
	
	// readme file
	zip.start_file("Program/readme.txt", options)?;
	let readme_file_contents = fs::read(releases_dir.join("assets/Program/readme.txt"))?;
	zip.write_all(&readme_file_contents)?;
	
	Ok(())
}
