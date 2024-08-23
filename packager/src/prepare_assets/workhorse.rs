use crate::*;
use std::{fs::File, path::Path};
use anyhow::*;
use zip::{write::FileOptions, ZipWriter};

pub const EXTENSION_DIR: &str = "C:\\ProgramData\\WorkbenchRevitExt";



pub fn prepare_assets_and_installer(zip: &mut ZipWriter<File>, options: FileOptions<()>, releases_dir: &Path) -> Result<()> {
	
	fs::write(releases_dir.join("installer/src/settings/addin_name.txt"), "WorkHorse")?;
	fs::write(releases_dir.join("installer/src/settings/addin_id.txt"), "194a4656-92f2-4a74-8703-745b37ae3b13")?;
	fs::write(releases_dir.join("installer/src/settings/assets_url.txt"), "https://github.com/RaynierDiaz/WorkHorse/releases/download/latest/Assets.zip")?;
	fs::write(releases_dir.join("installer/src/settings/installer_url.txt"), "https://github.com/RaynierDiaz/WorkHorse/releases/download/latest/Installer.zip")?;
	fs::write(releases_dir.join("installer/src/settings/assembly_name.txt"), "Frontend.dll")?;
	fs::write(releases_dir.join("installer/src/settings/full_class_name.txt"), "TupeloWorkbenchExt.MainApp")?;
	
	// frontend dll
	zip.start_file("Program/Frontend.dll", options)?;
	let frontend_file_contents = fs::read(PathBuf::from(EXTENSION_DIR).join("C#Frontend/bin/release/net48/TupeloWorkbenchExt.dll"))?;
	zip.write_all(&frontend_file_contents)?;
	
	// backend dll
	zip.start_file("Program/Backend.dll", options)?;
	let backend_file_contents = fs::read(PathBuf::from(EXTENSION_DIR).join("C#Backend/bin/release/net48/TupeloWorkbenchExt.dll"))?;
	zip.write_all(&backend_file_contents)?;
	
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
