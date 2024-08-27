use crate::*;
use std::{fs::File, path::Path};
use anyhow::*;
use zip::{write::FileOptions, ZipWriter};

pub const EXTENSION_DIR: &str = "C:\\ProgramData\\WorkbenchRevitExt";



pub fn prepare_assets_and_installer(zip: &mut ZipWriter<File>, options: FileOptions<()>, releases_dir: &Path) -> Result<()> {
	
	fs::write(releases_dir.join("installer/src/settings/addin_name.txt"), "WorkVision")?;
	fs::write(releases_dir.join("installer/src/settings/addin_id.txt"), "c03cf504-109a-4426-88e6-59dfa2d718ba")?;
	fs::write(releases_dir.join("installer/src/settings/assets_url.txt"), "https://github.com/RaynierDiaz/WorkVision/releases/download/latest/Assets.zip")?;
	fs::write(releases_dir.join("installer/src/settings/installer_url.txt"), "https://github.com/RaynierDiaz/WorkVision/releases/download/latest/Installer.zip")?;
	fs::write(releases_dir.join("installer/src/settings/assembly_name.txt"), "")?;
	fs::write(releases_dir.join("installer/src/settings/full_class_name.txt"), "")?;
	
	// frontend dll
	zip.start_file("Program/Frontend.dll", options)?;
	let file_contents = fs::read(PathBuf::from(EXTENSION_DIR).join("C#Frontend/bin/release/net48/TupeloWorkbenchExt.dll"))?;
	zip.write_all(&file_contents)?;
	
	// backend dll
	zip.start_file("Program/Backend.dll", options)?;
	let file_contents = fs::read(PathBuf::from(EXTENSION_DIR).join("C#Backend/bin/release/net48/TupeloWorkbenchExt.dll"))?;
	zip.write_all(&file_contents)?;
	
	// addin file
	zip.start_file("AddinFile.addin", options)?;
	let file_contents = fs::read(releases_dir.join("assets/AddinFile.addin"))?;
	zip.write_all(&file_contents)?;
	
	// readme file
	zip.start_file("Program/readme.txt", options)?;
	let file_contents = fs::read(releases_dir.join("assets/Program/readme.txt"))?;
	zip.write_all(&file_contents)?;
	
	Ok(())
}
