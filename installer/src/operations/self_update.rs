use crate::prelude::*;
use smart_read::prelude::*;



pub fn self_update(inner: Arc<Mutex<InnerApp>>) -> Result<()> {
	
	// uninstall
	let (uninstall_succeeded, revit_path) = crate::operations::uninstall::uninstall(inner.clone(), true, None)?;
	if !uninstall_succeeded {return Ok(());}
	
	// reinstall
	crate::operations::install::install(inner, true, Some(revit_path))?;
	
	// delete self
	if let Err(err) = self_replace::self_delete() {
		prompt!(format!("Failed to delete this installer. Please contact Tupelo Workbench with this error message: {err:?} "));
		return Ok(());
	}
	
	prompt!("Update is complete, press enter to close the installer");
	
	Ok(())
}
