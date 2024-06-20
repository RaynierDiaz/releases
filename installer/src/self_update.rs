pub fn self_update() {
	let uninstall_succeeded  = crate::uninstall::uninstall(true);
	if !uninstall_succeeded {return;}
	crate::install::install(false);
}
