export interface ConfigFile {
  name: string
  path: string
  format: 'Properties' | 'Yaml' | 'Toml' | 'Json'
}
