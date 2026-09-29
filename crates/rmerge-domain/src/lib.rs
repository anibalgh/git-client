pub mod entities;
pub mod errors;
pub mod services;
pub mod value_objects;

pub use entities::*;
pub use errors::*;
pub use services::*;
pub use value_objects::*;

pub const APP_VERSION: &str = "1.0.0";
pub const APP_NAME: &str = "Git-Client";
pub const APP_DESCRIPTION: &str = "Cliente Git de alto rendimiento y herramienta 3-Way Merge multiplataforma desarrollada en Rust.";

