import {
  expandAllFeature,
  hotkeysCoreFeature,
  selectionFeature,
  syncDataLoaderFeature,
} from "@headless-tree/core"
import { useTree } from "@headless-tree/react"
import { invoke } from "@tauri-apps/api/core"
import {
  FolderIcon,
  FolderOpenIcon,
  FileIcon,
} from "lucide-react"

import { Tree, TreeItem, TreeItemLabel } from "@/components/tree"
import { useQuery } from "@tanstack/react-query"

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
const indent = 12

export function WatchedFoldersTree() {
  const { isPending, error, data } = useQuery({
    queryKey: ['watched_folder_tree'],
    queryFn: () => invoke('get_watched_folders'),
    retry: true
  })

  if (isPending) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary mx-auto mb-4"></div>
          <p className="text-muted-foreground">Loading watched folders...</p>
        </div>
      </div>
    )
  }

  if (error) {
    <div className="min-h-screen bg-background flex items-center justify-center">
      <div className="text-center">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary mx-auto mb-4"></div>
        <p className="text-muted-foreground">No folders could be found</p>
      </div>
    </div>
  }

  const watched_folder_tree = data as Record<string, WatchedFolder> | undefined;

  if (watched_folder_tree) {
    console.log(watched_folder_tree);
    const tree = useTree<WatchedFolder>({
      initialState: {
        expandedItems: ["0"],
        selectedItems: [],
      },
      indent,
      rootItemId: folderId.rootFolderId,
      getItemName: (item) => item.getItemData().name,
      isItemFolder: () => true,
      dataLoader: {
        getItem: (itemId) => watched_folder_tree[itemId],
        getChildren: (itemId) => watched_folder_tree[itemId]?.children ?? [],
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
}

