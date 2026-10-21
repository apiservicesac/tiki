// Writes updates/latest.json, the file installed copies of Tiki read to find a new version.
// Usage (inside the ui container): node tools/update-manifest.mjs <version> <notes>
import { readFileSync, readdirSync, writeFileSync, mkdirSync } from "node:fs";
import { join } from "node:path";

class UpdateManifest {
  static REPOSITORY = "https://github.com/apiservicesac/tiki";
  static BUNDLE_FOLDER = "target/x86_64-pc-windows-msvc/release/bundle/nsis";
  static OUTPUT_FILE = "updates/latest.json";

  constructor(version, notes) {
    this.version = version;
    this.notes = notes;
  }

  write() {
    const installer = this.installerName();
    const manifest = {
      version: this.version,
      notes: this.notes,
      pub_date: new Date().toISOString(),
      platforms: {
        "windows-x86_64": {
          signature: readFileSync(join(UpdateManifest.BUNDLE_FOLDER, `${installer}.sig`), "utf8").trim(),
          url: `${UpdateManifest.REPOSITORY}/releases/download/v${this.version}/${installer}`,
        },
      },
    };
    mkdirSync("updates", { recursive: true });
    writeFileSync(UpdateManifest.OUTPUT_FILE, `${JSON.stringify(manifest, null, 2)}\n`);
    console.log(`Wrote ${UpdateManifest.OUTPUT_FILE} for ${installer}`);
  }

  installerName() {
    const installer = readdirSync(UpdateManifest.BUNDLE_FOLDER).find(
      (name) => name.endsWith("-setup.exe") && name.includes(this.version),
    );
    if (!installer) {
      throw new Error(`No installer for ${this.version} in ${UpdateManifest.BUNDLE_FOLDER}`);
    }
    return installer;
  }
}

const [version, notes = ""] = process.argv.slice(2);
if (!version) {
  throw new Error("Usage: node tools/update-manifest.mjs <version> <notes>");
}
new UpdateManifest(version, notes).write();
