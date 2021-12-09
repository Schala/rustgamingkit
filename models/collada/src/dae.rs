pub mod import {
	use thiserror::Error;

	use xml::reader::{
		EventReader,
		XmlEvent
	};

	#[derive(Error, Debug, PartialEq)]
	pub enum ColladaImportError {

	}
}
