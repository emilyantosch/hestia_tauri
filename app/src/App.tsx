import "./App.css";

import {
  InspectorPanelProvider,
  InspectorPanel,
} from "@/components/inspector-panel";
import Chat from "@/components/chat";
import Layout from "./Layout";

function App() {
  return (
    <div>
      <Layout>
        <header className="flex h-4 shrink-0 items-center gap-2 px-4 md:px-6 lg:px-8 bg-sidebar text-sidebar-foreground relative before:absolute before:inset-y-3 before:-left-px before:w-px before:bg-gradient-to-b before:from-white/5 before:via-white/15 before:to-white/5 before:z-50">
        </header>
        <InspectorPanelProvider>
          <div className="flex h-[calc(100svh-2rem)] bg-muted md:rounded-s-3xl md:group-peer-data-[state=collapsed]/sidebar-inset:rounded-s-none transition-all ease-in-out duration-300">
            <Chat />
            <InspectorPanel />
          </div>
        </InspectorPanelProvider>
      </Layout>
    </div>
  );
}

export default App;
