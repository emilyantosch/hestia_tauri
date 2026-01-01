use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Color {
    name: String,
    hex: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Icon {
    name: String,
    content: String,
    content_type: IconType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IconType {
    Icon,
    Image,
    Text,
}

impl Default for IconType {
    fn default() -> Self {
        IconType::Icon
    }
}
