# Resume Build

This repository stores the resume source in Markdown and generates a PDF version from it.

## Files

- `Julien Pireaud.md`: main resume source
- `resume.css`: PDF styling used during rendering
- `Dockerfile`: build image with `pandoc` and `wkhtmltopdf`
- `compose.yaml`: local build entrypoint
- `scripts/build-resume.sh`: shared build script used by Docker locally and in GitHub Actions
- `.github/workflows/release-resume.yml`: CI workflow that builds the PDF and publishes a GitHub release

## Local Build

Build the container image:

```bash
docker compose build resume-builder
```

Generate the PDF:

```bash
docker compose run --rm resume-builder
```

The output file is:

```bash
build/Julien_Pireaud_Resume.pdf
```

## Custom Input Or Output

You can override the input or output path with environment variables:

```bash
docker compose run --rm \
  -e INPUT_FILE="Julien Pireaud.md" \
  -e OUTPUT_FILE="build/custom-resume.pdf" \
  resume-builder
```

## GitHub Actions

On each push to `main` that changes the resume or build files, GitHub Actions:

1. Checks out the repository
2. Builds the Docker image
3. Runs the shared resume build
4. Uploads the generated PDF as a workflow artifact
5. Creates a GitHub release tagged `resume-<commit-sha>` and attaches the PDF

This keeps local builds and CI builds aligned by using the same Docker and shell entrypoint.

## Typical Update Flow

Edit the resume:

```bash
Julien Pireaud.md
```

Rebuild locally:

```bash
docker compose run --rm resume-builder
```

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
