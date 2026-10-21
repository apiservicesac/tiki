import { TikiApp } from "./TikiApp";

const root = document.getElementById("island-root");
if (!root) {
  throw new Error("The page is missing the #island-root element.");
}
void new TikiApp(root).start();
