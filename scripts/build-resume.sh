#!/bin/sh

set -eu

INPUT_FILE="${INPUT_FILE:-Julien Pireaud.md}"
OUTPUT_FILE="${OUTPUT_FILE:-build/Julien_Pireaud_Resume.pdf}"

mkdir -p "$(dirname "$OUTPUT_FILE")"

pandoc "$INPUT_FILE" \
  --from gfm \
  --standalone \
  --css resume.css \
  --metadata title="Julien Pireaud Resume" \
  --pdf-engine=wkhtmltopdf \
  --pdf-engine-opt=--enable-local-file-access \
  -o "$OUTPUT_FILE"
