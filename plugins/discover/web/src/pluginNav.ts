import { PLUGIN_EVENTS, PluginNavigationInit } from '@panorama/shared-types';

export const pluginNav: PluginNavigationInit = {
  pluginId: 'discover',
  items: [
    {
      label: 'Home',
      to: '/',
    },
    {
      label: 'Test',
      to: '/test',
    },
  ],
};

export const dispatchNavEvent = () => {
  window.dispatchEvent(
    new CustomEvent(PLUGIN_EVENTS.INIT_NAVIGATION, { detail: pluginNav }),
  );
};
