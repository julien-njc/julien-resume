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


def _entry_index(entries: list[dict]) -> dict[str, dict]:
    return {entry["id"]: entry for entry in entries}


def _resolve_skills(skills: list[dict], selected_ids: list[str]) -> dict[str, list[str]]:
    skill_map = _entry_index(skills)
    grouped: dict[str, list[str]] = {}
    for skill_id in selected_ids:
        skill = skill_map.get(skill_id)
        if not skill:
            continue
        grouped.setdefault(skill["category"], []).append(skill["label"])
    return grouped


def _resolve_experience(
    experience: list[dict],
    selected_ids: list[str],
    overrides: dict[str, dict],
) -> list[dict]:
    experience_map = _entry_index(experience)
    resolved = []
    for experience_id in selected_ids:
        role = experience_map.get(experience_id)
        if not role:
            continue
        merged = deepcopy(role)
        for key, value in overrides.get(experience_id, {}).items():
            merged[key] = value
        resolved.append(merged)
    return resolved


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
        "skills": _resolve_skills(data["skills"], profile.get("skill_ids", [])),
        "experience": _resolve_experience(
            data["experience"],
            profile.get("experience_ids", []),
            profile.get("experience_overrides", {}),
        ),
        "education": data["education"],
        "languages": data["languages"],
        "active_profile": {
            "id": profile["id"],
            "label": profile["label"],
        },
    }
