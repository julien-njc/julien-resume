# Resume Editor

This app is a local Tauri + SvelteKit desktop UI for editing `../resume.json` and running the existing Docker resume build.

The editor now works with a profile-aware resume model: shared skills and experience live in one inventory, and profiles select which items render for a specific target job.

## Intended workflow

1. Maintain the shared resume inventory in the form UI
2. Build a profile by selecting the right skills and experience for a target role
3. Save back to `resume.json`
4. Run the existing `docker compose run --rm resume-builder` flow from the app
5. Iterate quickly on the generated ATS and styled outputs

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
- The active rendered profile is controlled by `active_profile_id` in `resume.json`
