import {
  SidebarGroup,
  SidebarMenu,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenuItem,
  SidebarMenuButton,
} from "../ui/sidebar";

import { ChevronDown } from "lucide-react";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "../ui/collapsible";

export type QuickAccessItem = {
  title: string;
  icon: React.ElementType;
};

type QuickAccess = {
  title: string;
  quickAccessItems: Array<QuickAccessItem>;
}

export function SidebarQuickAccess({ title, quickAccessItems }: QuickAccess) {
  return (
    <Collapsible className="group/collapsible">
      <SidebarGroup className="overflow-clip">
        <SidebarGroupLabel asChild>
          <CollapsibleTrigger>
            {title}
            <ChevronDown className="ml-auto transition-transform group-data-[state=open]/collapsible:rotate-180" />
          </CollapsibleTrigger>
        </SidebarGroupLabel>
        <CollapsibleContent>
          <SidebarGroupContent>
            <SidebarMenu>
              {quickAccessItems.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton asChild>
                    <div>
                      <span>{item.title}</span>
                    </div>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </CollapsibleContent>
      </SidebarGroup>
    </Collapsible >
  );
}
