"use client";

import { InspectorPanelTrigger } from "@/components/inspector-panel";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  RiGridLine,
  RiListUnordered,
  RiFilter3Line,
  RiShiningFill,
  RiSearchLine,
} from "@remixicon/react";
import ImageGrid from "@/components/image-grid";
import { useState } from "react";

export default function Chat() {
  const [isGridView, setIsGridView] = useState(true);

  return (
    <ScrollArea className="flex-1 [&>div>div]:h-full w-full shadow-2xl md:rounded-s-[inherit] min-[1024px]:rounded-e-3xl bg-background overflow-hidden">
      <div className="h-full flex flex-col px-4 md:px-6 lg:px-8">
        {/* Header */}
        <div className="py-5 bg-background sticky top-0 z-10">
          <div className="flex items-center gap-4">
            {/* Left: Breadcrumb */}
            <Breadcrumb className="flex-shrink-0">
              <BreadcrumbList className="sm:gap-1.5">
                <BreadcrumbItem>
                  <BreadcrumbLink href="#">Library</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator className="text-sidebar-primary" />
                <BreadcrumbItem>
                  <BreadcrumbPage>Gallery</BreadcrumbPage>
                </BreadcrumbItem>
              </BreadcrumbList>
            </Breadcrumb>
            
            {/* Center: Search Bar */}
            <div className="flex-1 flex justify-center">
              <div className="relative max-w-sm w-full">
                <RiSearchLine 
                  className="absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground/70 size-4" 
                  size={16} 
                  aria-hidden="true" 
                />
                <Input
                  type="search"
                  placeholder="Search files..."
                  className="pl-10 h-9 bg-muted/50 border-border/50 focus:bg-background"
                />
              </div>
            </div>
            
            {/* Right: Toolbar Buttons */}
            <div className="flex items-center gap-1 -my-2 -me-2 flex-shrink-0">
              <Button 
                variant="ghost" 
                size="icon"
                className="h-8 w-8"
                onClick={() => setIsGridView(!isGridView)}
              >
                {isGridView ? (
                  <RiGridLine
                    className="text-sidebar-primary size-5"
                    size={20}
                    aria-hidden="true"
                  />
                ) : (
                  <RiListUnordered
                    className="text-sidebar-primary size-5"
                    size={20}
                    aria-hidden="true"
                  />
                )}
                <span className="sr-only">{isGridView ? 'Switch to list view' : 'Switch to grid view'}</span>
              </Button>
              <Button variant="ghost" size="icon" className="h-8 w-8">
                <RiFilter3Line
                  className="text-sidebar-primary size-5"
                  size={20}
                  aria-hidden="true"
                />
                <span className="sr-only">Filter</span>
              </Button>
              <Button variant="ghost" size="icon" className="h-8 w-8">
                <RiShiningFill
                  className="text-sidebar-primary size-5"
                  size={20}
                  aria-hidden="true"
                />
                <span className="sr-only">Quick actions</span>
              </Button>
              <InspectorPanelTrigger />
            </div>
          </div>
        </div>
        {/* Image Grid */}
        <div className="relative grow">
          <ImageGrid />
        </div>
      </div>
    </ScrollArea>
  );
}
