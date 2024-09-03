use crate::*;
use std::{fs::File, path::Path};
use anyhow::*;
use zip::{write::FileOptions, ZipWriter};

pub const EXTENSION_DIR: &str = "C:\\ProgramData\\WorkbenchRevitExt";



pub fn prepare_assets_and_installer(zip: &mut ZipWriter<File>, options: FileOptions<()>, releases_dir: &Path) -> Result<()> {
	
	fs::write(releases_dir.join("installer/src/settings/addin_name.txt"), "WorkHorse")?;
	fs::write(releases_dir.join("installer/src/settings/addin_id.txt"), "194a4656-92f2-4a74-8703-745b37ae3b13")?;
	fs::write(releases_dir.join("installer/src/settings/installer_url.txt"), "http://158.120.176.164:8080/Installers/WorkHorse.exe")?;
	fs::write(releases_dir.join("installer/src/settings/assembly_name.txt"), "Frontend.dll")?;
	fs::write(releases_dir.join("installer/src/settings/full_class_name.txt"), "TupeloWorkbenchExt.MainApp")?;
	
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
	
	// eula file
	zip.start_file("Program/eula.txt", options)?;
	let file_contents = fs::read(releases_dir.join("assets/Program/eula.txt"))?;
	zip.write_all(&file_contents)?;
	
	Ok(())
}
