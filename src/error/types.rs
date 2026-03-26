use thiserror::Error; #[derive(Error, Debug)] pub enum InfynonError { #[error("Installation Blocked: {0}")] Blocked(String), #[error("System Error: {0}")] System(String), }
