# Resume Build

This repository stores the resume source as structured data and generates both an ATS-friendly PDF and a styled PDF from that single source.

## Files

- `resume.json`: single source of truth for resume content
- `resume-ats.css`: styling for the ATS-friendly PDF
- `resume-stylish.css`: styling for the styled PDF
- `Dockerfile`: build image with `pandoc` and `wkhtmltopdf`
- `compose.yaml`: local build entrypoint
- `scripts/render_resume.py`: renders the structured source into ATS and styled Markdown files
- `scripts/build_ats_docx.py`: generates a tighter ATS `.docx` directly from the structured source
- `scripts/ats_smoke_test.py`: extracts text from `.docx` or `.pdf` and heuristically parses experience blocks
- `scripts/build-resume.sh`: shared build script used by Docker locally and in GitHub Actions
- `.github/workflows/release-resume.yml`: CI workflow that builds the PDF and publishes a GitHub release

## Local Build

Build the container image:

```bash
docker compose build resume-builder
```

Generate both PDFs:

```bash
docker compose run --rm resume-builder
```

The generated files are:

```bash
build/Julien_Pireaud_Resume_ATS.md
build/Julien_Pireaud_Resume_Styled.md
build/Julien_Pireaud_Resume_ATS.docx
build/Julien_Pireaud_Resume_ATS.pdf
build/Julien_Pireaud_Resume_Styled.pdf
```

## GitHub Actions

On each push to `main` that changes the resume or build files, GitHub Actions:

1. Checks out the repository
2. Builds the Docker image
3. Runs the shared resume build
4. Uploads the ATS DOCX, ATS PDF, and styled PDF as workflow artifacts
5. Creates a GitHub release tagged `resume-<commit-sha>` and attaches all generated deliverables

This keeps local builds and CI builds aligned by using the same Docker and shell entrypoint.

## Typical Update Flow

Edit the structured source:

```bash
resume.json
```

Rebuild locally:

```bash
docker compose run --rm resume-builder
```

Run the local ATS smoke test against the generated ATS `.docx`:

```bash
docker compose run --rm resume-builder python3 scripts/ats_smoke_test.py build/Julien_Pireaud_Resume_ATS.docx
```

Run it against the ATS `.pdf`:

```bash
docker compose run --rm resume-builder python3 scripts/ats_smoke_test.py build/Julien_Pireaud_Resume_ATS.pdf
```

The script prints extracted jobs and warnings for title, company, and bullet count mismatches compared with `resume.json`.

Commit and push:

```bash
git add .
git commit -m "Update resume"
git push origin main
```

After push, GitHub Actions will generate a new PDF and publish a new release for that commit.

## Notes

- Generated PDFs are ignored through `.gitignore`
- The release workflow accumulates one release per commit
- If Docker is not running locally, `docker compose` commands will fail until the daemon is available

## Desktop Editor

A local Tauri + SvelteKit editor now lives in `editor/`.

From that folder:

```bash
cd editor
npm install
npm run tauri dev
```

The editor can:

- load and save the parent `resume.json`
- edit the main resume sections with a graphical form
- trigger the existing Docker resume build locally
- show the build output in the app
