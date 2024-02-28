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

type Module = fn(Python<'_>, &PyModule) -> PyResult<()>;

#[pymodule]
fn oasysdb(py: Python, m: &PyModule) -> PyResult<()> {
    let sys = py.import("sys")?;
    let modules = sys.getattr("modules")?;

    let mods: Vec<(&str, Module)> = vec![
        ("collection", collection_modules),
        ("vector", vector_modules),
        ("database", database_modules),
    ];

    for (name, module) in mods {
        let full_name = format!("oasysdb.{}", name);
        let pymod = PyModule::new(py, &full_name)?;
        module(py, pymod)?;
        m.add(name, pymod)?;
        modules.set_item(full_name, pymod)?;
    }

    Ok(())
}

#[pymodule]
fn collection_modules(_py: Python, m: &PyModule) -> PyResult<()> {
    Ok(())
}

#[pymodule]
fn vector_modules(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<vector::Vector>()?;
    Ok(())
}

#[pymodule]
fn database_modules(_py: Python, m: &PyModule) -> PyResult<()> {
    Ok(())
}
