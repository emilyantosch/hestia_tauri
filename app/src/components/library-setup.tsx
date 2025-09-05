"use client";


import { open } from '@tauri-apps/plugin-dialog';
import { openPath } from '@tauri-apps/plugin-opener';

import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import {
  RiFolderAddLine,
  RiFolderOpenLine,
  RiFolderLine,
  RiCloudLine,
  RiPriceTagLine,
  RiArrowRightLine,
} from "@remixicon/react";
import { QueryClient, QueryClientProvider, useQuery } from '@tanstack/react-query';

const queryClient = new QueryClient();

interface LibrarySetupProps {
  onLibraryCreated: () => void;
}

interface LibraryListProps {
  callback: (path: String) => Promise<any>;
}

function LibraryList({ callback }: LibraryListProps): React.ReactNode {
  const { isPending, error, data, refetch } = useQuery({
    queryKey: ['library-list'],
    queryFn: () => invoke('list_available_library'),
    retry: false
  });

  if (isPending) {
    return (
      <div>
        List is loading...
      </div>
    )
  }

  if (error) {
    return (
      <div>
        There are no libraries available
      </div>
    )
  }

  const library_list_data = data as String[] || [];
  return (
    <div className="space-y-2">
      {library_list_data.length === 0 ? (
        <div className="text-center py-4">
          <p className="text-sm text-muted-foreground">No libraries found</p>
        </div>
      ) : (
        library_list_data.map((libraryName, index) => (
          <div
            key={index}
            onClick={() => callback(libraryName)}
            className="group flex items-center justify-between p-3 bg-muted/30 hover:bg-muted/60 border border-border/40 hover:border-border/60 rounded-lg cursor-pointer transition-all duration-200 hover:shadow-sm"
          >
            <div className="flex items-center space-x-3">
              <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center">
                <RiFolderLine size={16} className="text-primary" />
              </div>
              <span className="font-medium text-sm">{libraryName}</span>
            </div>
            <RiArrowRightLine
              size={16}
              className="text-muted-foreground group-hover:text-foreground group-hover:translate-x-0.5 transition-all duration-200"
            />
          </div>
        ))
      )}
    </div>
  )
}



