//! Generation of present or absent, valid or invalid key value pairs.

mod occupation;
mod site;

pub use occupation::Occupation;
pub use site::{IntoCorrect, IntoDefect, IntoOccupied, Site, So};
