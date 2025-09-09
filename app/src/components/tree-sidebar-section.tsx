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
  icon?: string
  color?: string
  children?: string[]
}

enum folderId {
  rootFolderId = "0",
}
// Mock data - this will come from the backend via Tauri
const watchedFolders: Record<string, WatchedFolder> = {
  "0": {
    name: "Library",
    path: "",
    children: ["1", "4"],
  },
  "1": {
    name: "Projects",
    path: "/home/user/Documents/Projects",
    children: ["2", "3"],
  },
  "2": {
    name: "Hestia",
    path: "/home/user/Documents/Projects/hestia",
    children: ["5"],
  },
  "3": {
    name: "Portfolio",
    path: "/home/user/Documents/Projects/portfolio",
    children: [],
  },
  "4": {
    name: "Notes",
    path: "/home/user/Documents/Notes",
    children: [],
  },
  "5": {
    name: "Downloads",
    path: "/home/user/Downloads",
    children: ["6", "7"],
  },
  "6": {
    name: "Images",
    path: "/home/user/Downloads/Images",
    children: [],
  },
  "7": {
    name: "Videos",
    path: "/home/user/Downloads/Videos",
  },
}

const indent = 12

export function WatchedFoldersTree() {
  const tree = useTree<WatchedFolder>({
    initialState: {
      expandedItems: ["library", "projects"],
      selectedItems: [],
    },
    indent,
    rootItemId: folderId.rootFolderId,
    getItemName: (item) => item.getItemData().name,
    isItemFolder: (item) => true,
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

