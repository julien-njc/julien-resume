# Resume Editor

This app is a local Tauri + SvelteKit desktop UI for editing `../resume.json` and running the existing Docker resume build.

## Intended workflow

1. Edit the resume data in the form UI
2. Save back to `resume.json`
3. Run the existing `docker compose run --rm resume-builder` flow from the app
4. Iterate quickly on the generated ATS and styled outputs

## Install

From this folder:

```bash
npm install
```

## Run in dev mode

```bash
npm run tauri dev
```

## Build a desktop app

```bash
npm run tauri build
```

## Notes

- The app tries to auto-detect the parent resume repository by searching for `resume.json` and `compose.yaml`
- You can override the detected root path from the UI
- The Build button runs the existing Docker-based resume pipeline locally
