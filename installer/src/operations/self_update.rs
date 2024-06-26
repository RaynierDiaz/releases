use crate::prelude::*;
use smart_read::prelude::*;



pub fn self_update(inner: Arc<Mutex<InnerApp>>) -> Result<DidFinish<()>> {
	
	// uninstall
	let did_finish = crate::operations::uninstall::uninstall(inner.clone(), None)?;
	let Some(revit_path) = did_finish else {return Ok(None);};
	
	// reinstall
	crate::operations::install::install(inner, true, Some(revit_path))?;
	
	// delete self
	if let Err(err) = self_replace::self_delete() {
		prompt!(format!("Failed to delete this installer. Please contact Tupelo Workbench with this error message: {err:?} "));
		return Ok(None);
	}
	
	prompt!("Update is complete, press enter to close the installer");
	
	Ok(Some(()))
}
