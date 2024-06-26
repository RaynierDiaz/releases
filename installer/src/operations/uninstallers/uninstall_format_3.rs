use crate::prelude::*;
use crate::operations::uninstall::UninstallSucceeded;
use smart_read::prelude::*;



pub fn uninstall(revit_path: &Path) -> Result<UninstallSucceeded> {
	
	let did_delete_extension_folder = match delete_extension_folder(revit_path) {
		StdResult::Ok(v) => v,
		StdResult::Err(err) => {
			prompt!(format!("Failed to delete extension folder, usually located at C:\\ProgramData\\TupeloWorkbenchExt. Please contact Tupelo Workbench with this error message: {err:#?}. "));
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
	if let Err(err) = delete_addin_files(revit_path) {
		prompt!(format!("Failed to delete addin files, usually located at C:\\ProgramData\\Autodesk\\Revit\\Addins\\___\\TupeloWorkbench.addin. Please contact Tupelo Workbench with this error message: {err:#?} "));
		return Ok(false);
	}
	
	Ok(true)
}



pub type DidDeleteExtensionFolder = bool;

pub fn delete_extension_folder(revit_path: &Path) -> Result<DidDeleteExtensionFolder> {
	
	println!("Removing extension files...");
	loop {
		match fs::remove_dir_all(&revit_path.join("Tupelo Workbench")) {
			StdResult::Ok(()) => break,
			StdResult::Err(err) => {
				println!();
				println!("Failed to delete files (NOTE: If Revit is open, please close it and wait a few seconds before continuing). Error message: {err:#?}");
				let result = prompt!("Would you like to retry? "; [true] YesNoInput);
				if !result {return Ok(false);}
			}
		};
	}
	println!("Done.");
	
	Ok(true)
}



pub fn delete_addin_files(revit_path: &Path) -> Result<()> {
	println!("Removing .addin files...");
	let addins_path = revit_path.join("Addins");
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
