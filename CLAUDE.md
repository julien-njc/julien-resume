# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This System Is

A unified resume engine and public website. One source of truth (`resume.json`) feeds a Tauri desktop editor, generates ATS and styled PDF/DOCX outputs, and powers a public SvelteKit website — all filtered through **profiles** (role-oriented resume variants: iOS Engineer, Data Engineer, Development Manager, etc.).

The public website will be merged in from the `julienpireaud` SvelteKit repo. This repo will be renamed.

---

## Commands

### PDF/DOCX Build (Docker)

```bash
docker compose build resume-builder
docker compose run --rm resume-builder
docker compose run --rm resume-builder python3 scripts/ats_smoke_test.py build/Julien_Pireaud_Resume_ATS.docx
docker compose run --rm resume-builder python3 scripts/ats_smoke_test.py build/Julien_Pireaud_Resume_ATS.pdf
```

Output: `build/` — ATS Markdown, Styled Markdown, ATS DOCX, ATS PDF, Styled PDF.
Active rendered profile controlled by `active_profile_id` in `resume.json`.

### Desktop Editor (Tauri + SvelteKit)

```bash
cd editor && npm install && npm run tauri dev
```

To keep Rust crates and NPM packages in sync when Tauri releases a new version:

```bash
# 1. Update npm packages
cd editor && npm install @tauri-apps/api@latest @tauri-apps/cli@latest @tauri-apps/plugin-dialog@latest

# 2. Pin Cargo.toml versions to match (tauri and tauri-plugin-dialog; tauri-build follows its own track)
#    Then update the lockfile:
cd editor/src-tauri && cargo update tauri tauri-build tauri-plugin-dialog
```

### CI

`.github/workflows/release-resume.yml` — triggers on pushes to `main` that touch resume or build files, runs Docker build, publishes GitHub release with all generated files.

---

## Folder-Based Data Layout

User selects a root folder on first launch; path is persisted in Tauri app settings. All reads/writes go through this folder.

```
[user-selected folder]/
  resume.json           ← English master (source of truth, no en/ subfolder)
  applications.json     ← job tracker with JSON snapshots (editor-only, never public)
  fr/
    resume.json         ← French mirror of translatable fields, user-editable
  [lang]/
    resume.json         ← one subfolder per additional language
  build/                ← generated PDF/DOCX
```

**No `en/` folder.** English IS `resume.json` at root. Language subfolders hold AI-translated, user-editable mirrors. Only text fields are translated — IDs, URLs, and dates are never translated.

---

## Core Schema Design

### Item-Centric Profile Association

**Every item in every section has `profile_ids: string[]`.** If this array is empty or absent, the item is excluded from all resume outputs. Section headers are omitted when no items pass the active profile filter.

Profiles no longer maintain item lists (`skill_ids[]`, `experience_ids[]` are gone). Instead, each item declares its own profile membership. This makes the UI straightforward — look at any item, see which profiles it belongs to, toggle in/out.

```json
// Profile — defines identity only
{ "id": "ios-engineer", "label": "iOS / Product Engineering", "summary": "...", "chips": ["SwiftUI", "UIKit"] }

// Item — declares membership and profile-specific overrides
{
  "id": "sensopia",
  "profile_ids": ["ios-engineer", "applied-science"],
  "employer": "Sensopia Inc",
  "bullets": ["default bullets..."],
  "overrides": {
    "ios-engineer": { "bullets": ["ios-specific bullets..."] }
  }
}
```

### Section Inventory

| Section | Key | Singleton? | Overrides? |
|---------|-----|-----------|-----------|
| Contact info | `basics` | yes — always included | — |
| Default summary | `summary` | yes — overridden per profile | profile-level `summary` |
| Work history | `experience[]` | no | bullet-level per profile |
| Projects | `projects[]` | no | bullet-level per profile |
| Skills | `skills[]` | no | — (atomic) |
| Education | `education[]` | no | — |
| Certifications | `certifications[]` | no | — |
| Awards | `awards[]` | no | — |
| Publications | `publications[]` | no | — |
| Volunteer | `volunteer[]` | no | bullet-level per profile |
| References | `references[]` | no | — ; output can render as "available upon request" |
| Spoken languages | `languages[]` | no | — |
| Writing style | `writing_style` | yes — singleton | — |

### TypeScript types (`editor/src/lib/types.ts`)

```typescript
// Every list item in any section
interface ProfileScoped {
  profile_ids: string[];
}

interface ResumeData {
  basics: BasicsData;
  writing_style: WritingStyleData;
  active_profile_id: string;
  profiles: ResumeProfile[];
  summary?: string;                   // default; overridden by active profile
  experience: ExperienceEntry[];
  projects: ProjectEntry[];
  skills: SkillEntry[];
  education: EducationEntry[];
  certifications: CertificationEntry[];
  awards: AwardEntry[];
  publications: PublicationEntry[];
  volunteer: VolunteerEntry[];
  references: ReferenceEntry[];
  languages: SpokenLanguageEntry[];
}

interface ResumeProfile {
  id: string;
  label: string;
  summary: string;
  chips?: string[];                   // highlight keywords for hero section
}

interface WritingStyleData {
  context: string;                    // user-authored; injected into all AI prompts
}

interface SkillEntry extends ProfileScoped {
  id: string;
  category: string;
  label: string;
  years_of_experience?: number;
  usage_examples?: string[];
}

interface ExperienceEntry extends ProfileScoped {
  id: string;
  employer: string;
  job_title: string;
  location: string;
  dates: string;
  additional_role?: string;
  bullets: string[];
  overrides?: Record<string, ExperienceOverride>;   // profile_id → override
}

interface ExperienceOverride {
  job_title?: string;
  additional_role?: string;
  bullets?: string[];
}

interface ProjectEntry extends ProfileScoped {
  id: string;
  name: string;
  url?: string;
  role?: string;
  dates?: string;
  bullets: string[];
  overrides?: Record<string, { bullets?: string[] }>;
}

interface EducationEntry extends ProfileScoped {
  id: string;
  degree: string;
  field_of_study: string;
  school: string;
  graduation_date: string;
  notes?: string;
}

interface CertificationEntry extends ProfileScoped {
  id: string;
  name: string;
  issuer: string;
  date: string;
  expiry?: string;
  url?: string;
}

interface AwardEntry extends ProfileScoped {
  id: string;
  title: string;
  issuer: string;
  date: string;
  description?: string;
}

interface PublicationEntry extends ProfileScoped {
  id: string;
  title: string;
  publisher: string;
  date: string;
  url?: string;
  description?: string;
}

interface VolunteerEntry extends ProfileScoped {
  id: string;
  organization: string;
  role: string;
  dates: string;
  bullets: string[];
  overrides?: Record<string, { bullets?: string[] }>;
}

interface ReferenceEntry extends ProfileScoped {
  id: string;
  name: string;
  title: string;
  company: string;
  relationship?: string;
  email?: string;
  phone?: string;
  notes?: string;
}

interface SpokenLanguageEntry extends ProfileScoped {
  id: string;
  name: string;
  proficiency: string;
}
```

