use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct WatchedFolderTree {
    name: String,
    path: String,
    children: Option<Vec<String>>,
    icon: Option<String>,
    color: Option<String>,
}

impl WatchedFolderTree {
    pub fn new(name: String, path: String) -> WatchedFolderTree {
        WatchedFolderTree {
            name,
            path,
            children: None,
            icon: None,
            color: None,
        }
    }

    pub fn with_icon_and_color(
        name: String,
        path: String,
        icon: Option<String>,
        color: Option<String>,
    ) -> WatchedFolderTree {
        WatchedFolderTree {
            name,
            path,
            children: None,
            icon,
            color,
        }
    }

    pub fn with(
        name: String,
        path: String,
        children: Option<Vec<String>>,
        icon: Option<String>,
        color: Option<String>,
    ) -> WatchedFolderTree {
        WatchedFolderTree {
            name,
            path,
            children,
            icon,
            color,
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
