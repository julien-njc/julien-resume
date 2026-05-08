#!/bin/sh

set -eu

TITLE="$(python3 -c 'from scripts.resume_model import load_resume; print(load_resume()["name"])')"

python3 scripts/render_resume.py
python3 scripts/build_ats_docx.py

pandoc "build/resume_ats.md" \
  --from gfm \
  --standalone \
  --css resume-ats.css \
  --metadata pagetitle="$TITLE" \
  --pdf-engine=wkhtmltopdf \
  --pdf-engine-opt=--enable-local-file-access \
  -o "build/resume_ats.pdf"

pandoc "build/resume_styled.md" \
  --from gfm \
  --standalone \
  --css resume-stylish.css \
  --metadata pagetitle="$TITLE" \
  --pdf-engine=wkhtmltopdf \
  --pdf-engine-opt=--enable-local-file-access \
  -o "build/resume_styled.pdf"
