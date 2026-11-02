import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";

const root = process.cwd();
const sources = [
  {
    name: "frontend",
    directory: path.join(root, "workspace", "frontend"),
    required: ["package.json", "index.html", "src/main.jsx"],
  },
  {
    name: "backend",
    directory: path.join(root, "workspace", "backend"),
    required: ["main.py", "backend/db.py", "requirements.txt"],
  },
  {
    name: "revelaai",
    directory: path.join(root, "workspace", "revelaai"),
    required: ["asgi.py", "app.py", "requirements.txt"],
  },
];

const manifest = {
  application: "RevelaCode Desktop",
  createdAt: new Date().toISOString(),
  sources: {},
};

for (const source of sources) {
  if (!fs.existsSync(source.directory)) {
    throw new Error("Missing source checkout: " + source.name + " (" + source.directory + ")");
  }

  for (const relative of source.required) {
    const candidate = path.join(source.directory, relative);
    if (!fs.existsSync(candidate)) {
      throw new Error("Missing required " + source.name + " file: " + relative);
    }
  }

  const sha = execFileSync("git", ["-C", source.directory, "rev-parse", "HEAD"], {
    encoding: "utf8",
  }).trim();

  manifest.sources[source.name] = {
    directory: path.relative(root, source.directory).replaceAll(path.sep, "/"),
    commit: sha,
  };
  console.log(source.name + ": " + sha);
}

const output = path.join(root, "artifacts", "source-revisions.json");
fs.mkdirSync(path.dirname(output), { recursive: true });
fs.writeFileSync(output, JSON.stringify(manifest, null, 2) + "\n", "utf8");
console.log("Wrote " + path.relative(root, output));
