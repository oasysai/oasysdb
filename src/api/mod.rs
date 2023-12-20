use crate::db::database::*;
use std::collections::HashMap;

mod utils;
mod values;

pub use utils::*;
pub use values::*;

pub type StringMap = HashMap<&'static str, &'static str>;

// Not the recommended way to do this as this requires manually
// serializing the response. Be careful with this approach.
#[derive(Responder)]
#[response(content_type = "json")]
pub struct Response(String);
