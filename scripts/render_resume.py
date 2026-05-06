#!/usr/bin/env python3

from pathlib import Path

from resume_model import load_resume


ROOT = Path(__file__).resolve().parent.parent
BUILD_DIR = ROOT / "build"


def section(title: str) -> str:
    return f"## {title}\n"


def bullets(items) -> str:
    return "\n".join(f"- {item}" for item in items) + "\n"


def render_ats(data: dict) -> str:
    parts = [
        f"# {data['name']}\n",
        f"Address: {data['address']}\n",
        f"Email: {data['email']} | Phone: {data['phone']}\n",
        f"LinkedIn: {data['linkedin']}\n",
        f"GitHub: {data['github']}\n",
        f"Website: {data['website']}\n",
        section("Summary"),
        f"{data['summary']}\n",
        section("Technical Skills")
    ]

    for skill_name, skill_values in data["skills"].items():
        parts.append(f"{skill_name}: {', '.join(skill_values)}\n")

    parts.append(section("Professional Experience"))
    for role in data["experience"]:
        parts.append(f"{role['job_title']}, {role['employer']}\n")
        parts.append(f"{role['location']} | {role['dates']}\n")
        parts.append(bullets(role["bullets"]))

    parts.append(section("Education"))
    for edu in data["education"]:
        parts.append(f"Degree: {edu['degree']}\n")
        parts.append(f"Field of Study: {edu['field_of_study']}\n")
        parts.append(f"School: {edu['school']}\n")
        parts.append(f"Graduation Date: {edu['graduation_date']}\n")

    parts.extend([section("Languages"), bullets(data["languages"])])

    return "\n".join(parts).strip() + "\n"


def render_stylish(data: dict) -> str:
    parts = [
        f"# {data['name']}\n",
        f"{data['address']}\n",
        f"{data['email']} | {data['phone']}\n",
        f"LinkedIn: {data['linkedin']}\n",
        f"GitHub: {data['github']}\n",
        f"Website: {data['website']}\n",
        section("Summary"),
        f"{data['summary']}\n",
        section("Technical Skills")
    ]

    for skill_name, skill_values in data["skills"].items():
        parts.append(f"**{skill_name}:** {', '.join(skill_values)}\n")

    parts.append(section("Professional Experience"))
    for role in data["experience"]:
        parts.append(f"### {role['employer']}\n")
        parts.append(f"{role['job_title']} | {role['location']} | {role['dates']}\n")
        if role.get("additional_role"):
            parts.append(f"*Additional Role:* {role['additional_role']}\n")
        parts.append(bullets(role["bullets"]))

    parts.append(section("Education"))
    for edu in data["education"]:
        parts.append(f"**{edu['school']}**\n")
        parts.append(f"{edu['degree']}, {edu['field_of_study']} | {edu['graduation_date']}\n")

    parts.extend([section("Languages"), ", ".join(data["languages"]) + "\n"])

    return "\n".join(parts).strip() + "\n"


def main():
    data = load_resume()
    BUILD_DIR.mkdir(exist_ok=True)
    (BUILD_DIR / "Julien_Pireaud_Resume_ATS.md").write_text(render_ats(data))
    (BUILD_DIR / "Julien_Pireaud_Resume_Styled.md").write_text(render_stylish(data))


if __name__ == "__main__":
    main()
