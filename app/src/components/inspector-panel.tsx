"use client";
import { lighten } from 'polished';
import { useIsMobile } from "@/hooks/use-mobile";
import { RiImageLine, RiFileTextLine, RiAddLine, RiCloseLine } from "@remixicon/react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Sheet, SheetTitle, SheetContent } from "@/components/ui/sheet";
import * as React from "react";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Icon, IconNode, LucideAArrowDown, LucideAArrowUp, LucideCamera, LucideIceCreamCone, LucideIcon, LucideMap, LucideTreePine } from 'lucide-react';

interface PriceTag {
  children: React.ReactNode,
  color: string,
  icon: LucideIcon,
  onRemove?: () => void,
}


type InspectorPanelContext = {
  openMobile: boolean;
  setOpenMobile: (open: boolean) => void;
  isMobile: boolean;
  togglePanel: () => void;
};

const InspectorPanelContext = React.createContext<InspectorPanelContext | null>(
  null,
);

function useInspectorPanel() {
  const context = React.useContext(InspectorPanelContext);
  if (!context) {
    throw new Error(
      "useInspectorPanel must be used within a InspectorPanelProvider.",
    );
  }
  return context;
}

const InspectorPanelProvider = ({ children }: { children: React.ReactNode }) => {
  const isMobile = useIsMobile();
  const [openMobile, setOpenMobile] = React.useState(false);

  // Helper to toggle the sidebar.
  const togglePanel = React.useCallback(() => {
    return isMobile && setOpenMobile((open) => !open);
  }, [isMobile, setOpenMobile]);

  const contextValue = React.useMemo<InspectorPanelContext>(
    () => ({
      isMobile,
      openMobile,
      setOpenMobile,
      togglePanel,
    }),
    [isMobile, openMobile, setOpenMobile, togglePanel],
  );

  return (
    <InspectorPanelContext.Provider value={contextValue}>
      {children}
    </InspectorPanelContext.Provider>
  );
};
InspectorPanelProvider.displayName = "InspectorPanelProvider";

// Custom PriceTag component using SVG
const PriceTag = ({
  children,
  color,
  icon: Icon,
  onRemove
}: PriceTag) => {
  return (
    <div className="relative inline-block">
      <div className="relative inline-flex justify-start items-center h-8">
        {/* Rectangular background that scales with text */}
        <div
          className="h-full border-1 rounded-xl flex justify-start items-center"
          style={{
            backgroundColor: `${color}20`, // 50% opacity
            borderColor: color
          }}
        >
          {/* Tag content */}
          <div className="relative z-10 py-1 flex justify-between items-center text-sm font-medium text-white whitespace-nowrap"
            style={{
              color: color,
            }}>
            <div className="mx-1">
              <Icon></Icon>
            </div>
            <div>
              {children}
            </div>
          </div>
        </div>

        {/* Fixed-size pointed edge - positioned to connect seamlessly */}
      </div>

      {/* Remove button */}
      {onRemove && (
        <button
          onClick={onRemove}
          className="absolute -right-1 -top-1 w-4 h-4 bg-background border rounded-full flex items-center justify-center hover:bg-muted transition-colors z-20"
          style={{ borderColor: color }}
        >
          <RiCloseLine size={10} style={{ color }} />
          <span className="sr-only">Remove tag</span>
        </button>
      )}
    </div>
  );
};

