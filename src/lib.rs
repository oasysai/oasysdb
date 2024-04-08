#![warn(missing_docs)]
#![doc = include_str!("../readme.md")]
#![doc(html_favicon_url = "https://i.postimg.cc/W3T230zk/favicon.png")]
#![doc(html_logo_url = "https://i.postimg.cc/Vv0HPVwB/logo.png")]

#[cfg(test)]
mod tests;

mod db;
mod func;

/// Convenience re-exports for the public APIs.
pub mod prelude;

pub use db::database;
pub use func::collection;
pub use func::distance;
pub use func::err;
pub use func::metadata;
pub use func::vector;

#[cfg(feature = "py")]
use pyo3::prelude::*;

#[cfg(feature = "py")]
type Module = fn(Python<'_>, &PyModule) -> PyResult<()>;

#[cfg(feature = "py")]
#[pymodule]
fn oasysdb(py: Python, m: &PyModule) -> PyResult<()> {
    let sys = py.import("sys")?;
    let modules = sys.getattr("modules")?;

    let mods: Vec<(&str, Module)> = vec![
        ("collection", collection_modules),
        ("vector", vector_modules),
        ("database", database_modules),
        ("prelude", prelude_modules),
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

#[cfg(feature = "py")]
#[pymodule]
fn collection_modules(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<collection::Config>()?;
    m.add_class::<collection::Record>()?;
    m.add_class::<collection::Collection>()?;
    m.add_class::<collection::SearchResult>()?;
    Ok(())
}

#[cfg(feature = "py")]
#[pymodule]
fn vector_modules(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<vector::Vector>()?;
    m.add_class::<vector::VectorID>()?;
    Ok(())
}

#[cfg(feature = "py")]
#[pymodule]
fn database_modules(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<database::Database>()?;
    Ok(())
}

#[cfg(feature = "py")]
#[pymodule]
fn prelude_modules(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<collection::Config>()?;
    m.add_class::<collection::Record>()?;
    m.add_class::<collection::Collection>()?;
    m.add_class::<collection::SearchResult>()?;
    m.add_class::<vector::Vector>()?;
    m.add_class::<vector::VectorID>()?;
    m.add_class::<database::Database>()?;
    Ok(())
}
