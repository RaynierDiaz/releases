use std::{fs::{self, File}, path::{Path, PathBuf}};
use fs2::FileExt;
use smart_read::prelude::*;
use anyhow::*;
use std::result::Result as StdResult;
use crate::uninstall::UninstallSucceeded;



pub fn uninstall(revit_dir: PathBuf) -> Result<UninstallSucceeded> {
	
	let did_delete_extension_folder = match delete_extension_folder(&revit_dir) {
		StdResult::Ok(v) => v,
		StdResult::Err(err) => {
			prompt!(format!("Failed to delete extension folder, usually located at C:\\ProgramData\\TupeloWorkbenchExt. Please contact Tupelo Workbench with this error message: {err:?}. "));
			return Ok(false);
		}
	};
	if !did_delete_extension_folder {
		let exit = prompt!("The extension folder was not deleted, would you like to cancel the uninstall? "; [true] YesNoInput);
		if exit {
			prompt!("Affirmed, canceling uninstall.");
			return Ok(false);
		}
		println!("Affirmed, continuing uninstall...");
	}
	if let Err(err) = delete_addin_files(&revit_dir) {
		prompt!(format!("Failed to delete addin files, usually located at C:\\ProgramData\\Autodesk\\Revit\\Addins\\___\\TupeloWorkbench.addin. Please contact Tupelo Workbench with this error message: {err:?} "));
		return Ok(false);
	}
	
	Ok(true)
}



pub type DidDeleteExtensionFolder = bool;

pub fn delete_extension_folder(revit_dir: &Path) -> Result<DidDeleteExtensionFolder> {
	
	// read addins file
	let addins_path = revit_dir.join("Addins");
	let addin_file_path =
		fs::read_dir(&addins_path).context(format!("Attempted to read contents of {addins_path:?}"))?
		.find(|entry| {
			let StdResult::Ok(entry) = entry else {return false;};
			entry.path().join("TupeloWorkbench.addin").exists()
		});
	let Some(addin_file_path) = addin_file_path else {
		prompt!("Warning: Could not find any .addin file for this extension, so the path of the dlls is unknown and cannot be removed automatically. They are usually stored at C:\\ProgramData\\TupeloWorkbenchExt, though the install path could be customized");
		return Ok(false);
	};
	let addin_file_path = addin_file_path?.path().join("TupeloWorkbench.addin");
	
	// get extension path
	let addin_contents = fs::read_to_string(&addin_file_path).context(format!("Attempted to read contents of {addin_file_path:?}"))?;
	let assembly_line = addin_contents.lines()
		.find(|line| {
			line.trim().starts_with("<Assembly>")
		})
		.expect("Could not find \"<Assembly>\" line in addin file.")
		.trim();
	
	let extension_path = PathBuf::from(&assembly_line.trim()[10..assembly_line.len()-23]);
	let confirm = prompt!(format!("The detected extension path is {extension_path:?}, does that sound right? "); [true] YesNoInput);
	if !confirm {
		prompt!("Affirmed, said path will not be deleted.");
		return Ok(false);
	}
	
	// ensure file is unlocked
	let frontend_file = File::open(extension_path.join("Frontend.dll")).context("Attempted to open Frontend.dll")?;
	while is_file_locked(&frontend_file).context("Attempted to check if Frontend.dll is locked")? {
		let input = prompt!("Revit seems to be open, please close Revit or any other programs that locking this extension (or enter \"stop\" to stop)");
		if input.eq_ignore_ascii_case("stop") {
			prompt!("Affirmed, stopping uninstall.");
			return Ok(false);
		}
	}
	
	// delete
	println!("Removing extension files...");
	fs::remove_dir_all(extension_path)?;
	println!("Done.");
	
	Ok(true)
}



pub fn delete_addin_files(revit_dir: &Path) -> Result<()> {
	println!("Removing .addin files...");
	let addins_path = revit_dir.join("Addins");
	for entry in fs::read_dir(&addins_path).context(format!("Attempted to read contents of {addins_path:?}"))? {
		let StdResult::Ok(entry) = entry else {continue;};
		let addin_file_path = entry.path().join("TupeloWorkbench.addin");
		if addin_file_path.exists() {
			fs::remove_file(addin_file_path)?;
		}
	}
	println!("Done.");
	Ok(())
}

fn is_file_locked(file: &File) -> Result<bool> {
	match file.try_lock_exclusive() {
		StdResult::Ok(_) => {
			file.unlock().context("Attempted to undo lock on Frontend.dll")?;
			Ok(false)
		}
		StdResult::Err(_) => Ok(true),
	}
}
