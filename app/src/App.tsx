import "./App.css";
import { Check } from "lucide-react";

import { ComboBox, ComboBoxEntry } from "./components/molecules/combobox";
import Layout from "./Layout";

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
        <div></div>
      </Layout>
    </div>
  );
}

export default App;
