use std::fs;
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;
use smart_read::prelude::*;
use anyhow::*;



pub fn uninstall() {
	
	let revit_dir = prompt!("Where is Revit located? "; ["C:\\ProgramData\\Autodesk\\Revit"] SimpleValidate (|input| {
		println!("Testing: '{input}'");
		if PathBuf::from(input).exists() {
			StdResult::Ok(())
		} else {
			StdResult::Err(String::from("That path does not exist"))
		}
	}));
	let revit_dir = PathBuf::from(revit_dir);
	
	println!("Removing files...");
	
	let did_delete_extension_folder = match delete_extension_folder(&revit_dir) {
		StdResult::Ok(v) => v,
		StdResult::Err(err) => {
			println!("Failed to delete extension folder, usually located at C:\\ProgramData\\TupeloWorkbenchExt. Please contact Tupelo Workbench with this error message: {err:?}.");
			return;
		}
	};
	if !did_delete_extension_folder {
		let exit = prompt!("The extension folder was not deleted, would you like to cancel the uninstall? "; [true] YesNoInput);
		if exit {
			println!("Affirmed, canceling uninstall.");
			return;
		}
		println!("Affirmed, continuing uninstall...");
	}
	if let Err(err) = delete_addin_files(&revit_dir) {
		println!("Failed to delete addin files, usually located at C:\\ProgramData\\Autodesk\\Revit\\Addins\\___\\TupeloWorkbench.addin. Please contact Tupelo Workbench with this error message: {err:?}");
	}
	
	println!("Done.");
	
	prompt!("Uninstall successful, press enter to close the installer");
	
}



pub type DidDeleteExtensionFolder = bool;

pub fn delete_extension_folder(revit_dir: &Path) -> Result<DidDeleteExtensionFolder> {
	let addins_path = revit_dir.join("Addins");
	let addin_file_path =
		fs::read_dir(&addins_path).context(format!("Attempted to read contents of {addins_path:?}"))?
		.find(|entry| {
			let StdResult::Ok(entry) = entry else {return false;};
			entry.path().join("TupeloWorkbench.addin").exists()
		});
	let Some(addin_file_path) = addin_file_path else {
		println!("Warning: Could not find any .addin file for this extension, so the path of the dlls is unknown and cannot be removed automatically. They are usually stored at C:\\ProgramData\\TupeloWorkbenchExt, though the install path could be customized");
		return Ok(false);
	};
	let addin_file_path = addin_file_path?.path().join("TupeloWorkbench.addin");
	
	let addin_contents = fs::read_to_string(&addin_file_path).context(format!("Attempted to read contents of {addin_file_path:?}"))?;
	let assembly_line = addin_contents.lines()
		.find(|line| {
			line.trim().starts_with("<Assembly>")
		})
		.expect("Could not find <Assembly> in addin file.")
		.trim();
	
	let extension_path = PathBuf::from(&assembly_line.trim()[10..assembly_line.len()-23]);
	let confirm = prompt!(format!("The detected extension path is \"{extension_path:?}\", does that sound right? "); [true] YesNoInput);
	if !confirm {
		println!("Affirmed, said path will not be deleted.");
		return Ok(false);
	}
	
	fs::remove_dir_all(extension_path)?;
	
	Ok(true)
}



pub fn delete_addin_files(revit_dir: &Path) -> Result<()> {
	let addins_path = revit_dir.join("Addins");
	for entry in fs::read_dir(&addins_path).context(format!("Attempted to read contents of {addins_path:?}"))? {
		let StdResult::Ok(entry) = entry else {continue;};
		let addin_file_path = entry.path().join("TupeloWorkbench.addin");
		if addin_file_path.exists() {
			fs::remove_file(addin_file_path)?;
		}
	}
	Ok(())
}
