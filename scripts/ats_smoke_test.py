#!/usr/bin/env python3

import json
import re
import subprocess
import sys
import xml.etree.ElementTree as ET
from pathlib import Path
from zipfile import ZipFile


ROOT = Path(__file__).resolve().parent.parent
DEFAULT_SOURCE = ROOT / "resume.json"
SECTION_ENDERS = {"education", "languages"}
DATE_RE = re.compile(
    r"\b(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[a-z]*\s+\d{4}\s*[-|]\s*(?:"
    r"(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[a-z]*\s+\d{4}|Current|Present"
    r")\b",
    re.IGNORECASE,
)


def extract_docx(path: Path) -> list[str]:
    ns = {"w": "http://schemas.openxmlformats.org/wordprocessingml/2006/main"}
    with ZipFile(path) as archive:
        xml_bytes = archive.read("word/document.xml")
    root = ET.fromstring(xml_bytes)
    lines = []
    for paragraph in root.findall(".//w:p", ns):
        texts = [node.text or "" for node in paragraph.findall(".//w:t", ns)]
        lines.append("".join(texts).strip())
    return lines


def extract_pdf(path: Path) -> list[str]:
    result = subprocess.run(
        ["pdftotext", str(path), "-"],
        check=True,
        capture_output=True,
        text=True,
    )
    return [line.strip() for line in result.stdout.splitlines()]


def extract_text(path: Path) -> list[str]:
    suffix = path.suffix.lower()
    if suffix == ".docx":
        return extract_docx(path)
    if suffix == ".pdf":
        return extract_pdf(path)
    return path.read_text().splitlines()


def compact(lines: list[str]) -> list[str]:
    return [line.strip() for line in lines]


def find_section(lines: list[str], name: str) -> int | None:
    needle = name.lower()
    for idx, line in enumerate(lines):
        normalized = line.strip().lstrip("#").strip().lower()
        if normalized == needle:
            return idx
    return None


def split_bullets(lines: list[str]) -> list[str]:
    bullets = []
    for line in lines:
        stripped = line.strip()
        if not stripped:
            continue
        if stripped.startswith(("- ", "* ", "• ")):
            bullets.append(stripped[2:].strip())
            continue
        if " - " in stripped:
            pieces = [piece.strip() for piece in re.split(r"\s+-\s+", stripped) if piece.strip()]
            if len(pieces) > 1:
                bullets.extend(pieces)
                continue
        bullets.append(stripped)
    return bullets


def parse_header_blob(lines: list[str]) -> dict:
    cleaned = [line.strip() for line in lines if line.strip()]
    job = {"raw_header": cleaned, "title": None, "company": None, "location": None, "dates": None}

    if len(cleaned) >= 2 and DATE_RE.search(cleaned[1]) and "," in cleaned[0]:
        left, _, right = cleaned[0].partition(",")
        job["title"] = left.strip()
        job["company"] = right.strip()
        location, _, dates = cleaned[1].partition("|")
        job["location"] = location.strip()
        job["dates"] = dates.strip() or cleaned[1].strip()
        return job

    labeled = {}
    for line in cleaned:
        if ":" in line:
            key, value = line.split(":", 1)
            labeled[key.strip().lower()] = value.strip()

    if labeled:
        job["title"] = labeled.get("title") or labeled.get("job title")
        job["company"] = labeled.get("company") or labeled.get("employer")
        job["location"] = labeled.get("location")
        job["dates"] = labeled.get("dates")
        return job

    if len(cleaned) >= 4:
        job["title"] = cleaned[0]
        job["company"] = cleaned[1]
        job["location"] = cleaned[2]
        job["dates"] = cleaned[3]
        return job

    if cleaned:
        job["title"] = cleaned[0]
    return job


def parse_experience(lines: list[str]) -> list[dict]:
    start = find_section(lines, "Professional Experience")
    if start is None:
        return []

    jobs = []
    i = start + 1
    n = len(lines)
    while i < n:
        line = lines[i].strip()
        normalized = line.lstrip("#").strip().lower()
        if normalized in SECTION_ENDERS:
            break
        if not line:
            i += 1
            continue

        header_lines = []
        while i < n:
            line = lines[i].strip()
            normalized = line.lstrip("#").strip().lower()
            if normalized in SECTION_ENDERS:
                break
            if not line:
                if header_lines:
                    i += 1
                    break
                i += 1
                continue
            if line.startswith(("- ", "* ", "• ")):
                break
            header_lines.append(line)
            i += 1

        bullet_lines = []
        while i < n:
            line = lines[i].strip()
            normalized = line.lstrip("#").strip().lower()
            if normalized in SECTION_ENDERS:
                break
            if not line:
                if bullet_lines:
                    i += 1
                    break
                i += 1
                continue
            if header_lines and not line.startswith(("- ", "* ", "• ")) and DATE_RE.search(line):
                break
            if header_lines and not line.startswith(("- ", "* ", "• ")) and not bullet_lines and len(header_lines) < 4:
                break
            bullet_lines.append(line)
            i += 1

        if header_lines:
            job = parse_header_blob(header_lines)
            job["bullets"] = split_bullets(bullet_lines)
            jobs.append(job)
        else:
            i += 1

    return jobs


def load_expected() -> list[dict]:
    if not DEFAULT_SOURCE.exists():
        return []
    data = json.loads(DEFAULT_SOURCE.read_text())
    return data.get("experience", [])


def compare(jobs: list[dict], expected: list[dict]) -> list[str]:
    warnings = []
    if expected and len(jobs) != len(expected):
        warnings.append(f"expected {len(expected)} jobs, extracted {len(jobs)}")

    for idx, job in enumerate(jobs):
        if idx >= len(expected):
            break
        target = expected[idx]
        if job.get("company") != target.get("employer"):
            warnings.append(
                f"job {idx + 1} company mismatch: extracted '{job.get('company')}' expected '{target.get('employer')}'"
            )
        if job.get("title") != target.get("job_title"):
            warnings.append(
                f"job {idx + 1} title mismatch: extracted '{job.get('title')}' expected '{target.get('job_title')}'"
            )
        if len(job.get("bullets", [])) != len(target.get("bullets", [])):
            warnings.append(
                f"job {idx + 1} bullet count mismatch: extracted {len(job.get('bullets', []))} expected {len(target.get('bullets', []))}"
            )
    return warnings


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: ats_smoke_test.py <resume.docx|resume.pdf|resume.md>", file=sys.stderr)
        return 2

    path = Path(sys.argv[1]).resolve()
    lines = compact(extract_text(path))
    jobs = parse_experience(lines)
    warnings = compare(jobs, load_expected())

    print(json.dumps({"file": str(path), "jobs": jobs, "warnings": warnings}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
