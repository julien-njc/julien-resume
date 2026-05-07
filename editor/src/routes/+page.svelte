<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import type {
    BasicsData,
    BuildResult,
    EducationEntry,
    ExperienceEntry,
    ExperienceOverride,
    GeneratedFile,
    ResumeData,
    ResumeProfile,
    SkillEntry,
    SpokenLanguageEntry,
  } from '$lib/types';

  // ── Blank constructors ──────────────────────────────────────────────────────

  const blankBasics = (): BasicsData => ({
    name: '', address: '', email: '', phone: '', linkedin: '', github: '', website: ''
  });

  const blankProfile = (id = 'default-profile', label = 'Default Profile'): ResumeProfile => ({
    id, label, summary: '', chips: []
  });

  const blankSkill = (): SkillEntry => ({
    id: '', category: '', label: '', profile_ids: [], years_of_experience: undefined, usage_examples: []
  });

  const blankExperience = (id = 'experience-1'): ExperienceEntry => ({
    id, employer: '', job_title: '', location: '', dates: '', additional_role: '',
    bullets: [''], profile_ids: [], overrides: {}
  });

  const blankEducation = (id = 'edu-1'): EducationEntry => ({
    id, degree: '', field_of_study: '', school: '', graduation_date: '', profile_ids: []
  });

  const blankLanguage = (): SpokenLanguageEntry => ({
    id: '', name: '', proficiency: '', profile_ids: []
  });

  const blankResume = (): ResumeData => ({
    basics: blankBasics(),
    writing_style: { context: '' },
    active_profile_id: 'default-profile',
    profiles: [blankProfile()],
    skills: [],
    experience: [],
    projects: [],
    education: [],
    certifications: [],
    awards: [],
    publications: [],
    volunteer: [],
    references: [],
    languages: []
  });

  // ── Schema detection & upgrade ──────────────────────────────────────────────

  function isV2(v: unknown): v is ResumeData {
    if (!v || typeof v !== 'object') return false;
    const d = v as Record<string, unknown>;
    if (!('basics' in d) || !('profiles' in d)) return false;
    const skills = d.skills as unknown[];
    if (!Array.isArray(skills) || skills.length === 0) return true;
    return 'profile_ids' in (skills[0] as object);
  }

  function isV1(v: unknown): boolean {
    if (!v || typeof v !== 'object') return false;
    const d = v as Record<string, unknown>;
    if (!('basics' in d) || !('profiles' in d)) return false;
    const profiles = d.profiles as unknown[];
    return Array.isArray(profiles) && profiles.length > 0 && 'skill_ids' in (profiles[0] as object);
  }

  // Migrates old profile-centric schema (skill_ids/experience_ids on profiles)
  // to item-centric schema (profile_ids on items).
  function upgradeV1(raw: Record<string, unknown>): ResumeData {
    const profiles = (raw.profiles as Array<Record<string, unknown>>) ?? [];
    const cleanProfiles: ResumeProfile[] = profiles.map(p => ({
      id: p.id as string,
      label: p.label as string,
      summary: (p.summary as string) ?? '',
      chips: (p.chips as string[]) ?? []
    }));
    const allProfileIds = cleanProfiles.map(p => p.id);

    const skills: SkillEntry[] = ((raw.skills as Array<Record<string, unknown>>) ?? []).map(s => ({
      id: s.id as string,
      category: s.category as string,
      label: s.label as string,
      profile_ids: [] as string[],
      years_of_experience: s.years_of_experience as number | undefined,
      usage_examples: (s.usage_examples as string[]) ?? []
    }));

    const experience: ExperienceEntry[] = ((raw.experience as Array<Record<string, unknown>>) ?? []).map(e => ({
      id: e.id as string,
      employer: e.employer as string,
      job_title: e.job_title as string,
      location: e.location as string,
      dates: e.dates as string,
      additional_role: e.additional_role as string | undefined,
      bullets: (e.bullets as string[]) ?? [],
      profile_ids: [] as string[],
      overrides: {} as Record<string, ExperienceOverride>
    }));

    for (const profile of profiles) {
      const pid = profile.id as string;
      const skillIds = (profile.skill_ids as string[]) ?? [];
      const expIds = (profile.experience_ids as string[]) ?? [];
      const expOverrides = (profile.experience_overrides as Record<string, ExperienceOverride>) ?? {};

      for (const s of skills) {
        if (skillIds.includes(s.id) && !s.profile_ids.includes(pid)) s.profile_ids.push(pid);
      }
      for (const e of experience) {
        if (expIds.includes(e.id)) {
          if (!e.profile_ids.includes(pid)) e.profile_ids.push(pid);
          if (expOverrides[e.id]) e.overrides![pid] = expOverrides[e.id];
        }
      }
    }

    const rawEdu = raw.education;
    const education: EducationEntry[] = Array.isArray(rawEdu)
      ? rawEdu.map((e: Record<string, unknown>, i: number) => ({
          id: (e.id as string) ?? `edu-${i + 1}`,
          degree: e.degree as string,
          field_of_study: e.field_of_study as string,
          school: e.school as string,
          graduation_date: e.graduation_date as string,
          notes: e.notes as string | undefined,
          profile_ids: (e.profile_ids as string[]) ?? allProfileIds
        }))
      : rawEdu && typeof rawEdu === 'object'
        ? [{ id: 'edu-1', ...(rawEdu as Record<string, unknown>), profile_ids: allProfileIds } as unknown as EducationEntry]
        : [];

    const rawLangs = raw.languages as unknown[];
    const languages: SpokenLanguageEntry[] = Array.isArray(rawLangs)
      ? rawLangs.map((l, i) => {
          if (typeof l === 'string') {
            const name = l.replace(/\s*\(.*\)$/, '').trim();
            const proficiency = l.match(/\((.+)\)$/)?.[1] ?? '';
            return { id: `lang-${i}`, name, proficiency, profile_ids: allProfileIds };
          }
          const le = l as Record<string, unknown>;
          return {
            id: (le.id as string) ?? `lang-${i}`,
            name: le.name as string,
            proficiency: (le.proficiency as string) ?? '',
            profile_ids: (le.profile_ids as string[]) ?? allProfileIds
          };
        })
      : [];

    return {
      basics: raw.basics as BasicsData,
      writing_style: (raw.writing_style as { context: string }) ?? { context: '' },
      active_profile_id: raw.active_profile_id as string,
      profiles: cleanProfiles,
      skills,
      experience,
      projects: [],
      education,
      certifications: [],
      awards: [],
      publications: [],
      volunteer: [],
      references: [],
      languages
    };
  }

  // ── State ───────────────────────────────────────────────────────────────────

  let dataFolder = '';
  let resume: ResumeData = blankResume();
  let selectedProfileIndex = 0;
  let selectedExperienceIndex = 0;
  let selectedEducationIndex = 0;
  let selectedSkillId: string | null = null;
  let notice = '';
  let appError = '';
  let buildLog = '';
  let generatedFiles: GeneratedFile[] = [];
  let busy = false;

  // Inline add state (replaces window.prompt)
  let addingProfile = false;
  let newProfileLabel = '';
  let addingSkill = false;
  let newSkillLabel = '';
  let newSkillCategory = 'General';   // '__new__' signals custom entry
  let newSkillCategoryName = '';      // typed when __new__ is selected

  // Category editing
  let editingCategoryName: string | null = null;
  let renamedCategoryValue = '';

  // Skill category editor (inside edit panel)
  let skillEditingNewCategory = false;
  let skillNewCategoryName = '';

  // Custom dropdown open state (one at a time)
  let openDropdown: 'addSkill' | 'skillEdit' | null = null;

  // ── History (undo / redo) ────────────────────────────────────────────────────
  const MAX_HISTORY = 60;
  let _dataLoaded = false;
  let _savedState = '';           // JSON of the last-persisted resume
  let _prevState = '';            // baseline for the next history entry
  let _historyTimer: ReturnType<typeof setTimeout> | null = null;
  let _suppressHistory = false;
  let history: string[] = [];
  let future: string[] = [];

  // Unsaved-changes flag: true whenever current state differs from last save.
  $: hasUnsavedChanges = _dataLoaded && JSON.stringify(resume) !== _savedState;

  // Coalescing history: record a snapshot 600 ms after the last change.
  $: if (_dataLoaded && !_suppressHistory) {
    void resume;
    if (_historyTimer !== null) clearTimeout(_historyTimer);
    _historyTimer = setTimeout(commitHistory, 600);
  }

  function commitHistory() {
    _historyTimer = null;
    const current = JSON.stringify(resume);
    if (_prevState && current !== _prevState) {
      history = [...history.slice(-(MAX_HISTORY - 1)), _prevState];
      future = [];
    }
    _prevState = current;
  }

  async function undo() {
    if (history.length === 0) return;
    _suppressHistory = true;
    if (_historyTimer !== null) { clearTimeout(_historyTimer); _historyTimer = null; }
    future = [JSON.stringify(resume), ...future.slice(0, MAX_HISTORY - 1)];
    const prev = history[history.length - 1];
    history = history.slice(0, -1);
    resume = JSON.parse(prev);
    _prevState = prev;
    hydrateDrafts();
    await tick();
    _suppressHistory = false;
  }

  async function redo() {
    if (future.length === 0) return;
    _suppressHistory = true;
    if (_historyTimer !== null) { clearTimeout(_historyTimer); _historyTimer = null; }
    history = [...history.slice(-(MAX_HISTORY - 1)), JSON.stringify(resume)];
    const next = future[0];
    future = future.slice(1);
    resume = JSON.parse(next);
    _prevState = next;
    hydrateDrafts();
    await tick();
    _suppressHistory = false;
  }

  // ── Utilities ───────────────────────────────────────────────────────────────

  function slugify(value: string): string {
    return (value.toLowerCase().trim().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '')) || 'item';
  }

  function uniqueId(base: string, existing: string[]): string {
    let candidate = base;
    let n = 2;
    while (existing.includes(candidate)) { candidate = `${base}-${n}`; n++; }
    return candidate;
  }

  function normalizeLines(value: string): string[] {
    return value.split('\n').map(l => l.trim()).filter(Boolean);
  }

  function allProfileIds(): string[] {
    return resume.profiles.map(p => p.id);
  }

  // ── Resume hydration ────────────────────────────────────────────────────────

  function hydrateDrafts() {
    resume.writing_style ??= { context: '' };
    resume.projects ??= [];
    resume.certifications ??= [];
    resume.awards ??= [];
    resume.publications ??= [];
    resume.volunteer ??= [];
    resume.references ??= [];

    if (resume.profiles.length === 0) resume.profiles = [blankProfile()];
    if (!resume.profiles.some(p => p.id === resume.active_profile_id)) {
      resume.active_profile_id = resume.profiles[0].id;
    }
    selectedProfileIndex = Math.max(0, Math.min(selectedProfileIndex, resume.profiles.length - 1));
    selectedExperienceIndex = Math.max(0, Math.min(selectedExperienceIndex, resume.experience.length - 1));
    selectedEducationIndex = Math.max(0, Math.min(selectedEducationIndex, resume.education.length - 1));
  }

  // ── Folder selection ────────────────────────────────────────────────────────

  async function chooseFolder() {
    const selected = await openDialog({ directory: true, multiple: false, title: 'Select Resume Data Folder' });
    if (!selected) return;
    dataFolder = selected as string;
    await invoke('save_app_config', { dataFolder });
    await loadData();
  }

  // ── Data loading / saving ───────────────────────────────────────────────────

  async function loadData() {
    if (!dataFolder) return;
    _dataLoaded = false;
    if (_historyTimer !== null) { clearTimeout(_historyTimer); _historyTimer = null; }
    history = [];
    future = [];
    busy = true;
    appError = '';
    notice = '';
    try {
      const raw = await invoke<string>('load_resume', { dataFolder });
      const parsed = JSON.parse(raw) as unknown;
      if (!parsed || typeof parsed !== 'object' || Object.keys(parsed as object).length === 0) {
        resume = blankResume();
        notice = 'New resume — fill in your details';
      } else if (isV2(parsed)) {
        resume = parsed;
        notice = `Loaded ${resume.basics.name || 'resume'}`;
      } else if (isV1(parsed)) {
        resume = upgradeV1(parsed as Record<string, unknown>);
        notice = 'Migrated resume to current schema';
      } else {
        resume = blankResume();
        notice = 'Unrecognised format — starting blank';
      }
      hydrateDrafts();
      await refreshGeneratedFiles();
    } catch (cause) {
      appError = `Load failed: ${String(cause)}`;
    } finally {
      busy = false;
      await tick();
      const initial = JSON.stringify(resume);
      _savedState = initial;
      _prevState = initial;
      _dataLoaded = true;
    }
  }

  async function saveResume() {
    if (!dataFolder) return;
    busy = true;
    appError = '';
    try {
      const payload = JSON.stringify(resume, null, 2) + '\n';
      await invoke('save_resume', { dataFolder, contents: payload });
      _savedState = JSON.stringify(resume);
      notice = 'Saved';
    } catch (cause) {
      appError = `Save failed: ${String(cause)}`;
    } finally {
      busy = false;
    }
  }

  async function refreshGeneratedFiles() {
    if (!dataFolder) return;
    try {
      generatedFiles = await invoke<GeneratedFile[]>('list_generated_files', { dataFolder });
    } catch {
      generatedFiles = [];
    }
  }

  async function buildResume() {
    // Flush pending history snapshot then persist before building.
    if (_historyTimer !== null) { clearTimeout(_historyTimer); _historyTimer = null; commitHistory(); }
    await saveResume();
    if (appError) return;
    busy = true;
    notice = '';
    appError = '';
    buildLog = '';
    try {
      const result = await invoke<BuildResult>('build_resume', { dataFolder });
      buildLog = [result.stdout, result.stderr].filter(Boolean).join('\n');
      generatedFiles = result.generated_files;
      notice = result.status === 0 ? 'Build completed' : `Build exited with status ${result.status}`;
    } catch (cause) {
      appError = `Build failed: ${String(cause)}`;
    } finally {
      busy = false;
    }
  }

  async function openGeneratedFile(file: GeneratedFile) {
    try {
      await invoke('open_generated_file', { dataFolder, path: file.path });
    } catch (cause) {
      appError = `Open failed: ${String(cause)}`;
    }
  }

  // ── Profile helpers ─────────────────────────────────────────────────────────

  function selectedProfile(): ResumeProfile | null {
    return resume.profiles[selectedProfileIndex] ?? null;
  }

  function addProfile() {
    if (!newProfileLabel.trim()) return;
    const id = uniqueId(slugify(newProfileLabel), resume.profiles.map(p => p.id));
    resume.profiles = [...resume.profiles, blankProfile(id, newProfileLabel.trim())];
    // Copy all items that belong to the currently selected profile into the new one
    const currentId = selectedProfile()?.id;
    if (currentId) {
      const allItems = [
        ...resume.skills, ...resume.experience, ...resume.education, ...resume.languages,
        ...resume.projects, ...resume.certifications, ...resume.awards,
        ...resume.publications, ...resume.volunteer, ...resume.references,
      ];
      for (const item of allItems) {
        if (item.profile_ids.includes(currentId) && !item.profile_ids.includes(id))
          item.profile_ids = [...item.profile_ids, id];
      }
    }
    selectedProfileIndex = resume.profiles.length - 1;
    newProfileLabel = '';
    addingProfile = false;
    resume = resume;
  }

  function removeSelectedProfile() {
    if (resume.profiles.length === 1) return;
    const removed = resume.profiles[selectedProfileIndex];
    resume.profiles = resume.profiles.filter((_, i) => i !== selectedProfileIndex);
    // Remove this profile from all item profile_ids
    for (const s of resume.skills) s.profile_ids = s.profile_ids.filter(id => id !== removed.id);
    for (const e of resume.experience) {
      e.profile_ids = e.profile_ids.filter(id => id !== removed.id);
      if (e.overrides) delete e.overrides[removed.id];
    }
    for (const ed of resume.education) ed.profile_ids = ed.profile_ids.filter(id => id !== removed.id);
    for (const l of resume.languages) l.profile_ids = l.profile_ids.filter(id => id !== removed.id);
    if (removed.id === resume.active_profile_id) resume.active_profile_id = resume.profiles[0].id;
    selectedProfileIndex = Math.max(0, selectedProfileIndex - 1);
    resume = resume;
  }

  function setActiveProfile(index: number) {
    const profile = resume.profiles[index];
    if (!profile) return;
    resume.active_profile_id = profile.id;
    selectedProfileIndex = index;
  }

  function updateSelectedProfileId(value: string) {
    const profile = selectedProfile();
    if (!profile) return;
    const oldId = profile.id;
    const nextId = uniqueId(
      slugify(value || profile.label),
      resume.profiles.filter(p => p !== profile).map(p => p.id)
    );
    // Propagate rename to all items
    for (const s of resume.skills) s.profile_ids = s.profile_ids.map(id => id === oldId ? nextId : id);
    for (const e of resume.experience) {
      e.profile_ids = e.profile_ids.map(id => id === oldId ? nextId : id);
      if (e.overrides?.[oldId]) { e.overrides[nextId] = e.overrides[oldId]; delete e.overrides[oldId]; }
    }
    for (const ed of resume.education) ed.profile_ids = ed.profile_ids.map(id => id === oldId ? nextId : id);
    for (const l of resume.languages) l.profile_ids = l.profile_ids.map(id => id === oldId ? nextId : id);
    if (resume.active_profile_id === oldId) resume.active_profile_id = nextId;
    profile.id = nextId;
    resume = resume;
  }

  // ── Skill helpers ───────────────────────────────────────────────────────────

  function profileIncludesSkill(skillId: string): boolean {
    const profile = selectedProfile();
    if (!profile) return false;
    return resume.skills.find(s => s.id === skillId)?.profile_ids.includes(profile.id) ?? false;
  }

  function toggleSkillForProfile(skillId: string, checked: boolean) {
    const profile = selectedProfile();
    if (!profile) return;
    const skill = resume.skills.find(s => s.id === skillId);
    if (!skill) return;
    if (checked) {
      if (!skill.profile_ids.includes(profile.id)) skill.profile_ids = [...skill.profile_ids, profile.id];
    } else {
      skill.profile_ids = skill.profile_ids.filter(id => id !== profile.id);
    }
    resume = resume;
  }

  // ── Skill helpers ───────────────────────────────────────────────────────────

  function categoriesInOrder(): string[] {
    const seen = new Set<string>();
    const result: string[] = [];
    for (const s of resume.skills) {
      if (s.category && !seen.has(s.category)) { seen.add(s.category); result.push(s.category); }
    }
    return result;
  }

  function addSkill() {
    if (!newSkillLabel.trim()) return;
    const finalCat = newSkillCategory === '__new__'
      ? (newSkillCategoryName.trim() || 'General')
      : (newSkillCategory.trim() || 'General');
    const id = uniqueId(slugify(`${finalCat}-${newSkillLabel}`), resume.skills.map(s => s.id));
    resume.skills = [...resume.skills, { id, category: finalCat, label: newSkillLabel.trim(), profile_ids: [], usage_examples: [] }];
    selectedSkillId = id;
    newSkillLabel = '';
    newSkillCategoryName = '';
    addingSkill = false;
  }

  function removeSkill(index: number) {
    resume.skills = resume.skills.filter((_, i) => i !== index);
  }

  function normalizeSkillId(index: number) {
    const s = resume.skills[index];
    if (!s) return;
    s.id = uniqueId(
      slugify(`${s.category || 'skill'}-${s.label || 'item'}`),
      resume.skills.filter((_, i) => i !== index).map(sk => sk.id)
    );
    resume = resume;
  }

  function moveSkillLeft(skillId: string) {
    const idx = resume.skills.findIndex(s => s.id === skillId);
    if (idx <= 0) return;
    const cat = resume.skills[idx].category;
    let prev = idx - 1;
    while (prev >= 0 && resume.skills[prev].category !== cat) prev--;
    if (prev < 0) return;
    const arr = [...resume.skills];
    [arr[prev], arr[idx]] = [arr[idx], arr[prev]];
    resume.skills = arr;
  }

  function moveSkillRight(skillId: string) {
    const idx = resume.skills.findIndex(s => s.id === skillId);
    if (idx < 0) return;
    const cat = resume.skills[idx].category;
    let next = idx + 1;
    while (next < resume.skills.length && resume.skills[next].category !== cat) next++;
    if (next >= resume.skills.length) return;
    const arr = [...resume.skills];
    [arr[idx], arr[next]] = [arr[next], arr[idx]];
    resume.skills = arr;
  }

  function moveCategoryUp(category: string) {
    const cats = categoriesInOrder();
    const ci = cats.indexOf(category);
    if (ci <= 0) return;
    const prev = cats[ci - 1];
    const catSkills = resume.skills.filter(s => s.category === category);
    const rest = resume.skills.filter(s => s.category !== category);
    const insertAt = rest.findIndex(s => s.category === prev);
    rest.splice(insertAt, 0, ...catSkills);
    resume.skills = rest;
  }

  function moveCategoryDown(category: string) {
    const cats = categoriesInOrder();
    const ci = cats.indexOf(category);
    if (ci >= cats.length - 1) return;
    const next = cats[ci + 1];
    const catSkills = resume.skills.filter(s => s.category === category);
    const rest = resume.skills.filter(s => s.category !== category);
    let insertAt = rest.findIndex(s => s.category === next);
    while (insertAt < rest.length && rest[insertAt].category === next) insertAt++;
    rest.splice(insertAt, 0, ...catSkills);
    resume.skills = rest;
  }

  function renameCategory(oldName: string) {
    const newName = renamedCategoryValue.trim();
    if (newName && newName !== oldName) {
      for (const s of resume.skills) { if (s.category === oldName) s.category = newName; }
      resume = resume;
    }
    editingCategoryName = null;
  }

  function applySkillNewCategory(idx: number) {
    const name = skillNewCategoryName.trim();
    if (name) { resume.skills[idx].category = name; resume = resume; }
    skillEditingNewCategory = false;
    skillNewCategoryName = '';
  }

  // ── Experience helpers ──────────────────────────────────────────────────────

  function selectedExperience(): ExperienceEntry | null {
    return resume.experience[selectedExperienceIndex] ?? null;
  }

  function profileIncludesExperience(expId: string): boolean {
    const profile = selectedProfile();
    if (!profile) return false;
    return resume.experience.find(e => e.id === expId)?.profile_ids.includes(profile.id) ?? false;
  }

  function toggleExperienceForProfile(expId: string, checked: boolean) {
    const profile = selectedProfile();
    if (!profile) return;
    const exp = resume.experience.find(e => e.id === expId);
    if (!exp) return;
    if (checked) {
      if (!exp.profile_ids.includes(profile.id)) exp.profile_ids = [...exp.profile_ids, profile.id];
    } else {
      exp.profile_ids = exp.profile_ids.filter(id => id !== profile.id);
      if (exp.overrides) delete exp.overrides[profile.id];
    }
    resume = resume;
  }

  function selectedExperienceOverride(): ExperienceOverride | null {
    const profile = selectedProfile();
    const role = selectedExperience();
    if (!profile || !role) return null;
    return role.overrides?.[profile.id] ?? null;
  }

  function ensureExpOverride(role: ExperienceEntry, profileId: string): ExperienceOverride {
    role.overrides ??= {};
    role.overrides[profileId] ??= {};
    return role.overrides[profileId];
  }

  function setExperienceOverrideField(field: keyof ExperienceOverride, value: string) {
    const profile = selectedProfile();
    const role = selectedExperience();
    if (!profile || !role) return;
    const trimmed = value.trim();
    if (!trimmed) {
      if (role.overrides?.[profile.id]) {
        delete role.overrides[profile.id][field];
        if (Object.keys(role.overrides[profile.id]).length === 0) delete role.overrides[profile.id];
      }
      return;
    }
    ensureExpOverride(role, profile.id)[field] = trimmed;
    resume = resume;
  }

  function setExperienceOverrideBullets(value: string) {
    const profile = selectedProfile();
    const role = selectedExperience();
    if (!profile || !role) return;
    const bullets = normalizeLines(value);
    if (bullets.length === 0) {
      if (role.overrides?.[profile.id]) {
        delete role.overrides[profile.id].bullets;
        if (Object.keys(role.overrides[profile.id]).length === 0) delete role.overrides[profile.id];
      }
      return;
    }
    ensureExpOverride(role, profile.id).bullets = bullets;
    resume = resume;
  }

  function clearExperienceOverride() {
    const profile = selectedProfile();
    const role = selectedExperience();
    if (!profile || !role?.overrides) return;
    delete role.overrides[profile.id];
    resume = resume;
  }

  function addExperience() {
    const id = uniqueId(`experience-${resume.experience.length + 1}`, resume.experience.map(e => e.id));
    resume.experience = [...resume.experience, blankExperience(id)];
    selectedExperienceIndex = resume.experience.length - 1;
  }

  function removeExperience(index: number) {
    resume.experience = resume.experience.filter((_, i) => i !== index);
    selectedExperienceIndex = Math.max(0, Math.min(selectedExperienceIndex, resume.experience.length - 1));
  }

  function moveExperience(index: number, delta: number) {
    const target = index + delta;
    if (target < 0 || target >= resume.experience.length) return;
    const next = [...resume.experience];
    const [item] = next.splice(index, 1);
    next.splice(target, 0, item);
    resume.experience = next;
    selectedExperienceIndex = target;
  }

  function normalizeExperienceId(index: number) {
    const role = resume.experience[index];
    if (!role) return;
    role.id = uniqueId(
      slugify(`${role.employer || 'employer'}-${role.job_title || 'role'}`),
      resume.experience.filter((_, i) => i !== index).map(e => e.id)
    );
    resume = resume;
  }

  function updateBaseBullets(value: string) {
    const role = selectedExperience();
    if (!role) return;
    role.bullets = normalizeLines(value);
  }

  // ── Education helpers ───────────────────────────────────────────────────────

  function selectedEducation(): EducationEntry | null {
    return resume.education[selectedEducationIndex] ?? null;
  }

  function profileIncludesEducation(eduId: string): boolean {
    const profile = selectedProfile();
    if (!profile) return false;
    return resume.education.find(e => e.id === eduId)?.profile_ids.includes(profile.id) ?? false;
  }

  function toggleEducationForProfile(eduId: string, checked: boolean) {
    const profile = selectedProfile();
    if (!profile) return;
    const edu = resume.education.find(e => e.id === eduId);
    if (!edu) return;
    if (checked) {
      if (!edu.profile_ids.includes(profile.id)) edu.profile_ids = [...edu.profile_ids, profile.id];
    } else {
      edu.profile_ids = edu.profile_ids.filter(id => id !== profile.id);
    }
    resume = resume;
  }

  function addEducation() {
    const id = uniqueId(`edu-${resume.education.length + 1}`, resume.education.map(e => e.id));
    resume.education = [...resume.education, blankEducation(id)];
    selectedEducationIndex = resume.education.length - 1;
  }

  function removeEducation(index: number) {
    resume.education = resume.education.filter((_, i) => i !== index);
    selectedEducationIndex = Math.max(0, Math.min(selectedEducationIndex, resume.education.length - 1));
  }

  // ── Language helpers ────────────────────────────────────────────────────────

  function profileIncludesLanguage(langId: string): boolean {
    const profile = selectedProfile();
    if (!profile) return false;
    return resume.languages.find(l => l.id === langId)?.profile_ids.includes(profile.id) ?? false;
  }

  function toggleLanguageForProfile(langId: string, checked: boolean) {
    const profile = selectedProfile();
    if (!profile) return;
    const lang = resume.languages.find(l => l.id === langId);
    if (!lang) return;
    if (checked) {
      if (!lang.profile_ids.includes(profile.id)) lang.profile_ids = [...lang.profile_ids, profile.id];
    } else {
      lang.profile_ids = lang.profile_ids.filter(id => id !== profile.id);
    }
    resume = resume;
  }

  function addLanguage() {
    const id = uniqueId(`lang-${resume.languages.length + 1}`, resume.languages.map(l => l.id));
    resume.languages = [...resume.languages, { ...blankLanguage(), id }];
  }

  function removeLanguage(index: number) {
    resume.languages = resume.languages.filter((_, i) => i !== index);
  }

  // ── Bootstrap ───────────────────────────────────────────────────────────────

  async function bootstrap() {
    try {
      const config = await invoke<{ data_folder: string | null }>('load_app_config');
      if (config.data_folder) {
        dataFolder = config.data_folder;
        await loadData();
      }
    } catch {
      // No config yet — show the choose-folder screen
    }
  }

  onMount(bootstrap);
