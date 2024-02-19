#![warn(missing_docs)]
#![doc = include_str!("../readme.md")]
#![doc(html_favicon_url = "https://i.postimg.cc/W3T230zk/favicon.png")]
#![doc(html_logo_url = "https://i.postimg.cc/Vv0HPVwB/logo.png")]

#[cfg(test)]
mod tests;

mod db;
mod func;

pub use db::database;
pub use func::collection;
pub use func::vector;

use pyo3::prelude::*;

#[pymodule]
fn oasysdb(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    collection_module(py, m)?;
    Ok(())
}

fn collection_module(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    let module = PyModule::new(py, "collection")?;
    m.add_submodule(module)?;
    Ok(())
}
