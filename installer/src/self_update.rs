use smart_read::prelude::*;



pub fn self_update() {
	
	// uninstall
	let (uninstall_succeeded, revit_dir)  = crate::uninstall::uninstall(true);
	if !uninstall_succeeded {return;}
	
	// reinstall
	crate::install::install(true, Some(revit_dir));
	
	// delete self
	if let Err(err) = self_replace::self_delete() {
		prompt!(format!("Failed to delete this installer. Please contact Tupelo Workbench with this error message: {err:?} "));
		return;
	}
	
	prompt!("Update is complete, press enter to close the installer");
	
}
