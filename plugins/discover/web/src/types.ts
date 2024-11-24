export interface PluginStartupConfig {
  pluginId: string;
  basename: string;
  /**
   * Started as a plugin
   */
  nested?: boolean;
}
