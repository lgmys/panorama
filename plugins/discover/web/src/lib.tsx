import { createRoot } from "react-dom/client"
import App from "./App";

export const start = (container: HTMLElement) => {
  const root = createRoot(container);

  root.render(<App />)
}
