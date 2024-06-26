use crate::prelude::*;
use crate::operations::uninstallers;
use smart_read::prelude::*;



pub type UninstallSucceeded = bool;

pub fn uninstall(is_self_update: bool) -> (UninstallSucceeded, PathBuf) {
	
	let revit_path = prompt!("Where is Revit located? "; ["C:\\ProgramData\\Autodesk\\Revit"] SimpleValidate (|input| {
		println!("Testing: '{input}'");
		if PathBuf::from(input).exists() {
			StdResult::Ok(())
		} else {
			StdResult::Err(String::from("That path does not exist"))
		}
	}));
	let revit_path = PathBuf::from(revit_path);
	
	let format_version = match get_format_version(&revit_path) {
		StdResult::Ok(v) => v,
		StdResult::Err(err) => {
			prompt!(format!("Error while uninstalling, please contact Tupelo Workbench with this message: {err:?} "));
			return (false, revit_path);
		}
	};
	
	let result = match format_version {
		1 => uninstallers::uninstall_format_1::uninstall(&revit_path),
		2 => uninstallers::uninstall_format_2::uninstall(&revit_path),
		3 => uninstallers::uninstall_format_3::uninstall(&revit_path),
		_ => Err(Error::msg(format!("Unknown format version: {format_version}"))),
	};
	let uninstall_succeeded = match result {
		StdResult::Ok(v) => v,
		StdResult::Err(err) => {
			prompt!(format!("Error while uninstalling, please contact Tupelo Workbench with this message: {err:?} "));
			return (false, revit_path);
		}
	};
	
	if !is_self_update && uninstall_succeeded {
		prompt!("Uninstall successful, press enter to close the installer");
	}
	
	(true, revit_path)
}



pub fn get_format_version(revit_path: &Path) -> Result<usize> {
	let addin_file_path = 'addin_path: {
		for entry in fs::read_dir(revit_path.join("Addins"))? {
			let StdResult::Ok(entry) = entry else {continue;};
			let entry = entry.path();
			let addin_file_path = entry.join("TupeloWorkbench.addin");
			if !addin_file_path.exists() {continue;}
			break 'addin_path addin_file_path;
		}
		return Err(Error::msg("Could not find any .addin files for Tupelo Workbench"));
	};
	let format_version = {
		let addin_contents = fs::read_to_string(addin_file_path)?;
		let format_line =
			addin_contents.lines()
			.find(|line| line.starts_with("<!--FORMAT_VERSION_"))
			.unwrap_or_else(|| {
				println!("Warning: could not find format version within files, assuming installed version if format 1...");
				"<!--FORMAT_VERSION_1"
			});
		let format_num = &format_line[19..];
		format_num.parse::<usize>().map_err(Error::from)?
	};
	Ok(format_version)
}
