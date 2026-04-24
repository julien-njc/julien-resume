#!/bin/sh

set -eu

TITLE="$(python3 -c 'import json; print(json.load(open("resume.json"))["name"])')"

python3 scripts/render_resume.py
python3 scripts/build_ats_docx.py

pandoc "build/Julien_Pireaud_Resume_ATS.md" \
  --from gfm \
  --standalone \
  --css resume-ats.css \
  --metadata pagetitle="$TITLE" \
  --pdf-engine=wkhtmltopdf \
  --pdf-engine-opt=--enable-local-file-access \
  -o "build/Julien_Pireaud_Resume_ATS.pdf"

pandoc "build/Julien_Pireaud_Resume_Styled.md" \
  --from gfm \
  --standalone \
  --css resume-stylish.css \
  --metadata pagetitle="$TITLE" \
  --pdf-engine=wkhtmltopdf \
  --pdf-engine-opt=--enable-local-file-access \
  -o "build/Julien_Pireaud_Resume.pdf"
