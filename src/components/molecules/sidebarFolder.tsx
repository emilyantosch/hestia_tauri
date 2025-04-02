import {
  SidebarGroup,
  SidebarMenu,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenuItem,
  SidebarMenuButton,
  SidebarMenuSub,
  SidebarMenuSubItem,
  SidebarMenuSubButton,
} from "../ui/sidebar";

import { ChevronDown, FolderClosed } from "lucide-react";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "../ui/collapsible";

export type Folder = {
  title: string;
  subfolder: Array<Folder>;
};

// Recursive component to render nested folders
const RecursiveFolderItem = ({ folder }: { folder: Folder }) => {
  return (
    <SidebarMenuSubItem key={folder.title}>
      <Collapsible defaultOpen className="group/collapsible">
        <CollapsibleTrigger>
          <SidebarMenuSubButton asChild>
            <div>
              <FolderClosed className="mx-1" />
              <span>{folder.title}</span>
              {folder.subfolder.length > 0 && (
                <ChevronDown className="ml-auto transition-transform group-data-[state=open]/collapsible:rotate-180" />
              )}
            </div>
          </SidebarMenuSubButton>
        </CollapsibleTrigger>
        {folder.subfolder.length > 0 && (
          <CollapsibleContent>
            <SidebarMenuSub>
              {folder.subfolder.map((subFolder) => (
                <RecursiveFolderItem key={subFolder.title} folder={subFolder} />
              ))}
            </SidebarMenuSub>
          </CollapsibleContent>
        )}
      </Collapsible>
    </SidebarMenuSubItem>
  );
};

export function SidebarFolder({ title, subfolder }: Folder) {
  return (
    <Collapsible defaultOpen className="group/collapsible">
      <SidebarGroup>
        <SidebarGroupLabel asChild>
          <CollapsibleTrigger>
            {title}
            <ChevronDown className="ml-auto transition-transform group-data-[state=open]/collapsible:rotate-180" />
          </CollapsibleTrigger>
        </SidebarGroupLabel>
        <CollapsibleContent>
          <SidebarGroupContent>
            <SidebarMenu>
              {subfolder.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <Collapsible defaultOpen className="group/collapsible">
                    <CollapsibleTrigger>
                      <SidebarMenuButton asChild>
                        <div>
                          <FolderClosed className="mx-1" />
                          <span>{item.title}</span>
                          {item.subfolder.length > 0 && (
                            <ChevronDown className="ml-auto transition-transform group-data-[state=open]/collapsible:rotate-180" />
                          )}
                        </div>
                      </SidebarMenuButton>
                    </CollapsibleTrigger>
                    {item.subfolder.length > 0 && (
                      <CollapsibleContent>
                        <SidebarMenuSub>
                          {item.subfolder.map((subFolder) => (
                            <RecursiveFolderItem
                              key={subFolder.title}
                              folder={subFolder}
                            />
                          ))}
                        </SidebarMenuSub>
                      </CollapsibleContent>
                    )}
                  </Collapsible>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </CollapsibleContent>
      </SidebarGroup>
    </Collapsible>
  );
}
