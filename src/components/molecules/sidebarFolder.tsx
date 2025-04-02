import {
  SidebarGroup,
  SidebarMenu,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenuItem,
  SidebarMenuButton,
  SidebarMenuSub,
  SidebarMenuSubItem,
  SidebarMenuSubButton
} from "../ui/sidebar";

import { ChevronDown, FolderClosed } from "lucide-react";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "../ui/collapsible";

export type Folder = {
  title: string;
  subfolder: Array<Folder>;
}

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
              <Collapsible defaultOpen className="group/collapsible">
                {subfolder.map((item) => (
                  <SidebarMenuItem key={item.title}>
                    <CollapsibleTrigger>
                      <SidebarMenuButton asChild>
                        <div>
                          <FolderClosed className="mx-1" />
                          <span>{item.title}</span>
                        </div>
                      </SidebarMenuButton>
                    </CollapsibleTrigger>
                    <CollapsibleContent>
                      <SidebarMenuSub>
                        {item.subfolder.map((subitem) => (
                          <SidebarMenuSubItem key={subitem.title}>
                            <SidebarMenuSubButton asChild>
                              <div>
                                <FolderClosed className="mx-1" />
                                <span>{subitem.title}</span>
                              </div>
                            </SidebarMenuSubButton>
                            <SidebarMenuSub>
                              {subitem.subfolder.map((subitem) => (
                                <SidebarMenuSubItem key={subitem.title}>
                                  <SidebarMenuSubButton asChild>
                                    <div>
                                      <FolderClosed className="mx-1" />
                                      <span>{subitem.title}</span>
                                    </div>
                                  </SidebarMenuSubButton>
                                </SidebarMenuSubItem>
                              ))}
                            </SidebarMenuSub>
                          </SidebarMenuSubItem>
                        ))}
                      </SidebarMenuSub>
                    </CollapsibleContent>
                  </SidebarMenuItem>
                ))}
              </Collapsible>
            </SidebarMenu>
          </SidebarGroupContent>
        </CollapsibleContent>
      </SidebarGroup>
    </Collapsible >
  )
}
