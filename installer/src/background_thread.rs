use crate::prelude::*;



pub fn run(commands: Sender<GuiCommand>, results: Receiver<GuiResult>) {
	let result = try_run(&commands, results);
	if let Err(err) = result {
		commands.send(GuiCommand::ShowWorkError (err)).unwrap();
	}
}



pub fn try_run(commands: &Sender<GuiCommand>, results: Receiver<GuiResult>) -> Result<()> {
	match results.recv()? {
		
		
		
		GuiResult::StartInstall {is_offline} => {
			let _ = operations::install::install(is_offline, None, false, commands, results);
		}
		
		
		
		GuiResult::StartUninstall => {
			
		}
		
		
		
		result => commands.send(GuiCommand::ShowWorkError (Error::msg(format!("Invalid gui result received: {result:?}"))))?,
	}
	Ok(())
}
