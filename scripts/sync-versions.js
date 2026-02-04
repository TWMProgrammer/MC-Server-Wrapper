import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.join(__dirname, '..');

function syncVersions() {
  const rootCargoPath = path.join(rootDir, 'Cargo.toml');
  const tauriCargoPath = path.join(rootDir, 'src-tauri', 'Cargo.toml');
  const tauriConfPath = path.join(rootDir, 'src-tauri', 'tauri.conf.json');
  const packageJsonPath = path.join(rootDir, 'package.json');

  try {
    // 1. Get version (from argument, environment, or root Cargo.toml)
    let version = process.argv[2] || process.env.npm_package_version;

    if (!version) {
      const rootCargo = fs.readFileSync(rootCargoPath, 'utf8');
      const versionMatch = rootCargo.match(/^version\s*=\s*"([^"]+)"/m);
      
      if (!versionMatch) {
        console.error('Could not find version in root Cargo.toml');
        process.exit(1);
      }
      
      version = versionMatch[1];
    } else {
      // If version is provided, update root Cargo.toml first
      let rootCargo = fs.readFileSync(rootCargoPath, 'utf8');
      rootCargo = rootCargo.replace(/^version\s*=\s*"([^"]+)"/m, `version = "${version}"`);
      fs.writeFileSync(rootCargoPath, rootCargo);
      console.log(`Updated ${rootCargoPath}`);
    }

    console.log(`Syncing version: ${version}`);

    // 2. Update src-tauri/Cargo.toml
    let tauriCargo = fs.readFileSync(tauriCargoPath, 'utf8');
    tauriCargo = tauriCargo.replace(/^version\s*=\s*"([^"]+)"/m, `version = "${version}"`);
    fs.writeFileSync(tauriCargoPath, tauriCargo);
    console.log(`Updated ${tauriCargoPath}`);

    // 3. Update src-tauri/tauri.conf.json
    const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'));
    tauriConf.version = version;
    fs.writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2));
    console.log(`Updated ${tauriConfPath}`);

    // 4. Update package.json
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
    packageJson.version = version;
    fs.writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2) + '\n');
    console.log(`Updated ${packageJsonPath}`);

    console.log('Successfully synced all versions!');
  } catch (error) {
    console.error('Error syncing versions:', error.message);
    process.exit(1);
  }
}

syncVersions();
