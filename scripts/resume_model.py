#!/usr/bin/env python3

import json
from copy import deepcopy
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "resume.json"


def _is_legacy(data: dict) -> bool:
    return "basics" not in data


def _profile_index(profiles: list[dict]) -> dict[str, dict]:
    return {profile["id"]: profile for profile in profiles}


def _resolve_skills(skills: list[dict], active_profile_id: str) -> dict[str, list[str]]:
    grouped: dict[str, list[str]] = {}
    for skill in skills:
        if active_profile_id not in skill.get("profile_ids", []):
            continue
        grouped.setdefault(skill["category"], []).append(skill["label"])
    return grouped


def _resolve_experience(experience: list[dict], active_profile_id: str) -> list[dict]:
    resolved = []
    for entry in experience:
        if active_profile_id not in entry.get("profile_ids", []):
            continue
        merged = deepcopy(entry)
        override = merged.pop("overrides", {}).get(active_profile_id, {})
        for key, value in override.items():
            merged[key] = value
        merged.pop("profile_ids", None)
        resolved.append(merged)
    return resolved


def _resolve_education(education: list[dict], active_profile_id: str) -> list[dict]:
    return [
        {k: v for k, v in entry.items() if k not in ("profile_ids", "id")}
        for entry in education
        if active_profile_id in entry.get("profile_ids", [])
    ]


def _resolve_languages(languages: list[dict], active_profile_id: str) -> list[str]:
    result = []
    for lang in languages:
        if active_profile_id not in lang.get("profile_ids", []):
            continue
        proficiency = lang.get("proficiency", "")
        result.append(f"{lang['name']} ({proficiency})" if proficiency else lang["name"])
    return result


def load_resume(path: Path = SOURCE, profile_id: str | None = None) -> dict:
    data = json.loads(path.read_text())
    if _is_legacy(data):
        return data

    basics = data["basics"]
    profiles = _profile_index(data["profiles"])
    active_profile_id = profile_id or data["active_profile_id"]
    if active_profile_id not in profiles:
        raise ValueError(f"unknown profile id: {active_profile_id}")

    profile = profiles[active_profile_id]

    return {
        **basics,
        "summary": profile["summary"],
        "skills": _resolve_skills(data["skills"], active_profile_id),
        "experience": _resolve_experience(data["experience"], active_profile_id),
        "education": _resolve_education(data.get("education", []), active_profile_id),
        "languages": _resolve_languages(data.get("languages", []), active_profile_id),
        "active_profile": {
            "id": profile["id"],
            "label": profile["label"],
        },
    }
