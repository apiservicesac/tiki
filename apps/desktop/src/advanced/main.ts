import { AdvancedSettingsApp } from "./AdvancedSettingsApp";

const root = document.getElementById("advanced-root");
if (!root) {
  throw new Error("The page is missing the #advanced-root element.");
}
void new AdvancedSettingsApp(root).start();
