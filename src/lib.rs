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
