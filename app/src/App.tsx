import "./App.css";
import { invoke } from '@tauri-apps/api/core';

import {
  InspectorPanelProvider,
  InspectorPanel,
} from "@/components/inspector-panel";
import Chat from "@/components/chat";
import { LibrarySetup } from "@/components/library-setup";
import Layout from "./Layout";
import { PathLike } from "fs";

import { QueryClient, QueryClientProvider, useQuery } from '@tanstack/react-query';

const queryClient = new QueryClient();
function App() {
  return (
    <div>
      <QueryClientProvider client={queryClient}>
        <Startup></Startup>
      </QueryClientProvider>
    </div>
  );
}


type Library = {
  share_path: PathLike,
  config: LibraryConfig,
}

type LibraryConfig = {
  paths: LibraryPathConfig[],
}

type LibraryPathConfig = {
  name: String,
  path: PathLike
}



function Startup(): React.ReactNode {
  const { isPending, error, data, refetch } = useQuery({
    queryKey: ['library-paths'],
    queryFn: () => invoke('get_library_paths'),
    retry: false
  });

  if (isPending) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary mx-auto mb-4"></div>
          <p className="text-muted-foreground">Loading library...</p>
        </div>
      </div>
    )
  }

  if (error) {
    // Show LibrarySetup if no library paths are configured
    return (
      <LibrarySetup
        onLibraryCreated={() => {
          refetch();
        }}
      />
    )
  }

  // Check if data is empty or undefined - also show LibrarySetup
  const libraryPaths = data as LibraryPathConfig[] | undefined;
  if (!libraryPaths || libraryPaths.length === 0) {
    return (
      <LibrarySetup
        onLibraryCreated={() => {
          refetch();
        }}
      />
    )
  }

  return (
    <Layout>
      <header className="flex h-4 shrink-0 items-center gap-2 px-4 md:px-6 lg:px-8 bg-sidebar text-sidebar-foreground relative before:absolute before:inset-y-3 before:-left-px before:w-px before:bg-gradient-to-b before:from-white/5 before:via-white/15 before:to-white/5 before:z-50">
      </header>
      <InspectorPanelProvider>
        <div className="flex h-[calc(100svh-2rem)] md:rounded-s-3xl md:group-peer-data-[state=collapsed]/sidebar-inset:rounded-s-none transition-all ease-in-out duration-300">
          <Chat />
          <InspectorPanel />
        </div>
      </InspectorPanelProvider>
    </Layout>
  )
}

export default App;
