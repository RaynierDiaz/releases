use crate::prelude::*;
use std::fmt::Debug;



pub trait StdResultFns<T> {
	fn map_err_string(self) -> Result<T>;
}

impl<T, E: Debug> StdResultFns<T> for StdResult<T, E> {
	fn map_err_string(self) -> Result<T> {
		self.map_err(|err| Error::msg(format!("{err:?}")))
	}
}
