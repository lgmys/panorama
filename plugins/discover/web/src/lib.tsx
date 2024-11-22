import { createRoot } from "react-dom/client"
import {Router} from "./App";

export interface PluginStartupConfig {
  pluginId: string;
  basename: string;
}

export const start = (container: HTMLElement, config: PluginStartupConfig) => {
  console.log('starting discover plugin', config);
  const root = createRoot(container);

  root.render(<Router basename={config.basename} />)
}
