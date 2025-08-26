import * as React from "react";

import { LibrarySwitcher } from "@/components/library-switcher";
import { WatchedFoldersTree } from "@/components/tree-sidebar-section";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarTrigger,
} from "@/components/ui/sidebar";
import { Button } from "@/components/ui/button";
import {
  RiFolderLine,
  RiPriceTag3Line,
  RiTimeLine,
  RiPlanetLine,
  RiSeedlingLine,
  RiImageLine,
} from "@remixicon/react";

// This is sample data.
const data = {
  libraries: [
    {
      name: "My Library",
      logo: "https://raw.githubusercontent.com/origin-space/origin-images/refs/heads/main/exp2/logo-01_upxvqe.png",
    },
    {
      name: "Work Library",
      logo: "https://raw.githubusercontent.com/origin-space/origin-images/refs/heads/main/exp2/logo-01_upxvqe.png",
    },
    {
      name: "Project Library",
      logo: "https://raw.githubusercontent.com/origin-space/origin-images/refs/heads/main/exp2/logo-01_upxvqe.png",
    },
  ],
  navMain: [
    {
      title: "Quick Access",
      url: "#",
      items: [
        {
          title: "All",
          url: "#",
          icon: RiFolderLine,
          isActive: true,
        },
        {
          title: "Untagged",
          url: "#",
          icon: RiPriceTag3Line,
          isActive: false,
        },
        {
          title: "Recent",
          url: "#",
          icon: RiTimeLine,
          isActive: false,
        },
      ],
    },
    {
      title: "More",
      url: "#",
      items: [
        {
          title: "Community",
          url: "#",
          icon: RiPlanetLine,
          isActive: false,
        },
        {
          title: "Help Centre",
          url: "#",
          icon: RiSeedlingLine,
          isActive: false,
        },
        {
          title: "Settings",
          url: "#",
          icon: RiImageLine,
          isActive: false,
        },
      ],
    },
  ],
};

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  return (
    <Sidebar {...props} className="!border-none">
      <SidebarHeader>
        <div className="flex items-center gap-2">
          <div className="flex-1">
            <LibrarySwitcher libraries={data.libraries} />
          </div>
          <SidebarTrigger className="h-8 w-8" />
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8 text-sidebar-foreground/50 hover:text-sidebar-foreground hover:bg-sidebar-accent"
          >
            <RiImageLine size={18} aria-hidden="true" />
            <span className="sr-only">Thumbnail</span>
          </Button>
        </div>
      </SidebarHeader>
      <SidebarContent>
        {/* Quick Access Section */}
        <SidebarGroup>
          <SidebarGroupLabel className="uppercase text-sidebar-foreground/50">
            {data.navMain[0]?.title}
          </SidebarGroupLabel>
          <SidebarGroupContent className="px-2">
            <SidebarMenu>
              {data.navMain[0]?.items.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton
                    asChild
                    className="group/menu-button font-medium gap-3 h-9 rounded-md data-[active=true]:hover:bg-transparent data-[active=true]:bg-gradient-to-b data-[active=true]:from-sidebar-primary data-[active=true]:to-sidebar-primary/70 data-[active=true]:shadow-[0_1px_2px_0_rgb(0_0_0/.05),inset_0_1px_0_0_rgb(255_255_255/.12)] [&>svg]:size-auto"
                    isActive={item.isActive}
                  >
                    <a href={item.url}>
                      {item.icon && (
                        <item.icon
                          className="text-sidebar-foreground/50 group-data-[active=true]/menu-button:text-white"
                          size={22}
                          aria-hidden="true"
                        />
                      )}
                      <span className="group-data-[active=true]/menu-button:text-white">{item.title}</span>
                    </a>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
        
        {/* Watched Folders Section */}
        <SidebarGroup className="mt-4">
          <SidebarGroupLabel className="uppercase text-sidebar-foreground/50">
            Watched Folders
          </SidebarGroupLabel>
          <SidebarGroupContent className="px-2">
            <WatchedFoldersTree />
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter>
      </SidebarFooter>
    </Sidebar>
  );
}
