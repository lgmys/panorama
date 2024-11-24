import { createRoot } from 'react-dom/client';
import { Router } from './App';
import { dispatchNavEvent } from './pluginNav';
import { PluginStartupConfig } from './types';

export const createRouter = (
  container: HTMLElement,
  config: PluginStartupConfig,
) => {
  console.log('starting discover plugin', config);

  dispatchNavEvent(config);

  const root = createRoot(container);

  root.render(<Router basename={config.basename} />);

  return root;
};