</script>

<svelte:head>
  <title>Resume Editor</title>
</svelte:head>

<svelte:window on:keydown={e => {
  if (e.metaKey && !e.shiftKey && e.key === 'z') { e.preventDefault(); undo(); }
  if (e.metaKey && e.shiftKey && e.key === 'z') { e.preventDefault(); redo(); }
  if (e.metaKey && e.key === 's') { e.preventDefault(); saveResume(); }
}} />

{#if !dataFolder}
  <!-- ── Setup screen ────────────────────────────────────────────────────── -->
  <div class="setup-shell">
    <div class="setup-card">
      <p class="eyebrow">Resume Editor</p>
      <h1>Choose your data folder</h1>
      <p class="setup-lede">
        Select the folder where your <code>resume.json</code> lives — or an empty folder to start
        from scratch. Everything (resume data, applications, generated files) stays in this folder.
      </p>
      <button class="primary" on:click={chooseFolder}>Choose Folder…</button>
    </div>
  </div>
{:else}
  <!-- ── Floating command bar ─────────────────────────────────────────── -->
  <div class="command-bar">
    <button class="cmd-btn" on:click={undo} disabled={history.length === 0} title="Undo (⌘Z)">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M9 14 4 9l5-5"/><path d="M4 9h10.5a5.5 5.5 0 0 1 0 11H11"/>
      </svg>
      Undo
    </button>
    <button class="cmd-btn" on:click={redo} disabled={future.length === 0} title="Redo (⌘⇧Z)">
      Redo
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M15 14l5-5-5-5"/><path d="M20 9H9.5a5.5 5.5 0 0 0 0 11H13"/>
      </svg>
    </button>
    <div class="cmd-sep"></div>
    <button class="cmd-btn cmd-save" class:cmd-save-active={hasUnsavedChanges} on:click={saveResume} disabled={busy} title="Save (⌘S)">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
        <polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/>
      </svg>
      Save{#if hasUnsavedChanges} ●{/if}
    </button>
  </div>

  <!-- ── Main editor ─────────────────────────────────────────────────────── -->
  <div class="shell">
    <aside class="sidebar">
      <div class="brand">
        <p class="eyebrow">Local Editor</p>
        <h1>Resume Control Room</h1>
      </div>

      <div class="folder-row">
        <div class="folder-path" title={dataFolder}>{dataFolder}</div>
        <button class="ghost small" on:click={chooseFolder}>Change</button>
      </div>

      <div class="actions">
        <button class="secondary" on:click={loadData} disabled={busy}>Reload</button>
        <button class="secondary" on:click={saveResume} disabled={busy}>Save</button>
        <button class="primary" on:click={buildResume} disabled={busy}>Build Resume</button>
      </div>

      <div class="status">
        {#if busy}<p class="saving">Working…</p>
        {:else if notice}<p class="notice">{notice}</p>{/if}
        {#if appError}<p class="error">{appError}</p>{/if}
      </div>

      <div class="build-panel">
        <h2>Generated Files</h2>
        {#if generatedFiles.length}
          <div class="generated-files">
            {#each generatedFiles as file}
              <button class="file-link" on:click={() => openGeneratedFile(file)}>
                <strong>{file.label}</strong>
                <small>{file.relative_path}</small>
              </button>
            {/each}
          </div>
        {:else}
          <p class="empty-state">Run a build to populate generated outputs.</p>
        {/if}
      </div>

      <div class="build-panel grow">
        <h2>Build Log</h2>
        <pre>{buildLog || 'Build output will appear here.'}</pre>
      </div>
    </aside>

    <main class="editor">

      <!-- Profiles ─────────────────────────────────────────────────────────── -->
      <section class="card">
        <div class="section-head split">
          <div>
            <p class="eyebrow">Profiles</p>
            <h2>Targeted Resume Variants</h2>
          </div>
          <div class="actions compact">
            {#if addingProfile}
              <input
                class="inline-input"
                bind:value={newProfileLabel}
                placeholder="Profile label…"
                on:keydown={e => { if (e.key === 'Enter') addProfile(); if (e.key === 'Escape') { addingProfile = false; newProfileLabel = ''; } }}
              />
              <button class="secondary" on:click={addProfile}>Create</button>
              <button class="ghost" on:click={() => { addingProfile = false; newProfileLabel = ''; }}>Cancel</button>
            {:else}
              <button class="secondary" on:click={() => addingProfile = true}>Add Profile</button>
              <button class="ghost" on:click={removeSelectedProfile} disabled={resume.profiles.length === 1}>Delete</button>
            {/if}
          </div>
        </div>

        <div class="profile-pills">
          {#each resume.profiles as profile, index}
            <button
              class="profile-pill"
              class:selected={index === selectedProfileIndex}
              class:active={profile.id === resume.active_profile_id}
              on:click={() => (selectedProfileIndex = index)}
            >
              <span>{profile.label || 'Untitled Profile'}</span>
              <small>{profile.id === resume.active_profile_id ? 'Active build profile' : profile.id}</small>
            </button>
          {/each}
        </div>

        {#if selectedProfile()}
          <div class="grid two">
            <label class="stack">
              <span>Label</span>
              <input bind:value={resume.profiles[selectedProfileIndex].label} />
            </label>
            <label class="stack">
              <span>Id</span>
              <div class="inline-row">
                <input
                  value={resume.profiles[selectedProfileIndex].id}
                  on:change={e => updateSelectedProfileId((e.currentTarget as HTMLInputElement).value)}
                />
                <button class="ghost" on:click={() => setActiveProfile(selectedProfileIndex)}>
                  {resume.profiles[selectedProfileIndex].id === resume.active_profile_id ? 'Active ✓' : 'Set Active'}
                </button>
              </div>
            </label>
            <label class="stack full">
              <span>Summary</span>
              <textarea bind:value={resume.profiles[selectedProfileIndex].summary} rows="5"></textarea>
            </label>
          </div>
        {/if}
      </section>

      <!-- Basics ───────────────────────────────────────────────────────────── -->
      <section class="card">
        <div class="section-head">
          <p class="eyebrow">Identity</p>
          <h2>Contact Information</h2>
        </div>
        <div class="grid two">
          <label class="stack"><span>Name</span><input bind:value={resume.basics.name} /></label>
          <label class="stack"><span>Email</span><input bind:value={resume.basics.email} /></label>
          <label class="stack full"><span>Address</span><input bind:value={resume.basics.address} /></label>
          <label class="stack"><span>Phone</span><input bind:value={resume.basics.phone} /></label>
          <label class="stack"><span>Website</span><input bind:value={resume.basics.website} /></label>
          <label class="stack"><span>LinkedIn</span><input bind:value={resume.basics.linkedin} /></label>
          <label class="stack"><span>GitHub</span><input bind:value={resume.basics.github} /></label>
        </div>
      </section>

      <!-- Writing style ────────────────────────────────────────────────────── -->
      <section class="card">
        <div class="section-head">
          <p class="eyebrow">AI</p>
          <h2>Writing Style Context</h2>
        </div>
        <label class="stack">
          <span>Your writing voice</span>
          <textarea bind:value={resume.writing_style.context} rows="5"></textarea>
          <p class="hint">
            Describe how you naturally write professionally — sentence structure, level of formality,
            what you emphasise. This is injected into every AI generation (cover letters, translations)
            to keep output in your voice.
          </p>
        </label>
      </section>

      <!-- Experience ───────────────────────────────────────────────────────── -->
      <section class="card experience-card">
        <div class="section-head split">
          <div>
            <p class="eyebrow">Experience Inventory</p>
            <h2>Role History</h2>
          </div>
          <button class="secondary" on:click={addExperience}>Add Role</button>
        </div>

        <div class="experience-layout">
          <div class="experience-list">
            {#each resume.experience as role, index}
              <button
                class="experience-pill"
                class:selected={index === selectedExperienceIndex}
                class:included={profileIncludesExperience(role.id)}
                on:click={() => (selectedExperienceIndex = index)}
              >
                <span>{role.employer || 'New Employer'}</span>
                <small>{role.job_title || 'Untitled role'}</small>
              </button>
            {/each}
          </div>

          {#if selectedExperience()}
            <div class="experience-form">
              <div class="toolbar">
                <button class="ghost" on:click={() => moveExperience(selectedExperienceIndex, -1)}>↑ Up</button>
                <button class="ghost" on:click={() => moveExperience(selectedExperienceIndex, 1)}>↓ Down</button>
                <button class="danger" on:click={() => removeExperience(selectedExperienceIndex)}>Delete</button>
              </div>

              <div class="grid two">
                <label class="stack"><span>Employer</span><input bind:value={resume.experience[selectedExperienceIndex].employer} /></label>
                <label class="stack"><span>Job Title</span><input bind:value={resume.experience[selectedExperienceIndex].job_title} /></label>
                <label class="stack"><span>Location</span><input bind:value={resume.experience[selectedExperienceIndex].location} /></label>
                <label class="stack"><span>Dates</span><input bind:value={resume.experience[selectedExperienceIndex].dates} /></label>
                <label class="stack"><span>Additional Role</span><input bind:value={resume.experience[selectedExperienceIndex].additional_role} /></label>
                <label class="stack">
                  <span>Id</span>
                  <div class="inline-row">
                    <input bind:value={resume.experience[selectedExperienceIndex].id} />
                    <button class="ghost" on:click={() => normalizeExperienceId(selectedExperienceIndex)}>Normalize</button>
                  </div>
                </label>
                <label class="stack full">
                  <span>Base Bullets</span>
                  <textarea
                    rows="7"
                    value={resume.experience[selectedExperienceIndex].bullets.join('\n')}
                    on:input={e => updateBaseBullets((e.currentTarget as HTMLTextAreaElement).value)}
                  ></textarea>
                  <p class="hint">Default bullets for all profiles.</p>
                </label>
              </div>

              {#if selectedProfile()}
                <div class="profile-override">
                  <div class="section-head split compact-head">
                    <div>
                      <p class="eyebrow">Profile Override</p>
                      <h3>{selectedProfile()?.label}</h3>
                    </div>
                    <label class="toggle">
                      <input
                        type="checkbox"
                        checked={profileIncludesExperience(resume.experience[selectedExperienceIndex].id)}
                        on:change={e => toggleExperienceForProfile(
                          resume.experience[selectedExperienceIndex].id,
                          (e.currentTarget as HTMLInputElement).checked
                        )}
                      />
                      <span>Include in profile</span>
                    </label>
                  </div>

                  {#if profileIncludesExperience(resume.experience[selectedExperienceIndex].id)}
                    <div class="grid two">
                      <label class="stack">
                        <span>Job Title Override</span>
                        <input
                          value={selectedExperienceOverride()?.job_title ?? ''}
                          on:input={e => setExperienceOverrideField('job_title', (e.currentTarget as HTMLInputElement).value)}
                        />
                      </label>
                      <label class="stack">
                        <span>Additional Role Override</span>
                        <input
                          value={selectedExperienceOverride()?.additional_role ?? ''}
                          on:input={e => setExperienceOverrideField('additional_role', (e.currentTarget as HTMLInputElement).value)}
                        />
                      </label>
                      <label class="stack full">
                        <span>Bullets Override</span>
                        <textarea
                          rows="7"
                          value={selectedExperienceOverride()?.bullets?.join('\n') ?? ''}
                          on:input={e => setExperienceOverrideBullets((e.currentTarget as HTMLTextAreaElement).value)}
                        ></textarea>
                        <p class="hint">Leave blank to use base bullets.</p>
                      </label>
                    </div>
                    <button class="ghost" on:click={clearExperienceOverride}>Clear Override</button>
                  {:else}
                    <p class="hint">This role is excluded from the selected profile.</p>
                  {/if}
                </div>
              {/if}
            </div>
          {/if}
        </div>
      </section>

      <!-- Education ───────────────────────────────────────────────────────── -->
      <section class="card">
        <div class="section-head split">
          <div>
            <p class="eyebrow">Education</p>
            <h2>Academic Background</h2>
          </div>
          <button class="secondary" on:click={addEducation}>Add Degree</button>
        </div>

        <div class="experience-layout">
          <div class="experience-list">
            {#each resume.education as edu, index}
              <button
                class="experience-pill"
                class:selected={index === selectedEducationIndex}
                class:included={profileIncludesEducation(edu.id)}
                on:click={() => (selectedEducationIndex = index)}
              >
                <span>{edu.school || 'New School'}</span>
                <small>{edu.degree || 'Untitled degree'}</small>
              </button>
            {/each}
          </div>

          {#if selectedEducation()}
            <div class="grid two">
              <div class="skill-card-head full">
                <label class="toggle">
                  <input
                    type="checkbox"
                    checked={profileIncludesEducation(resume.education[selectedEducationIndex].id)}
                    on:change={e => toggleEducationForProfile(
                      resume.education[selectedEducationIndex].id,
                      (e.currentTarget as HTMLInputElement).checked
                    )}
                  />
                  <span>Include in selected profile</span>
                </label>
                <button class="danger" on:click={() => removeEducation(selectedEducationIndex)}>Delete</button>
              </div>
              <label class="stack"><span>School</span><input bind:value={resume.education[selectedEducationIndex].school} /></label>
              <label class="stack"><span>Degree</span><input bind:value={resume.education[selectedEducationIndex].degree} /></label>
              <label class="stack"><span>Field of Study</span><input bind:value={resume.education[selectedEducationIndex].field_of_study} /></label>
              <label class="stack"><span>Graduation Date</span><input bind:value={resume.education[selectedEducationIndex].graduation_date} /></label>
              <label class="stack full"><span>Notes</span><input bind:value={resume.education[selectedEducationIndex].notes} /></label>
            </div>
          {/if}
        </div>
      </section>

      <!-- Languages ───────────────────────────────────────────────────────── -->
      <section class="card">
        <div class="section-head split">
          <div>
            <p class="eyebrow">Languages</p>
            <h2>Spoken Languages</h2>
          </div>
          <button class="secondary" on:click={addLanguage}>Add Language</button>
        </div>
        <div class="skill-list">
          {#each resume.languages as lang, index}
            <div class="skill-card">
              <div class="skill-card-head">
                <label class="toggle">
                  <input
                    type="checkbox"
                    checked={profileIncludesLanguage(lang.id)}
                    on:change={e => toggleLanguageForProfile(lang.id, (e.currentTarget as HTMLInputElement).checked)}
                  />
                  <span>In selected profile</span>
                </label>
                <button class="ghost" on:click={() => removeLanguage(index)}>Remove</button>
              </div>
              <label class="stack"><span>Language</span><input bind:value={resume.languages[index].name} /></label>
              <label class="stack"><span>Proficiency</span><input bind:value={resume.languages[index].proficiency} placeholder="Native, Fluent, Conversational…" /></label>
            </div>
          {/each}
        </div>
      </section>

      <!-- Skills ───────────────────────────────────────────────────────────── -->
      <section class="card">
        <div class="section-head split">
          <div>
            <p class="eyebrow">Skills</p>
            <h2>Skills</h2>
          </div>
          <div class="actions compact">
            {#if addingSkill}
              <input
                class="inline-input"
                bind:value={newSkillLabel}
                placeholder="Skill label…"
                on:keydown={e => { if (e.key === 'Enter') addSkill(); if (e.key === 'Escape') { addingSkill = false; newSkillLabel = ''; } }}
              />
              <div class="custom-select compact">
                <button class="csel-trigger" on:click={() => openDropdown = openDropdown === 'addSkill' ? null : 'addSkill'}>
                  <span>{newSkillCategory === '__new__' ? '+ New category…' : (newSkillCategory || 'Category…')}</span>
                  <span class="csel-arrow">▾</span>
                </button>
                {#if openDropdown === 'addSkill'}
                  <div class="csel-backdrop" on:click={() => openDropdown = null}></div>
                  <div class="csel-options">
                    {#each categoriesInOrder() as cat}
                      <button class="csel-option" class:active={newSkillCategory === cat}
                        on:click={() => { newSkillCategory = cat; openDropdown = null; }}>
                        {cat}
                      </button>
                    {/each}
                    <button class="csel-option new-cat-opt"
                      on:click={() => { newSkillCategory = '__new__'; openDropdown = null; }}>
                      + New category…
                    </button>
                  </div>
                {/if}
              </div>
              {#if newSkillCategory === '__new__'}
                <input
                  class="inline-input"
                  bind:value={newSkillCategoryName}
                  placeholder="Category name…"
                  on:keydown={e => { if (e.key === 'Enter') addSkill(); }}
                />
              {/if}
              <button class="secondary" on:click={addSkill}>Add</button>
              <button class="ghost" on:click={() => { addingSkill = false; newSkillLabel = ''; }}>Cancel</button>
            {:else}
              <button class="secondary" on:click={() => { addingSkill = true; newSkillCategory = categoriesInOrder()[0] ?? '__new__'; }}>Add Skill</button>
            {/if}
          </div>
        </div>

        {#if resume.skills.length === 0}
          <p class="empty-state">No skills yet — click Add Skill to get started.</p>
        {:else}
          {#each categoriesInOrder() as category, catIdx}
            <div class="category-group">
              <div class="category-header">
                {#if editingCategoryName === category}
                  <input
                    class="inline-input category-rename-input"
                    bind:value={renamedCategoryValue}
                    on:keydown={e => { if (e.key === 'Enter') renameCategory(category); if (e.key === 'Escape') editingCategoryName = null; }}
                  />
                  <button class="secondary" on:click={() => renameCategory(category)}>Save</button>
                  <button class="ghost" on:click={() => editingCategoryName = null}>Cancel</button>
                {:else}
                  <span class="category-label">{category}</span>
                  <button class="ghost small" on:click={() => { editingCategoryName = category; renamedCategoryValue = category; }}>Rename</button>
                  <button class="ghost small" on:click={() => moveCategoryUp(category)} disabled={catIdx === 0}>↑</button>
                  <button class="ghost small" on:click={() => moveCategoryDown(category)} disabled={catIdx === categoriesInOrder().length - 1}>↓</button>
                {/if}
              </div>

              <div class="skill-cloud">
                {#each resume.skills.filter(s => s.category === category) as skill}
                  <div
                    role="button"
                    tabindex="0"
                    class="skill-chip"
                    class:selected={selectedSkillId === skill.id}
                    class:in-profile={profileIncludesSkill(skill.id)}
                    on:click={() => { selectedSkillId = selectedSkillId === skill.id ? null : skill.id; skillEditingNewCategory = false; }}
                    on:keydown={e => e.key === 'Enter' && (selectedSkillId = selectedSkillId === skill.id ? null : skill.id)}
                  >
                    {skill.label || 'Unnamed'}
                  </div>
                {/each}
              </div>
            </div>
          {/each}
        {/if}

        {#if selectedSkillId !== null}
          {@const idx = resume.skills.findIndex(s => s.id === selectedSkillId)}
          {#if idx >= 0}
            <div class="skill-edit-panel">
              <div class="skill-card-head">
                <div class="actions compact">
                  <button class="ghost small" on:click={() => moveSkillLeft(selectedSkillId!)}>← Left</button>
                  <button class="ghost small" on:click={() => moveSkillRight(selectedSkillId!)}>Right →</button>
                </div>
                <label class="toggle">
                  <input
                    type="checkbox"
                    checked={profileIncludesSkill(resume.skills[idx].id)}
                    on:change={e => toggleSkillForProfile(resume.skills[idx].id, (e.currentTarget as HTMLInputElement).checked)}
                  />
                  <span>In profile</span>
                </label>
                <button class="danger" on:click={() => { removeSkill(idx); selectedSkillId = null; }}>Delete</button>
              </div>
              <div class="grid two">
                <label class="stack"><span>Label</span><input bind:value={resume.skills[idx].label} /></label>
                <label class="stack">
                  <span>Category</span>
                  {#if skillEditingNewCategory}
                    <div class="inline-row">
                      <input bind:value={skillNewCategoryName} placeholder="New category…" on:keydown={e => e.key === 'Enter' && applySkillNewCategory(idx)} />
                      <button class="ghost" on:click={() => applySkillNewCategory(idx)}>OK</button>
                      <button class="ghost" on:click={() => skillEditingNewCategory = false}>✕</button>
                    </div>
                  {:else}
                    <div class="custom-select">
                      <button class="csel-trigger" on:click={() => openDropdown = openDropdown === 'skillEdit' ? null : 'skillEdit'}>
                        <span>{resume.skills[idx].category || 'Select…'}</span>
                        <span class="csel-arrow">▾</span>
                      </button>
                      {#if openDropdown === 'skillEdit'}
                        <div class="csel-backdrop" on:click={() => openDropdown = null}></div>
                        <div class="csel-options">
                          {#each categoriesInOrder() as cat}
                            <button class="csel-option" class:active={resume.skills[idx].category === cat}
                              on:click={() => { resume.skills[idx].category = cat; resume = resume; openDropdown = null; }}>
                              {cat}
                            </button>
                          {/each}
                          <button class="csel-option new-cat-opt"
                            on:click={() => { skillNewCategoryName = ''; skillEditingNewCategory = true; openDropdown = null; }}>
                            + New category…
                          </button>
                        </div>
                      {/if}
                    </div>
                  {/if}
                </label>
                <label class="stack">
                  <span>Years of experience</span>
                  <input type="number" bind:value={resume.skills[idx].years_of_experience} min="0" />
                </label>
                <label class="stack">
                  <span>Id</span>
                  <div class="inline-row">
                    <input bind:value={resume.skills[idx].id} />
                    <button class="ghost" on:click={() => normalizeSkillId(idx)}>Normalize</button>
                  </div>
                </label>
              </div>
            </div>
          {/if}
        {/if}
      </section>

    </main>
  </div>
{/if}

<style>
  :global(body) {
    margin: 0;
    font-family: 'Iowan Old Style', 'Palatino Linotype', 'Book Antiqua', Palatino, Georgia, serif;
    background:
      radial-gradient(circle at top left, rgba(226, 200, 168, 0.35), transparent 30%),
      linear-gradient(180deg, #f5efe6 0%, #ece2d2 100%);
    color: #221a14;
    min-height: 100vh;
  }

  :global(button), :global(input), :global(textarea) { font: inherit; }

  /* Setup screen */
  .setup-shell {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    padding: 2rem;
  }

  .setup-card {
    max-width: 520px;
    background: rgba(255, 252, 247, 0.88);
    border: 1px solid rgba(111, 84, 53, 0.14);
    border-radius: 28px;
    padding: 3rem;
    box-shadow: 0 24px 80px rgba(90, 62, 35, 0.12);
    text-align: center;
    display: grid;
    gap: 1rem;
  }

  .setup-card h1 { margin: 0; font-family: 'Avenir Next', 'Helvetica Neue', sans-serif; font-size: 1.8rem; }
  .setup-lede { margin: 0; line-height: 1.55; color: #4a3526; }
  .setup-lede code { font-family: 'SF Mono', Menlo, monospace; background: rgba(90, 62, 35, 0.1); padding: 0.1em 0.35em; border-radius: 6px; }

  /* Main layout */
  .shell {
    display: grid;
    grid-template-columns: 340px 1fr;
    min-height: 100vh;
  }

  .sidebar {
    padding: 2rem;
    background: rgba(41, 29, 21, 0.92);
    color: #f8f1e5;
    display: flex;
    flex-direction: column;
    gap: 1.1rem;
    box-shadow: inset -1px 0 0 rgba(255, 255, 255, 0.08);
  }

  .editor {
    padding: 2rem;
    display: grid;
    gap: 1.25rem;
    align-content: start;
  }

  /* Folder row */
  .folder-row {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .folder-path {
    flex: 1;
    font-family: 'SF Mono', Menlo, monospace;
    font-size: 0.78rem;
    color: rgba(248, 241, 229, 0.65);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .small { padding: 0.45rem 0.75rem; font-size: 0.82rem; }

  .brand h1, .section-head h2, .build-panel h2, .compact-head h3 {
    margin: 0;
    font-family: 'Avenir Next', 'Helvetica Neue', sans-serif;
  }

  .eyebrow {
    margin: 0 0 0.25rem;
    font-family: 'Avenir Next', 'Helvetica Neue', sans-serif;
    text-transform: uppercase;
    letter-spacing: 0.16em;
    font-size: 0.72rem;
    color: #b9824b;
  }

  .stack { display: grid; gap: 0.35rem; }

  .stack span, .hint, .experience-pill small, .profile-pill small, .toggle span {
    font-family: 'Avenir Next', 'Helvetica Neue', sans-serif;
  }

  .stack span {
    font-size: 0.82rem;
    color: #6b5643;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  input, textarea, pre {
    width: 100%;
    border: 1px solid rgba(90, 62, 35, 0.16);
    border-radius: 14px;
    background: rgba(255, 251, 245, 0.92);
    padding: 0.8rem 0.9rem;
    box-sizing: border-box;
    color: inherit;
  }

  textarea { resize: vertical; min-height: 5rem; }

  .actions, .toolbar, .section-head.split, .inline-row {
    display: flex;
    gap: 0.65rem;
    align-items: center;
  }

  .inline-row { width: 100%; }
  .section-head.split { justify-content: space-between; }
  .compact { gap: 0.5rem; }

  button {
    border: 0;
    border-radius: 999px;
    padding: 0.75rem 1rem;
    cursor: pointer;
    transition: transform 140ms ease, opacity 140ms ease, background 140ms ease;
    font-family: 'Avenir Next', 'Helvetica Neue', sans-serif;
    font-weight: 600;
  }

  button:hover:enabled { transform: translateY(-1px); }
  button:disabled { cursor: not-allowed; opacity: 0.55; }

  .primary { background: linear-gradient(135deg, #bb6a2f 0%, #8f4120 100%); color: white; }
  .secondary, .ghost, .danger { background: rgba(255, 251, 245, 0.92); color: #291d15; }
  .danger { color: #8b2b1d; }

  .status { min-height: 1.5rem; }

  .notice, .error {
    margin: 0;
    padding: 0.7rem 0.85rem;
    border-radius: 12px;
    font-family: 'Avenir Next', 'Helvetica Neue', sans-serif;
  }

  .notice { background: rgba(90, 137, 94, 0.2); color: #dbeed2; }
  .error { background: rgba(150, 49, 35, 0.22); color: #ffd8cf; }
  .saving { background: rgba(90, 62, 35, 0.18); color: rgba(248, 241, 229, 0.65); font-style: italic; }

  .build-panel { display: grid; gap: 0.6rem; }
  .build-panel.grow { flex: 1; min-height: 0; grid-template-rows: auto 1fr; }

  pre {
    margin: 0;
    background: rgba(11, 8, 6, 0.72);
    color: #f2e7d8;
    overflow: auto;
    min-height: 180px;
    white-space: pre-wrap;
    word-break: break-word;
    font-family: 'SF Mono', Menlo, Consolas, monospace;
    font-size: 0.84rem;
  }

  .generated-files, .profile-pills { display: grid; gap: 0.6rem; }
  .profile-pills { grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); margin-bottom: 1rem; }

  .file-link, .profile-pill {
    display: grid;
    gap: 0.18rem;
    text-align: left;
    border-radius: 16px;
    background: rgba(255, 251, 245, 0.92);
    color: #291d15;
    padding: 0.8rem 0.95rem;
  }

  .profile-pill.active { box-shadow: inset 0 0 0 2px rgba(187, 106, 47, 0.42); }
  .profile-pill.selected { background: linear-gradient(135deg, #3e5e63 0%, #1b2f37 100%); color: white; }
  .profile-pill.selected small { color: rgba(255, 255, 255, 0.76); }

  .file-link strong, .file-link small, .empty-state, .profile-pill span {
    font-family: 'Avenir Next', 'Helvetica Neue', sans-serif;
  }

  .file-link small, .profile-pill small { color: #6b5643; overflow-wrap: anywhere; }

  .empty-state { margin: 0; color: rgba(248, 241, 229, 0.78); }

  .card {
    background: rgba(255, 252, 247, 0.75);
    border: 1px solid rgba(111, 84, 53, 0.12);
    border-radius: 28px;
    padding: 1.35rem;
    box-shadow: 0 16px 50px rgba(90, 62, 35, 0.08), inset 0 1px 0 rgba(255, 255, 255, 0.48);
    backdrop-filter: blur(8px);
  }

  .grid { display: grid; gap: 0.95rem; }
  .grid.two { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .full { grid-column: 1 / -1; }

  .skill-list {
    display: grid;
    gap: 0.85rem;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  }

  .skill-card {
    padding: 1rem;
    border-radius: 22px;
    background: rgba(248, 240, 229, 0.88);
    border: 1px solid rgba(90, 62, 35, 0.12);
    display: grid;
    gap: 0.75rem;
  }

  .inline-input {
    padding: 0.45rem 0.75rem;
    border-radius: 10px;
    border: 1px solid rgba(90, 62, 35, 0.28);
    background: rgba(255, 252, 247, 0.9);
    font-size: 0.9rem;
    width: 160px;
  }

  .category-group { display: grid; gap: 0.55rem; }

  .category-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .category-label {
    font-family: 'Avenir Next', 'Helvetica Neue', sans-serif;
    font-weight: 600;
    font-size: 0.82rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: #7a5a3a;
    flex: 1;
  }

  .category-rename-input { width: 180px; }

  /* ── Custom select ── */
  .custom-select { position: relative; width: 100%; }
  .custom-select.compact { width: auto; min-width: 150px; }

  .csel-trigger {
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.6rem;
    padding: 0.8rem 0.9rem;
    border: 1px solid rgba(90, 62, 35, 0.16);
    border-radius: 14px;
    background: rgba(255, 251, 245, 0.92);
    color: #221a14;
    text-align: left;
    cursor: pointer;
    transition: border-color 0.15s;
  }
  .compact .csel-trigger { padding: 0.45rem 0.75rem; border-radius: 10px; font-size: 0.9rem; }
  .csel-trigger:hover { border-color: rgba(90, 62, 35, 0.38); }

  .csel-arrow { font-size: 0.7rem; color: #9a7050; flex-shrink: 0; }

  .csel-backdrop {
    position: fixed;
    inset: 0;
    z-index: 99;
  }

  .csel-options {
    position: absolute;
    top: calc(100% + 5px);
    left: 0;
    right: 0;
    min-width: 100%;
    background: rgba(255, 252, 247, 0.98);
    border: 1px solid rgba(90, 62, 35, 0.18);
    border-radius: 14px;
    box-shadow: 0 8px 28px rgba(90, 62, 35, 0.14);
    backdrop-filter: blur(10px);
    z-index: 100;
    overflow: hidden;
    display: grid;
  }

  .csel-option {
    padding: 0.7rem 0.9rem;
    text-align: left;
    background: transparent;
    border: none;
    border-bottom: 1px solid rgba(90, 62, 35, 0.07);
    color: #221a14;
    cursor: pointer;
    transition: background 0.1s;
  }
  .csel-option:last-child { border-bottom: none; }
  .csel-option:hover { background: rgba(187, 106, 47, 0.1); }
  .csel-option.active { background: rgba(187, 106, 47, 0.15); font-weight: 600; }
  .csel-option.new-cat-opt { color: #b9824b; font-style: italic; }

  .skill-cloud {
    display: flex;
    flex-wrap: wrap;
    gap: 0.55rem;
    align-items: flex-start;
  }

  .skill-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.45rem 0.9rem;
    border-radius: 999px;
    border: 1.5px solid rgba(90, 62, 35, 0.2);
    background: rgba(248, 240, 229, 0.9);
    color: #3a2a1a;
    font-size: 0.88rem;
    cursor: grab;
    transition: all 0.15s;
    user-select: none;
  }
  .skill-chip:active { cursor: grabbing; }
  .skill-chip:hover { border-color: rgba(90, 62, 35, 0.5); }
  .skill-chip.in-profile { background: rgba(187, 106, 47, 0.14); border-color: rgba(187, 106, 47, 0.45); }
  .skill-chip.selected { background: linear-gradient(135deg, #3e5e63, #1b2f37); color: white; border-color: transparent; }
  .skill-chip.selected .chip-cat { color: rgba(255,255,255,0.65); }

  .chip-cat {
    font-size: 0.75rem;
    color: #9a7a5a;
    font-style: italic;
  }

  .skill-edit-panel {
    margin-top: 0.75rem;
    padding: 1rem;
    border-radius: 18px;
    background: rgba(248, 240, 229, 0.88);
    border: 1px solid rgba(90, 62, 35, 0.12);
    display: grid;
    gap: 0.85rem;
  }

  .skill-card-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.75rem;
  }

  .profile-override {
    padding: 1rem;
    border-radius: 22px;
    background: rgba(248, 240, 229, 0.88);
    border: 1px solid rgba(90, 62, 35, 0.12);
    display: grid;
    gap: 0.75rem;
  }

  .experience-layout {
    display: grid;
    grid-template-columns: 240px 1fr;
    gap: 1rem;
  }

  .experience-list { display: grid; gap: 0.75rem; align-content: start; }

  .experience-pill {
    display: grid;
    gap: 0.2rem;
    text-align: left;
    padding: 0.95rem 1rem;
    border-radius: 18px;
    background: rgba(241, 230, 214, 0.94);
    color: #2f2218;
    border: 1px solid transparent;
  }

  .experience-pill.included { box-shadow: inset 0 0 0 2px rgba(187, 106, 47, 0.24); }
  .experience-pill.selected { background: linear-gradient(135deg, #3e5e63 0%, #1b2f37 100%); color: white; border-color: rgba(255,255,255,0.28); }
  .experience-pill.selected small { color: rgba(255, 255, 255, 0.76); }
  .experience-pill span { font-family: 'Avenir Next', 'Helvetica Neue', sans-serif; font-weight: 700; }

  .experience-form, .profile-override { display: grid; gap: 1rem; }

  .compact-head { align-items: center; }

  .toggle { display: inline-flex; align-items: center; gap: 0.55rem; }
  .toggle input { width: auto; margin: 0; }

  .hint { margin: 0; color: #7a695a; font-size: 0.82rem; }

  /* ── Floating command bar ── */
  .command-bar {
    position: fixed;
    top: 1.1rem;
    left: 50%;
    transform: translateX(-50%);
    z-index: 1000;
    display: flex;
    align-items: center;
    gap: 0.25rem;
    background: rgba(28, 20, 13, 0.9);
    backdrop-filter: blur(18px) saturate(1.6);
    -webkit-backdrop-filter: blur(18px) saturate(1.6);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 999px;
    padding: 0.4rem 0.6rem;
    box-shadow: 0 8px 32px rgba(10, 6, 2, 0.35), 0 1px 0 rgba(255,255,255,0.06) inset;
  }

  .cmd-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.38rem 0.72rem;
    border-radius: 999px;
    background: transparent;
    color: rgba(248, 241, 229, 0.55);
    border: none;
    font-size: 0.82rem;
    font-weight: 600;
    font-family: 'Avenir Next', 'Helvetica Neue', sans-serif;
    cursor: pointer;
    transition: background 0.14s, color 0.14s;
  }

  .cmd-btn:hover:enabled {
    background: rgba(255, 255, 255, 0.1);
    color: rgba(248, 241, 229, 0.9);
    transform: none;
  }

  .cmd-btn:disabled {
    color: rgba(248, 241, 229, 0.22);
    cursor: not-allowed;
    opacity: 1;
  }

  .cmd-save { color: rgba(248, 241, 229, 0.55); }
  .cmd-save-active { color: #f0c98b !important; }
  .cmd-save-active:hover:enabled { color: #f5d9a0 !important; }

  .cmd-sep {
    width: 1px;
    height: 1.15rem;
    background: rgba(255, 255, 255, 0.14);
    margin: 0 0.1rem;
    flex-shrink: 0;
  }

  @media (max-width: 1100px) {
    .shell { grid-template-columns: 1fr; }
    .experience-layout, .grid.two { grid-template-columns: 1fr; }
    .sidebar { order: 2; }
  }
</style>
