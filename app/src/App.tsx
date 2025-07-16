import "./App.css";
import { Check } from "lucide-react";

import { ComboBox, ComboBoxEntry } from "./components/molecules/combobox";
import Layout from "./Layout";
import FolderSidebar from "./components/comp-574";

const test_entry: ComboBoxEntry[] = [
  {
    value: "test",
    logo: Check,
    label: "test",
  },
];
function App() {
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
