use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct WatchedFolders {
    name: String,
    path: String,
    children: Option<Vec<String>>,
    icon: Option<String>,
    color: Option<String>,
}

impl WatchedFolders {
    pub fn new(name: String, path: String) -> WatchedFolders {
        WatchedFolders {
            name,
            path,
            children: None,
            icon: None,
            color: None,
        }
    }

    pub fn children(mut self, children: Vec<String>) -> Self {
        self.children = Some(children);
        self
    }

    pub fn icon(mut self, icon: String) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn color(mut self, color: String) -> Self {
        self.color = Some(color);
        self
    }
}