export function LibrarySetup({ onLibraryCreated }: LibrarySetupProps) {
  const [isCreating, setIsCreating] = useState(false);
  const [isOpening, setIsOpening] = useState(false);
  const [newLibraryName, setNewLibraryName] = useState("");
  const [selectedPath, setSelectedPath] = useState("");
  const [libraryList, setLibraryList] = useState([]);



  const handleSelectExisting = async (path: String) => {
    try {
      await invoke("select_library", { path });
      await invoke("initialize_library_workspace");
      console.log("INFO: Reinitialzation of libraray workspace complete, finalizing...");
      onLibraryCreated();
    } catch (error) {
      console.error("Failed to open existing library due to:", error);
    }
  };

  const handleCreateNew = async () => {
    if (!newLibraryName.trim()) return;

    try {
      setIsCreating(true);
      const path = await invoke("select_folder");
      console.log("INFO: Extracted Path for new library");
      // This will be implemented on the Rust side
      await invoke("create_new_library", { name: newLibraryName.trim(), path });
      console.log("INFO: Created new library, refetching...");
      await invoke("initialize_library_workspace");
      console.log("INFO: Reinitialzation of libraray workspace complete, finalizing...");
      onLibraryCreated();
    } catch (error) {
      console.error("Failed to create library:", error);
    } finally {
      setIsCreating(false);
    }
  };

  const handleOpenFromCloud = async () => {
    // Placeholder for future cloud functionality
    console.log("Cloud library setup - to be implemented");
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-background via-muted/30 to-muted/60 flex items-center justify-center p-6">
      <QueryClientProvider client={queryClient}>
        <div className="w-full max-w-2xl">
          {/* Header with Hestia branding */}
          <div className="text-center mb-12">
            <div className="mb-6">
              {/* Hestia Logo - using a tag icon as placeholder */}
              <div className="w-20 h-20 mx-auto mb-4 rounded-full bg-gradient-to-br from-primary to-primary/70 flex items-center justify-center shadow-lg">
                <RiPriceTagLine size={32} className="text-primary-foreground" />
              </div>
            </div>
            <h1 className="text-4xl font-bold bg-gradient-to-r from-foreground to-foreground/70 bg-clip-text text-transparent mb-2">
              Hestia
            </h1>
            <p className="text-muted-foreground text-lg">
              Tag-based File Management
            </p>
            <div className="mt-2 text-sm text-muted-foreground/70">
              Version 0.1.0
            </div>
          </div>

          {/* Setup Options */}
          <div className="space-y-4">
            {/* List of Libraries */}
            <Card className="group hover:shadow-md transition-all duration-200 border-2 hover:border-primary/20">
              <CardHeader className="pb-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-3">
                    <div className="w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center">
                      <RiFolderAddLine size={20} className="text-primary" />
                    </div>
                    <div>
                      <CardTitle className="text-lg">List of Libraries</CardTitle>
                      <CardDescription className="text-sm">
                        Select an existing Library from the list
                      </CardDescription>
                    </div>
                  </div>
                </div>
              </CardHeader>
              <CardContent className="pt-0">
                <div className="space-y-2">
                  <LibraryList callback={handleSelectExisting}></LibraryList>
                </div>
              </CardContent>
            </Card>
            {/* Create New Library */}
            <Card className="group hover:shadow-md transition-all duration-200 border-2 hover:border-primary/20">
              <CardHeader className="pb-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-3">
                    <div className="w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center">
                      <RiFolderAddLine size={20} className="text-primary" />
                    </div>
                    <div>
                      <CardTitle className="text-lg">Create new library</CardTitle>
                      <CardDescription className="text-sm">
                        Create a new Hestia library for organizing your files with tags.
                      </CardDescription>
                    </div>
                  </div>
                  <Button
                    onClick={handleCreateNew}
                    disabled={!newLibraryName.trim() || isCreating}
                    className="bg-primary hover:bg-primary/90 text-primary-foreground px-6"
                  >
                    {isCreating ? "Creating..." : "Create"}
                    <RiArrowRightLine size={16} className="ml-2" />
                  </Button>
                </div>
              </CardHeader>
              <CardContent className="pt-0">
                <div className="space-y-2">
                  <Label htmlFor="library-name" className="text-sm font-medium">
                    Library name
                  </Label>
                  <Input
                    id="library-name"
                    placeholder="My Library"
                    value={newLibraryName}
                    onChange={(e) => setNewLibraryName(e.target.value)}
                    onKeyDown={(e) => e.key === "Enter" && handleCreateNew()}
                    className="h-10"
                  />
                </div>
              </CardContent>
            </Card>


            {/* Cloud Library (Future) */}
            <Card className="group hover:shadow-md transition-all duration-200 border-2 hover:border-blue-500/20 opacity-60">
              <CardHeader className="pb-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-3">
                    <div className="w-10 h-10 rounded-full bg-blue-500/10 flex items-center justify-center">
                      <RiCloudLine size={20} className="text-blue-600 dark:text-blue-500" />
                    </div>
                    <div>
                      <CardTitle className="text-lg">Open library from cloud</CardTitle>
                      <CardDescription className="text-sm">
                        Set up a synced library with existing remote library.
                      </CardDescription>
                    </div>
                  </div>
                  <Button
                    onClick={handleOpenFromCloud}
                    disabled={true}
                    variant="outline"
                    className="border-muted-foreground/20 px-6"
                  >
                    Coming Soon
                  </Button>
                </div>
              </CardHeader>
            </Card>
          </div>

          {/* Footer */}
          <div className="mt-12 text-center">
            <Separator className="mb-6" />
            <div className="flex items-center justify-center space-x-6 text-sm text-muted-foreground/70">
              <button
                onClick={() => openPath("https://github.com/hestia-app/hestia")}
                className="hover:text-foreground transition-colors"
              >
                Documentation
              </button>
              <button
                onClick={() => openPath("https://github.com/hestia-app/hestia")}
                className="hover:text-foreground transition-colors"
              >
                GitHub
              </button>
              <button
                onClick={() => openPath("https://discord.gg/hestia")}
                className="hover:text-foreground transition-colors"
              >
                Community
              </button>
            </div>
          </div>
        </div>
      </QueryClientProvider>
    </div>
  );
}
