#!/bin/sh

set -eu

python3 scripts/render_resume.py
python3 scripts/build_ats_docx.py

pandoc "build/Julien_Pireaud_Resume_ATS.md" \
  --from gfm \
  --standalone \
  --css resume-ats.css \
  --metadata title="Julien Pireaud Resume ATS" \
  --pdf-engine=wkhtmltopdf \
  --pdf-engine-opt=--enable-local-file-access \
  -o "build/Julien_Pireaud_Resume_ATS.pdf"

pandoc "build/Julien_Pireaud_Resume_Styled.md" \
  --from gfm \
  --standalone \
  --css resume-stylish.css \
  --metadata title="Julien Pireaud Resume" \
  --pdf-engine=wkhtmltopdf \
  --pdf-engine-opt=--enable-local-file-access \
  -o "build/Julien_Pireaud_Resume_Styled.pdf"
