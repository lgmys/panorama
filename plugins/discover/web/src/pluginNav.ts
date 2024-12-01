import {
  PLUGIN_EVENTS,
  PluginNavigationInit,
  PluginStartupConfig,
} from '@panorama/shared-types';

export const pluginNav: PluginNavigationInit = {
  pluginId: 'discover',
  items: [
    {
      label: 'Home',
      to: '/',
    },
    {
      label: 'Browse',
      to: '/browse',
    },
  ],
};

export const dispatchNavEvent = (config: PluginStartupConfig) => {
  window.dispatchEvent(
    new CustomEvent(PLUGIN_EVENTS.INIT_NAVIGATION, {
      detail: { ...pluginNav, pluginId: config.pluginId },
    }),
  );
};
