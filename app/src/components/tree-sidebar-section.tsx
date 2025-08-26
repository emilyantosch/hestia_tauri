import {
  expandAllFeature,
  hotkeysCoreFeature,
  selectionFeature,
  syncDataLoaderFeature,
} from "@headless-tree/core"
import { useTree } from "@headless-tree/react"
import {
  FolderIcon,
  FolderOpenIcon,
  FileIcon,
} from "lucide-react"

import { Tree, TreeItem, TreeItemLabel } from "@/components/tree"

interface WatchedFolder {
  name: string
  path: string
  children?: string[]
}

// Mock data - this will come from the backend via Tauri
const watchedFolders: Record<string, WatchedFolder> = {
  "documents": {
    name: "Documents",
    path: "/home/user/Documents",
    children: ["projects", "notes", "readme-file"],
  },
  "projects": {
    name: "Projects",
    path: "/home/user/Documents/Projects",
    children: ["hestia", "portfolio", "config-file"],
  },
  "hestia": {
    name: "Hestia",
    path: "/home/user/Documents/Projects/hestia",
    children: ["src-file", "cargo-file"],
  },
  "portfolio": {
    name: "Portfolio",
    path: "/home/user/Documents/Projects/portfolio",
    children: ["index-file"],
  },
  "notes": {
    name: "Notes",
    path: "/home/user/Documents/Notes",
    children: ["todo-file", "ideas-file"],
  },
  "downloads": {
    name: "Downloads",
    path: "/home/user/Downloads",
    children: ["images", "videos", "temp-file"],
  },
  "images": {
    name: "Images",
    path: "/home/user/Downloads/Images",
    children: ["photo-file"],
  },
  "videos": {
    name: "Videos", 
    path: "/home/user/Downloads/Videos",
    children: ["movie-file"],
  },
  // Files
  "readme-file": {
    name: "README.md",
    path: "/home/user/Documents/README.md",
  },
  "config-file": {
    name: ".gitconfig",
    path: "/home/user/Documents/Projects/.gitconfig",
  },
  "src-file": {
    name: "main.rs",
    path: "/home/user/Documents/Projects/hestia/main.rs",
  },
  "cargo-file": {
    name: "Cargo.toml",
    path: "/home/user/Documents/Projects/hestia/Cargo.toml",
  },
  "index-file": {
    name: "index.html",
    path: "/home/user/Documents/Projects/portfolio/index.html",
  },
  "todo-file": {
    name: "todo.txt",
    path: "/home/user/Documents/Notes/todo.txt",
  },
  "ideas-file": {
    name: "ideas.md",
    path: "/home/user/Documents/Notes/ideas.md",
  },
  "temp-file": {
    name: "temp.zip",
    path: "/home/user/Downloads/temp.zip",
  },
  "photo-file": {
    name: "vacation.jpg",
    path: "/home/user/Downloads/Images/vacation.jpg",
  },
  "movie-file": {
    name: "presentation.mp4",
    path: "/home/user/Downloads/Videos/presentation.mp4",
  },
}

const indent = 12

export function WatchedFoldersTree() {
  const tree = useTree<WatchedFolder>({
    initialState: {
      expandedItems: ["documents", "projects"],
      selectedItems: [],
    },
    indent,
    rootItemId: "documents",
    getItemName: (item) => item.getItemData().name,
    isItemFolder: (item) => (item.getItemData()?.children?.length ?? 0) > 0,
    dataLoader: {
      getItem: (itemId) => watchedFolders[itemId],
      getChildren: (itemId) => watchedFolders[itemId]?.children ?? [],
    },
    features: [
      syncDataLoaderFeature,
      selectionFeature,
      hotkeysCoreFeature,
      expandAllFeature,
    ],
  })

  return (
    <Tree indent={indent} tree={tree}>
      {tree.getItems().map((item) => {
        return (
          <TreeItem key={item.getId()} item={item}>
            <TreeItemLabel className="bg-sidebar hover:bg-sidebar-accent data-[selected=true]:bg-sidebar-accent data-[selected=true]:text-sidebar-accent-foreground">
              <span className="flex items-center gap-2 text-sm">
                {item.isFolder() ? (
                  item.isExpanded() ? (
                    <FolderOpenIcon className="text-sidebar-foreground/70 pointer-events-none size-4" />
                  ) : (
                    <FolderIcon className="text-sidebar-foreground/70 pointer-events-none size-4" />
                  )
                ) : (
                  <FileIcon className="text-sidebar-foreground/70 pointer-events-none size-4" />
                )}
                <span className="truncate text-sidebar-foreground">{item.getItemName()}</span>
                {item.isFolder() && (
                  <span className="text-sidebar-foreground/50 text-xs">
                    ({item.getChildren().length})
                  </span>
                )}
              </span>
            </TreeItemLabel>
          </TreeItem>
        )
      })}
    </Tree>
  )
}

// Keep the original component for backward compatibility
export default function FolderSidebar() {
  const items: Record<string, { name: string; children?: string[] }> = {
    company: {
      name: "Company",
      children: ["engineering", "marketing", "operations"],
    },
    engineering: {
      name: "Engineering",
      children: ["frontend", "backend", "platform-team"],
    },
    frontend: { name: "Frontend", children: ["design-system", "web-platform"] },
    "design-system": {
      name: "Design System",
      children: ["components", "tokens", "guidelines"],
    },
    components: { name: "Components" },
    tokens: { name: "Tokens" },
    guidelines: { name: "Guidelines" },
    "web-platform": { name: "Web Platform" },
    backend: { name: "Backend", children: ["apis", "infrastructure"] },
    apis: { name: "APIs" },
    infrastructure: { name: "Infrastructure" },
    "platform-team": { name: "Platform Team" },
    marketing: { name: "Marketing", children: ["content", "seo"] },
    content: { name: "Content" },
    seo: { name: "SEO" },
    operations: { name: "Operations", children: ["hr", "finance"] },
    hr: { name: "HR" },
    finance: { name: "Finance" },
  }

  const tree = useTree<{ name: string; children?: string[] }>({
    initialState: {
      expandedItems: ["engineering", "frontend", "design-system"],
      selectedItems: ["components"],
    },
    indent: 20,
    rootItemId: "company",
    getItemName: (item) => item.getItemData().name,
    isItemFolder: (item) => (item.getItemData()?.children?.length ?? 0) > 0,
    dataLoader: {
      getItem: (itemId) => items[itemId],
      getChildren: (itemId) => items[itemId]?.children ?? [],
    },
    features: [
      syncDataLoaderFeature,
      selectionFeature,
      hotkeysCoreFeature,
      expandAllFeature,
    ],
  })

  return (
    <div className="flex h-full flex-col gap-2">
      <Tree indent={20} tree={tree}>
        {tree.getItems().map((item) => {
          return (
            <TreeItem key={item.getId()} item={item}>
              <TreeItemLabel>
                <span className="flex items-center gap-2">
                  {item.isFolder() &&
                    (item.isExpanded() ? (
                      <FolderOpenIcon className="text-muted-foreground pointer-events-none size-4" />
                    ) : (
                      <FolderIcon className="text-muted-foreground pointer-events-none size-4" />
                    ))}
                  {item.getItemName()}
                  {item.isFolder() && (
                    <span className="text-muted-foreground -ms-1">
                      {`(${item.getChildren().length})`}
                    </span>
                  )}
                </span>
              </TreeItemLabel>
            </TreeItem>
          )
        })}
      </Tree>
    </div>
  )
}