---

## Translation Workflow (bullet-level)

1. User edits English content → hits **Update** next to a translated field
2. AI re-translates that bullet from English using the configured model
3. **Bullet-level comparison view opens**:
   - **Left**: user's last saved translated version (with any prior manual edits)
   - **Right**: new AI translation
4. User edits the right side if needed, accepts → replaces `[lang]/resume.json`

The left side always shows previous manual corrections so the user can carry them forward across iterations without re-discovering them.

---

## Applications Tracker (`applications.json`)

Editor-only. Never rendered publicly. No PDF/DOCX stored — JSON snapshots only.

```json
{
  "applications": [
    {
      "id": "...",
      "job_url": "...",
      "job_title": "...",
      "company": "...",
      "date_applied": "...",
      "profile_id": "...",
      "language": "en",
      "match_score": 0.0,
      "cover_letter_submitted": "...",
      "notes": "...",
      "resume_snapshot": {},       // full copy of resume.json at submission time
      "translated_snapshot": {}    // copy of [lang]/resume.json if language != "en"; else null
    }
  ]
}
```

**Job matching flow:** paste URL → AI fetches/analyzes → match score → user selects profile → generates cover letter (always injects `writing_style.context`) → user reviews everything → hits explicit **Keep** button → snapshots are taken and record saved to `applications.json`.

The Keep button is the only trigger for snapshot creation. Nothing is saved automatically during the workflow.

---

## AI Features

- **Job matching**: match score against the selected profile's filtered skills + experience
- **Cover letter**: per job × profile; `writing_style.context` always injected; fine-tune and regenerate with iteration tracking
- **Translation**: bullet-level, with comparison view before any saved content is replaced
- **Model**: user-configurable in editor settings (Anthropic SDK preferred)

---

## Public Website (to be merged in)

Currently in `julienpireaud/`. Will become a `website/` or `site/` subdirectory here.

- Static SvelteKit, `@sveltejs/adapter-static`, Firebase Hosting
- All content loaded **at build time** from `resume.json` via `+page.server.ts` — no runtime fetching, fully offline-generated HTML
- EN + FR; English is source of truth; `active_profile_id` determines the default profile shown
- Contact form: Web3Forms (`PUBLIC_WEB3FORMS_ACCESS_KEY`)
- Live deploy: CI on `main` only; local script (`npm run deploy:firebase`) is preview-only
- The hardcoded `src/lib/content/resume.ts` is replaced entirely by importing `resume.json` at build time

### Route structure

```
/[lang]/resume                             → server-side redirects to default profile URL
/[lang]/resume/profile/[profile_id]        → specific profile (shareable, canonical URL)
/[lang]/contact                            → contact form (unchanged)
```

Profile switcher sits at the **bottom of the page** as plain `<a href>` links to the profile URLs — no query params, no JS required for sharing.

`[lang]` ∈ `{en, fr}` enforced by `src/params/lang.ts`. `[profile_id]` validated against profiles in `resume.json`.

### Build-time prerendering of profile routes

```ts
// src/routes/[lang]/resume/profile/[profile_id]/+page.server.ts
import resumeData from '$lib/resume.json';

export const entries = () =>
  resumeData.profiles.map(p => ({ profile_id: p.id }));
```

This generates all `[lang] × [profile_id]` combinations at build time.

### Publish flow (from Tauri editor)

The editor has a **Publish** button that:
1. Reads the local `resume.json`
2. Calls the GitHub Contents API (`PUT /repos/{owner}/{repo}/contents/{path}`) to update the file and create a commit
3. GitHub Actions on the website repo triggers a build and deploys to Firebase Hosting

Auth: **Personal Access Token** (PAT) with `repo` scope, stored in the OS keychain. Configured once in editor settings: PAT, repo (`owner/repo`), branch (default: `main`), file path (default: `resume.json`).

---

## Key Design Rules

- **No `en/` folder.** English edits go directly into `resume.json`.
- **Profile membership is on items, not profiles.** `profile_ids: []` on any item means excluded from all outputs.
- **Section headers are omitted if no items pass the active profile filter.**
- **`writing_style.context` is always injected into AI prompts.** Never skip it — this is what keeps generated output in the user's voice.
- **Translation comparison always shows previous user edits on the left.** The user must be able to carry forward manual style corrections.
- **Applications store JSON snapshots, not generated files.** Resume + translated version at submission time.
- **Live deploys are CI-only.**
