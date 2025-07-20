import "./App.css";
import { Check } from "lucide-react";

import { ComboBox, ComboBoxEntry } from "./components/molecules/combobox";
import Layout from "./Layout";
import FolderSidebar from "./components/comp-574";
import { invoke } from '@tauri-apps/api/core';

const test_entry: ComboBoxEntry[] = [
  {
    value: "test",
    logo: Check,
    label: "test",
  },
];
function App() {
  let folder_tree = invoke('get_folder_tree')
    .then((msg) => console.log(msg))
    .catch((e) => console.log(e))
    .finally(() => console.log("Invoke concluded"));
  return (
    <div>
      <Layout>
        <ComboBox list={test_entry} title="Vault" />
        <FolderSidebar></FolderSidebar>
      </Layout>
    </div>
  );
}

export default App;
