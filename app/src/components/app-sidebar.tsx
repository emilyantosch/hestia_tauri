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
import { Input } from "@/components/ui/input";
import {
  RiFolderLine,
  RiPriceTag3Line,
  RiTimeLine,
  RiPlanetLine,
  RiSeedlingLine,
  RiImageLine,
  RiSettings3Line,
  RiAddLine,
  RiFilter3Line,
  RiStarLine,
  RiSearchLine,
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
  smartFolders: [
    {
      title: "Recent Photos",
      url: "#",
      icon: RiImageLine,
      filter: "type:image AND modified:last7days",
    },
    {
      title: "Large Files",
      url: "#",
      icon: RiFilter3Line,
      filter: "size:>100MB",
    },
    {
      title: "Favorites",
      url: "#",
      icon: RiStarLine,
      filter: "tags:favorite",
    },
  ],
};

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  const [folderFilter, setFolderFilter] = React.useState("");

  // Filter folders based on search
  const filteredQuickAccess = data.navMain[0]?.items.filter((item) =>
    item.title.toLowerCase().includes(folderFilter.toLowerCase())
  ) || [];

  const filteredSmartFolders = data.smartFolders.filter((folder) =>
    folder.title.toLowerCase().includes(folderFilter.toLowerCase())
  );

  return (
    <Sidebar {...props} className="!border-none">
      <SidebarHeader>
        <div className="flex items-center gap-2">
          <div className="flex-1">
            <LibrarySwitcher libraries={data.libraries} />
          </div>
          <SidebarTrigger className="h-6 w-6" />
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6 text-sidebar-foreground hover:text-sidebar-foreground hover:bg-sidebar-accent"
          >
            <RiSettings3Line size={18} aria-hidden="true" />
            <span className="sr-only">Settings</span>
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
              {filteredQuickAccess.map((item) => (
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

        {/* Smart Folders Section */}
        <SidebarGroup className="mt-4">
          <div className="flex items-center justify-between px-2">
            <SidebarGroupLabel className="uppercase text-sidebar-foreground/50">
              Smart Folders
            </SidebarGroupLabel>
            <Button
              variant="ghost"
              size="icon"
              className="h-5 w-5 text-sidebar-foreground/50 hover:text-sidebar-foreground hover:bg-sidebar-accent"
            >
              <RiAddLine size={14} aria-hidden="true" />
              <span className="sr-only">Add Smart Folder</span>
            </Button>
          </div>
          <SidebarGroupContent className="px-2">
            <SidebarMenu>
              {filteredSmartFolders.map((folder) => (
                <SidebarMenuItem key={folder.title}>
                  <SidebarMenuButton
                    asChild
                    className="group/menu-button font-medium gap-3 h-9 rounded-md hover:bg-sidebar-accent/50 [&>svg]:size-auto"
                  >
                    <a href={folder.url} title={folder.filter}>
                      {folder.icon && (
                        <folder.icon
                          className="text-sidebar-foreground/50"
                          size={22}
                          aria-hidden="true"
                        />
                      )}
                      <span>{folder.title}</span>
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
            Folders
          </SidebarGroupLabel>
          <SidebarGroupContent className="px-2">
            <WatchedFoldersTree />
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter className="p-2">
        <div className="relative">
          <RiFilter3Line
            className="absolute left-3 top-1/2 transform -translate-y-1/2 text-sidebar-foreground"
            size={16}
            aria-hidden="true"
          />
          <Input
            type="text"
            placeholder="Filter"
            value={folderFilter}
            onChange={(e) => setFolderFilter(e.target.value)}
            className="pl-10 h-8 bg-sidebar-accent/30 border-border/80 text-sidebar-foreground placeholder:text-sidebar-foreground/50 focus:bg-sidebar-accent/50 focus:border-border/80"
          />
        </div>
      </SidebarFooter>
    </Sidebar>
  );
}
