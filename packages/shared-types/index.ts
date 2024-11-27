export interface PluginNavigationItem {
  label: string;
  to: string;
  children?: PluginNavigationItem[];
}

export interface PluginNavigationInit {
  items: PluginNavigationItem[];
  pluginId: string;
}

export interface PluginNavigate {
  to: string;
}

export const PLUGIN_EVENTS = {
  INIT_NAVIGATION: 'plugin:initNavigation',
  NAVIGATE: 'plugin:navigate',
  UNLOAD: 'plugin:unload',
  LOADING: 'plugin:loading',
};

export interface PluginStartupConfig {
  pluginId: string;
  basename: string;
  /**
   * Started as a plugin
   */
  nested?: boolean;
}