const InspectorPanelContent = () => {
  // Mock state for selected file - this would come from your app state
  const [selectedFile, setSelectedFile] = React.useState<{
    name: string;
    type: 'image' | 'document' | 'video' | 'audio' | 'other';
    thumbnail?: string;
    size: string;
    modified: string;
    tags: { name: string; color: string; icon: LucideIcon }[];
  } | null>(null);

  // State for tag input
  const [newTag, setNewTag] = React.useState("");
  const [isAddingTag, setIsAddingTag] = React.useState(false);

  // Mock file for demonstration - remove this when connecting to real state
  React.useEffect(() => {
    // Simulate selecting a file after 2 seconds for demo purposes
    const timer = setTimeout(() => {
      setSelectedFile({
        name: "vacation-photo.jpg",
        type: "image",
        thumbnail: "https://images.unsplash.com/photo-1506905925346-21bda4d32df4?w=400&h=300&fit=crop",
        size: "2.4 MB",
        modified: "2 hours ago",
        tags: [
          { name: "Travel", color: "#3B82F6", icon: LucideMap }, // blue
          { name: "Photography", color: "#10B981", icon: LucideCamera }, // green
          { name: "Nature", color: "#F59E0B", icon: LucideTreePine } // amber
        ]
      });
    }, 2000);
    return () => clearTimeout(timer);
  }, []);

  // Available colors for new tags
  const tagColors = [
    "#3B82F6", "#10B981", "#F59E0B", "#EF4444", "#8B5CF6",
    "#F97316", "#06B6D4", "#84CC16", "#EC4899", "#6B7280"
  ];

  // Tag management functions
  const addTag = (icon: LucideIcon) => {
    if (newTag.trim() && selectedFile && !selectedFile.tags.some(tag => tag.name === newTag.trim())) {
      // Assign a random color for new tags
      const randomColor = tagColors[Math.floor(Math.random() * tagColors.length)];
      setSelectedFile({
        ...selectedFile,
        tags: [...selectedFile.tags, { name: newTag.trim(), color: randomColor, icon: icon }]
      });
      setNewTag("");
      setIsAddingTag(false);
    }
  };

  const removeTag = (tagToRemove: string) => {
    if (selectedFile) {
      setSelectedFile({
        ...selectedFile,
        tags: selectedFile.tags.filter(tag => tag.name !== tagToRemove)
      });
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      addTag(LucideAArrowDown);
    } else if (e.key === 'Escape') {
      setNewTag("");
      setIsAddingTag(false);
    }
  };

  const renderPlaceholder = () => (
    <div className="flex flex-col items-center justify-center h-full min-h-[400px] text-muted-foreground/50">
      <RiFileTextLine size={48} className="mb-4" />
      <p className="text-sm text-center">
        Select a file to view<br />its thumbnail
      </p>
    </div>
  );

  const renderThumbnail = () => {
    if (!selectedFile) return renderPlaceholder();

    return (
      <div className="flex flex-col h-full">
        {/* Thumbnail container */}
        <div className="flex-1 p-4">
          <div className="w-full h-full min-h-[300px] bg-background rounded-lg border border-border/50 overflow-hidden flex items-center justify-center">
            {selectedFile.thumbnail ? (
              <img
                src={selectedFile.thumbnail}
                alt={selectedFile.name}
                className="max-w-full max-h-full object-contain"
              />
            ) : (
              <div className="flex flex-col items-center justify-center text-muted-foreground/50">
                <RiFileTextLine size={32} className="mb-2" />
                <p className="text-xs">No preview available</p>
              </div>
            )}
          </div>
        </div>

        {/* Separator */}
        <div className="border-t border-border/50" />

        {/* File info below thumbnail */}
        <div className="py-4 px-4">
          <div className="flex items-center gap-2 mb-2">
            <RiImageLine
              className="text-muted-foreground/70"
              size={16}
              aria-hidden="true"
            />
            <h2 className="text-sm font-medium truncate">{selectedFile.name}</h2>
          </div>
          <div className="text-xs text-muted-foreground/70 space-y-1">
            <div>{selectedFile.size}</div>
            <div>Modified {selectedFile.modified}</div>
          </div>
        </div>

        {/* Separator */}
        <div className="border-t border-border/50" />

        {/* Tags section */}
        <div className="py-4 px-4">
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-sm font-medium">Tags</h3>
            {!isAddingTag && (
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setIsAddingTag(true)}
                className="h-7 w-7 p-0"
              >
                <RiAddLine size={14} />
                <span className="sr-only">Add tag</span>
              </Button>
            )}
          </div>

          {/* Tag input */}
          {isAddingTag && (
            <div className="mb-3">
              <Input
                value={newTag}
                onChange={(e) => setNewTag(e.target.value)}
                onKeyDown={handleKeyDown}
                onBlur={() => {
                  if (!newTag.trim()) {
                    setIsAddingTag(false);
                  } else {
                    addTag(LucideAArrowDown);
                  }
                }}
                placeholder="Enter tag name"
                className="h-7 text-xs"
                autoFocus
              />
            </div>
          )}

          {/* Tags list */}
          <div className="flex flex-wrap gap-3">
            {selectedFile.tags.map((tag) => (
              <PriceTag
                key={tag.name}
                color={tag.color}
                icon={tag.icon}
                onRemove={() => removeTag(tag.name)}
              >
                {tag.name}
              </PriceTag>
            ))}
            {selectedFile.tags.length === 0 && !isAddingTag && (
              <p className="text-xs text-muted-foreground/70">No tags added</p>
            )}
          </div>
        </div>
      </div>
    );
  };

  return selectedFile ? renderThumbnail() : renderPlaceholder();
};
InspectorPanelContent.displayName = "InspectorPanelContent";

const InspectorPanel = () => {
  const { isMobile, openMobile, setOpenMobile } = useInspectorPanel();

  if (isMobile) {
    return (
      <Sheet open={openMobile} onOpenChange={setOpenMobile}>
        <SheetContent className="w-72 px-4 md:px-6 py-0 bg-muted [&>button]:hidden">
          <SheetTitle className="hidden">Inspector</SheetTitle>
          <div className="flex h-full w-full flex-col">
            <InspectorPanelContent />
          </div>
        </SheetContent>
      </Sheet>
    );
  }

  return (
    <ScrollArea className="bg-muted">
      <div className="w-[300px] px-4 md:px-6 bg-muted">
        <InspectorPanelContent />
      </div>
    </ScrollArea>
  );
};
InspectorPanel.displayName = "InspectorPanel";

const InspectorPanelTrigger = ({
  onClick,
}: {
  onClick?: (event: React.MouseEvent<HTMLButtonElement>) => void;
}) => {
  const { isMobile, togglePanel } = useInspectorPanel();

  if (!isMobile) {
    return null;
  }

  return (
    <Button
      variant="ghost"
      className="px-2"
      onClick={(event) => {
        onClick?.(event);
        togglePanel();
      }}
    >
      <RiImageLine
        className="text-muted-foreground sm:text-muted-foreground/70 size-5"
        size={20}
        aria-hidden="true"
      />
      <span className="max-sm:sr-only">Thumbnail</span>
    </Button>
  );
};
InspectorPanelTrigger.displayName = "InspectorPanelTrigger";

export {
  InspectorPanel,
  InspectorPanelProvider,
  InspectorPanelTrigger,
  useInspectorPanel,
};
