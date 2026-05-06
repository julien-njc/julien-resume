<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type {
    BasicsData,
    BuildResult,
    ExperienceEntry,
    ExperienceOverride,
    GeneratedFile,
    ResumeData,
    ResumeProfile,
    SkillEntry
  } from '$lib/types';

  type LegacyResumeData = {
    name: string;
    address: string;
    email: string;
    phone: string;
    linkedin: string;
    github: string;
    website: string;
    summary: string;
    skills: Record<string, string[]>;
    experience: Array<{
      employer: string;
      job_title: string;
      location: string;
      dates: string;
      additional_role?: string;
      bullets: string[];
    }>;
    education: ResumeData['education'];
    languages: string[];
  };

  const blankBasics = (): BasicsData => ({
    name: '',
    address: '',
    email: '',
    phone: '',
    linkedin: '',
    github: '',
    website: ''
  });

  const blankProfile = (id = 'default-profile', label = 'Default Profile'): ResumeProfile => ({
    id,
    label,
    summary: '',
    skill_ids: [],
    experience_ids: [],
    experience_overrides: {}
  });

  const blankSkill = (id = ''): SkillEntry => ({
    id,
    category: '',
    label: ''
  });

  const blankExperience = (id = 'experience-1'): ExperienceEntry => ({
    id,
    employer: '',
    job_title: '',
    location: '',
    dates: '',
    additional_role: '',
    bullets: ['']
  });

  const blankResume = (): ResumeData => ({
    basics: blankBasics(),
    active_profile_id: 'default-profile',
    profiles: [blankProfile()],
    skills: [blankSkill('skill-1')],
    experience: [blankExperience()],
    education: {
      degree: '',
      field_of_study: '',
      school: '',
      graduation_date: ''
    },
    languages: []
  });

  let repoRoot = '';
  let resume: ResumeData = blankResume();
  let selectedProfileIndex = 0;
  let selectedExperienceIndex = 0;
  let languagesDraft = '';
  let notice = '';
  let error = '';
  let buildLog = '';
  let generatedFiles: GeneratedFile[] = [];
  let busy = false;

  function normalizeLines(value: string): string[] {
    return value
      .split('\n')
      .map((line) => line.trim())
      .filter(Boolean);
  }

  function slugify(value: string): string {
    const slug = value
      .toLowerCase()
      .trim()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-+|-+$/g, '');
    return slug || 'item';
  }

  function uniqueId(base: string, existing: string[]): string {
    let candidate = base;
    let counter = 2;
    while (existing.includes(candidate)) {
      candidate = `${base}-${counter}`;
      counter += 1;
    }
    return candidate;
  }

  function isModernResumeData(value: unknown): value is ResumeData {
    return Boolean(value && typeof value === 'object' && 'basics' in value && 'profiles' in value);
  }

  function upgradeLegacyResumeData(data: LegacyResumeData): ResumeData {
    const skills: SkillEntry[] = [];
    const skillIds: string[] = [];
    for (const [category, values] of Object.entries(data.skills ?? {})) {
      values.forEach((label, index) => {
        const id = uniqueId(`${slugify(category)}-${slugify(label)}`, skills.map((skill) => skill.id));
        skills.push({ id, category, label });
        skillIds.push(id);
      });
    }

    const experience: ExperienceEntry[] = [];
    for (const [index, role] of (data.experience ?? []).entries()) {
      const id = uniqueId(
        `${slugify(role.employer || 'employer')}-${slugify(role.job_title || `role-${index + 1}`)}`,
        experience.map((entry) => entry.id)
      );
      experience.push({
        id,
        employer: role.employer,
        job_title: role.job_title,
        location: role.location,
        dates: role.dates,
        additional_role: role.additional_role,
        bullets: role.bullets
      });
    }

    return {
      basics: {
        name: data.name,
        address: data.address,
        email: data.email,
        phone: data.phone,
        linkedin: data.linkedin,
        github: data.github,
        website: data.website
      },
      active_profile_id: 'default-profile',
      profiles: [
        {
          id: 'default-profile',
          label: 'Default Profile',
          summary: data.summary,
          skill_ids: skillIds,
          experience_ids: experience.map((role) => role.id),
          experience_overrides: {}
        }
      ],
      skills,
      experience,
      education: data.education,
      languages: data.languages ?? []
    };
  }

  function hydrateDrafts() {
    languagesDraft = resume.languages.join('\n');
    if (resume.profiles.length === 0) {
      resume.profiles = [blankProfile()];
    }
    if (!resume.profiles.some((profile) => profile.id === resume.active_profile_id)) {
      resume.active_profile_id = resume.profiles[0].id;
    }
    selectedProfileIndex = Math.max(0, Math.min(selectedProfileIndex, resume.profiles.length - 1));
    if (resume.experience.length === 0) {
      selectedExperienceIndex = 0;
    } else {
      selectedExperienceIndex = Math.max(
        0,
        Math.min(selectedExperienceIndex, resume.experience.length - 1)
      );
    }
  }

  function selectedProfile(): ResumeProfile | null {
    return resume.profiles[selectedProfileIndex] ?? null;
  }

  function selectedExperience(): ExperienceEntry | null {
    return resume.experience[selectedExperienceIndex] ?? null;
  }

  function selectedExperienceOverride(): ExperienceOverride | null {
    const profile = selectedProfile();
    const role = selectedExperience();
    if (!profile || !role) return null;
    return profile.experience_overrides?.[role.id] ?? null;
  }

  async function bootstrap() {
    try {
      repoRoot = await invoke<string>('default_resume_root');
      await loadResume();
      await refreshGeneratedFiles();
    } catch (cause) {
      error = String(cause);
      resume = blankResume();
      hydrateDrafts();
    }
  }

  async function refreshGeneratedFiles() {
    if (!repoRoot) return;
    try {
      generatedFiles = await invoke<GeneratedFile[]>('list_generated_files', { repoRoot });
    } catch {
      generatedFiles = [];
    }
  }

  async function loadResume() {
    busy = true;
    error = '';
    notice = '';
    try {
      const raw = await invoke<string>('load_resume', { repoRoot });
      const parsed = JSON.parse(raw) as ResumeData | LegacyResumeData;
      resume = isModernResumeData(parsed) ? parsed : upgradeLegacyResumeData(parsed);
      hydrateDrafts();
      await refreshGeneratedFiles();
      notice = 'Loaded resume.json';
    } catch (cause) {
      error = `Load failed: ${String(cause)}`;
    } finally {
      busy = false;
    }
  }

  async function saveResume() {
    busy = true;
    error = '';
    notice = '';
    try {
      resume.languages = normalizeLines(languagesDraft);
      const payload = JSON.stringify(resume, null, 2) + '\n';
      await invoke('save_resume', { repoRoot, contents: payload });
      notice = 'Saved resume.json';
    } catch (cause) {
      error = `Save failed: ${String(cause)}`;
    } finally {
      busy = false;
    }
  }

  async function buildResume() {
    await saveResume();
    if (error) return;
    busy = true;
    notice = '';
    error = '';
    buildLog = '';
    try {
      const result = await invoke<BuildResult>('build_resume', { repoRoot });
      buildLog = [result.stdout, result.stderr].filter(Boolean).join('\n');
      generatedFiles = result.generated_files;
      notice = result.status === 0 ? 'Build completed' : `Build exited with status ${result.status}`;
    } catch (cause) {
      error = `Build failed: ${String(cause)}`;
    } finally {
      busy = false;
    }
  }

  function addProfile() {
    const label = window.prompt('New profile label');
    if (!label) return;
    const id = uniqueId(slugify(label), resume.profiles.map((profile) => profile.id));
    resume.profiles = [...resume.profiles, blankProfile(id, label.trim())];
    selectedProfileIndex = resume.profiles.length - 1;
  }

  function removeSelectedProfile() {
    if (resume.profiles.length === 1) return;
    const removed = resume.profiles[selectedProfileIndex];
    resume.profiles = resume.profiles.filter((_, index) => index !== selectedProfileIndex);
    if (removed.id === resume.active_profile_id) {
      resume.active_profile_id = resume.profiles[0].id;
    }
    selectedProfileIndex = Math.max(0, selectedProfileIndex - 1);
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
    const next = uniqueId(
      slugify(value || profile.label),
      resume.profiles.filter((item) => item !== profile).map((item) => item.id)
    );
    if (resume.active_profile_id === profile.id) {
      resume.active_profile_id = next;
    }
    profile.id = next;
  }

  function addSkill() {
    const label = window.prompt('Skill label');
    if (!label) return;
    const category = window.prompt('Skill category', 'General')?.trim() || 'General';
    const id = uniqueId(slugify(`${category}-${label}`), resume.skills.map((skill) => skill.id));
    resume.skills = [...resume.skills, { id, category, label: label.trim() }];
  }

  function removeSkill(index: number) {
    const skill = resume.skills[index];
    if (!skill) return;
    resume.skills = resume.skills.filter((_, current) => current !== index);
    for (const profile of resume.profiles) {
      profile.skill_ids = profile.skill_ids.filter((id) => id !== skill.id);
    }
  }

  function normalizeSkillId(index: number) {
    const skill = resume.skills[index];
    if (!skill) return;
    skill.id = uniqueId(
      slugify(`${skill.category || 'skill'}-${skill.label || 'item'}`),
      resume.skills.filter((_, current) => current !== index).map((item) => item.id)
    );
  }

  function profileIncludesSkill(skillId: string): boolean {
    const profile = selectedProfile();
    return profile ? profile.skill_ids.includes(skillId) : false;
  }

  function toggleSkillForProfile(skillId: string, checked: boolean) {
    const profile = selectedProfile();
    if (!profile) return;
    profile.skill_ids = checked
      ? profile.skill_ids.includes(skillId)
        ? profile.skill_ids
        : [...profile.skill_ids, skillId]
      : profile.skill_ids.filter((id) => id !== skillId);
  }

  function addExperience() {
    const id = uniqueId(
      `experience-${resume.experience.length + 1}`,
      resume.experience.map((role) => role.id)
    );
    resume.experience = [...resume.experience, blankExperience(id)];
    selectedExperienceIndex = resume.experience.length - 1;
  }

  function removeExperience(index: number) {
    const role = resume.experience[index];
    if (!role) return;
    resume.experience = resume.experience.filter((_, current) => current !== index);
    for (const profile of resume.profiles) {
      profile.experience_ids = profile.experience_ids.filter((id) => id !== role.id);
      if (profile.experience_overrides) {
        delete profile.experience_overrides[role.id];
      }
    }
    if (resume.experience.length === 0) {
      selectedExperienceIndex = 0;
    } else {
      selectedExperienceIndex = Math.max(
        0,
        Math.min(selectedExperienceIndex, resume.experience.length - 1)
      );
    }
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
    const previousId = role.id;
    const nextId = uniqueId(
      slugify(`${role.employer || 'employer'}-${role.job_title || 'role'}`),
      resume.experience.filter((_, current) => current !== index).map((item) => item.id)
    );
    role.id = nextId;
    for (const profile of resume.profiles) {
      profile.experience_ids = profile.experience_ids.map((id) => (id === previousId ? nextId : id));
      if (profile.experience_overrides?.[previousId]) {
        profile.experience_overrides[nextId] = profile.experience_overrides[previousId];
        delete profile.experience_overrides[previousId];
      }
    }
  }

  function updateBaseBullets(value: string) {
    const role = selectedExperience();
    if (!role) return;
    role.bullets = normalizeLines(value);
  }

  function profileIncludesExperience(experienceId: string): boolean {
    const profile = selectedProfile();
    return profile ? profile.experience_ids.includes(experienceId) : false;
  }

  function toggleExperienceForProfile(experienceId: string, checked: boolean) {
    const profile = selectedProfile();
    if (!profile) return;
    profile.experience_ids = checked
      ? profile.experience_ids.includes(experienceId)
        ? profile.experience_ids
        : [...profile.experience_ids, experienceId]
      : profile.experience_ids.filter((id) => id !== experienceId);
    if (!checked && profile.experience_overrides) {
      delete profile.experience_overrides[experienceId];
    }
  }

  function ensureExperienceOverride(profile: ResumeProfile, experienceId: string): ExperienceOverride {
    profile.experience_overrides ??= {};
    profile.experience_overrides[experienceId] ??= {};
    return profile.experience_overrides[experienceId];
  }

  function pruneExperienceOverride(profile: ResumeProfile, experienceId: string) {
    const override = profile.experience_overrides?.[experienceId];
    if (!override) return;
    if (Object.keys(override).length === 0) {
      if (profile.experience_overrides) {
        delete profile.experience_overrides[experienceId];
      }
    }
  }

  function setExperienceOverrideField(field: keyof ExperienceOverride, value: string) {
    const profile = selectedProfile();
    const role = selectedExperience();
    if (!profile || !role) return;
    const trimmed = value.trim();
    if (!trimmed) {
      if (profile.experience_overrides?.[role.id]) {
        delete profile.experience_overrides[role.id][field];
        pruneExperienceOverride(profile, role.id);
      }
      return;
    }
    const override = ensureExperienceOverride(profile, role.id);
    override[field] = trimmed;
  }

  function setExperienceOverrideBullets(value: string) {
    const profile = selectedProfile();
    const role = selectedExperience();
    if (!profile || !role) return;
    const bullets = normalizeLines(value);
    if (bullets.length === 0) {
      if (profile.experience_overrides?.[role.id]) {
        delete profile.experience_overrides[role.id].bullets;
        pruneExperienceOverride(profile, role.id);
      }
      return;
    }
    const override = ensureExperienceOverride(profile, role.id);
    override.bullets = bullets;
  }

  function clearExperienceOverride() {
    const profile = selectedProfile();
    const role = selectedExperience();
    if (!profile || !role || !profile.experience_overrides) return;
    delete profile.experience_overrides[role.id];
  }

  async function openGeneratedFile(file: GeneratedFile) {
    try {
      await invoke('open_generated_file', { repoRoot, path: file.path });
    } catch (cause) {
      error = `Open failed: ${String(cause)}`;
    }
  }

  onMount(bootstrap);
</script>

<svelte:head>
  <title>Resume Editor</title>
  <meta
    name="description"
    content="Edit resume profiles and run the local Docker resume build from a Tauri desktop app."
  />
</svelte:head>

<div class="shell">
  <aside class="sidebar">
    <div class="brand">
      <p class="eyebrow">Local Editor</p>
      <h1>Resume Control Room</h1>
      <p class="lede">
        Maintain one master resume inventory, then build role-specific profiles by selecting the
        skills and experience each target needs.
      </p>
    </div>

    <label class="stack">
      <span>Resume Repo Root</span>
      <input bind:value={repoRoot} placeholder="/path/to/resume" />
    </label>

    <div class="actions">
      <button class="secondary" on:click={loadResume} disabled={busy}>Reload</button>
      <button class="secondary" on:click={saveResume} disabled={busy}>Save</button>
      <button class="primary" on:click={buildResume} disabled={busy}>Build Resume</button>
    </div>

    <div class="status">
      {#if notice}<p class="notice">{notice}</p>{/if}
      {#if error}<p class="error">{error}</p>{/if}
    </div>

    <div class="build-panel">
      <div class="panel-header">
        <h2>Generated Files</h2>
      </div>
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
        <p class="empty-state">Run a build to populate the generated outputs.</p>
      {/if}
    </div>

    <div class="build-panel">
      <div class="panel-header">
        <h2>Build Log</h2>
      </div>
      <pre>{buildLog || 'Build output will appear here.'}</pre>
    </div>
  </aside>

  <main class="editor">
    <section class="card">
      <div class="section-head split">
        <div>
          <p class="eyebrow">Profiles</p>
          <h2>Targeted Resume Variants</h2>
        </div>
        <div class="actions compact">
          <button class="secondary" on:click={addProfile}>Add Profile</button>
          <button class="ghost" on:click={removeSelectedProfile} disabled={resume.profiles.length === 1}>
            Delete Profile
          </button>
        </div>
      </div>

      <div class="profile-pills">
        {#each resume.profiles as profile, index}
          <button
            class:selected={index === selectedProfileIndex}
            class:active={profile.id === resume.active_profile_id}
            class="profile-pill"
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
            <span>Profile Label</span>
            <input bind:value={resume.profiles[selectedProfileIndex].label} />
          </label>
          <label class="stack">
            <span>Profile Id</span>
            <div class="inline-row">
              <input
                value={resume.profiles[selectedProfileIndex].id}
                on:change={(event) =>
                  updateSelectedProfileId((event.currentTarget as HTMLInputElement).value)}
              />
              <button class="ghost" on:click={() => setActiveProfile(selectedProfileIndex)}>
                {resume.profiles[selectedProfileIndex].id === resume.active_profile_id ? 'Active' : 'Set Active'}
              </button>
            </div>
          </label>
          <label class="stack full">
            <span>Profile Summary</span>
            <textarea bind:value={resume.profiles[selectedProfileIndex].summary} rows="6"></textarea>
            <p class="hint">This summary is profile-specific and becomes the rendered resume summary.</p>
          </label>
        </div>
      {/if}
    </section>

    <section class="card">
      <div class="section-head">
        <p class="eyebrow">Identity</p>
        <h2>Header</h2>
      </div>
      <div class="grid two">
        <label class="stack">
          <span>Name</span>
          <input bind:value={resume.basics.name} />
        </label>
        <label class="stack">
          <span>Email</span>
          <input bind:value={resume.basics.email} />
        </label>
        <label class="stack full">
          <span>Address</span>
          <input bind:value={resume.basics.address} />
        </label>
        <label class="stack">
          <span>Phone</span>
          <input bind:value={resume.basics.phone} />
        </label>
        <label class="stack">
          <span>Website</span>
          <input bind:value={resume.basics.website} />
        </label>
        <label class="stack">
          <span>LinkedIn</span>
          <input bind:value={resume.basics.linkedin} />
        </label>
        <label class="stack">
          <span>GitHub</span>
          <input bind:value={resume.basics.github} />
        </label>
      </div>
    </section>

    <section class="card">
      <div class="section-head split">
        <div>
          <p class="eyebrow">Skills Inventory</p>
          <h2>Master Skills</h2>
        </div>
        <button class="secondary" on:click={addSkill}>Add Skill</button>
      </div>

      <div class="skill-list">
        {#each resume.skills as skill, index}
          <div class="skill-card">
            <div class="skill-card-head">
              <label class="toggle">
                <input
                  type="checkbox"
                  checked={profileIncludesSkill(skill.id)}
                  on:change={(event) =>
                    toggleSkillForProfile(
                      skill.id,
                      (event.currentTarget as HTMLInputElement).checked
                    )}
                />
                <span>Include in selected profile</span>
              </label>
              <button class="ghost" on:click={() => removeSkill(index)}>Remove</button>
            </div>
            <label class="stack">
              <span>Label</span>
              <input bind:value={resume.skills[index].label} />
            </label>
            <label class="stack">
              <span>Category</span>
              <input bind:value={resume.skills[index].category} />
            </label>
            <label class="stack">
              <span>Id</span>
              <div class="inline-row">
                <input bind:value={resume.skills[index].id} />
                <button class="ghost" on:click={() => normalizeSkillId(index)}>Normalize</button>
              </div>
            </label>
          </div>
        {/each}
      </div>
    </section>

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
              class:selected={index === selectedExperienceIndex}
              class:included={profileIncludesExperience(role.id)}
              class="experience-pill"
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
              <button class="ghost" on:click={() => moveExperience(selectedExperienceIndex, -1)}>
                Move Up
              </button>
              <button class="ghost" on:click={() => moveExperience(selectedExperienceIndex, 1)}>
                Move Down
              </button>
              <button class="danger" on:click={() => removeExperience(selectedExperienceIndex)}>
                Delete
              </button>
            </div>

            <div class="grid two">
              <label class="stack">
                <span>Employer</span>
                <input bind:value={resume.experience[selectedExperienceIndex].employer} />
              </label>
              <label class="stack">
                <span>Job Title</span>
                <input bind:value={resume.experience[selectedExperienceIndex].job_title} />
              </label>
              <label class="stack">
                <span>Location</span>
                <input bind:value={resume.experience[selectedExperienceIndex].location} />
              </label>
              <label class="stack">
                <span>Dates</span>
                <input bind:value={resume.experience[selectedExperienceIndex].dates} />
              </label>
              <label class="stack">
                <span>Additional Role</span>
                <input bind:value={resume.experience[selectedExperienceIndex].additional_role} />
              </label>
              <label class="stack">
                <span>Experience Id</span>
                <div class="inline-row">
                  <input bind:value={resume.experience[selectedExperienceIndex].id} />
                  <button class="ghost" on:click={() => normalizeExperienceId(selectedExperienceIndex)}>
                    Normalize
                  </button>
                </div>
              </label>
              <label class="stack full">
                <span>Base Bullets</span>
                <textarea
                  rows="7"
                  value={resume.experience[selectedExperienceIndex].bullets.join('\n')}
                  on:input={(event) =>
                    updateBaseBullets((event.currentTarget as HTMLTextAreaElement).value)}
                ></textarea>
                <p class="hint">These bullets belong to the master experience inventory.</p>
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
                      on:change={(event) =>
                        toggleExperienceForProfile(
                          resume.experience[selectedExperienceIndex].id,
                          (event.currentTarget as HTMLInputElement).checked
                        )}
                    />
                    <span>Include in selected profile</span>
                  </label>
                </div>

                {#if profileIncludesExperience(resume.experience[selectedExperienceIndex].id)}
                  <div class="grid two">
                    <label class="stack">
                      <span>Profile Job Title Override</span>
                      <input
                        value={selectedExperienceOverride()?.job_title ?? ''}
                        on:input={(event) =>
                          setExperienceOverrideField(
                            'job_title',
                            (event.currentTarget as HTMLInputElement).value
                          )}
                      />
                    </label>
                    <label class="stack">
                      <span>Profile Additional Role Override</span>
                      <input
                        value={selectedExperienceOverride()?.additional_role ?? ''}
                        on:input={(event) =>
                          setExperienceOverrideField(
                            'additional_role',
                            (event.currentTarget as HTMLInputElement).value
                          )}
                      />
                    </label>
                    <label class="stack full">
                      <span>Profile Bullets Override</span>
                      <textarea
                        rows="7"
                        value={selectedExperienceOverride()?.bullets?.join('\n') ?? ''}
                        on:input={(event) =>
                          setExperienceOverrideBullets(
                            (event.currentTarget as HTMLTextAreaElement).value
                          )}
                      ></textarea>
                      <p class="hint">
                        Leave blank to use the base bullets. Use this area to target one role
                        without rewriting the master inventory.
                      </p>
                    </label>
                  </div>
                  <button class="ghost" on:click={clearExperienceOverride}>Clear Override</button>
                {:else}
                  <p class="hint">
                    This role is currently excluded from the selected profile and will not render in
                    that resume variant.
                  </p>
                {/if}
              </div>
            {/if}
          </div>
        {/if}
      </div>
    </section>

    <section class="card">
      <div class="section-head">
        <p class="eyebrow">Education</p>
        <h2>Academic Details</h2>
      </div>
      <div class="grid two">
        <label class="stack">
          <span>Degree</span>
          <input bind:value={resume.education.degree} />
        </label>
        <label class="stack">
          <span>Field of Study</span>
          <input bind:value={resume.education.field_of_study} />
        </label>
        <label class="stack">
          <span>School</span>
          <input bind:value={resume.education.school} />
        </label>
        <label class="stack">
          <span>Graduation Date</span>
          <input bind:value={resume.education.graduation_date} />
        </label>
      </div>
    </section>

    <section class="card">
      <div class="section-head">
        <p class="eyebrow">Languages</p>
        <h2>Spoken Languages</h2>
      </div>
      <label class="stack">
        <span>Languages</span>
        <textarea bind:value={languagesDraft} rows="4"></textarea>
        <p class="hint">One language per line.</p>
      </label>
    </section>
  </main>
</div>

<style>
  :global(body) {
    margin: 0;
    font-family:
      'Iowan Old Style', 'Palatino Linotype', 'Book Antiqua', Palatino, Georgia, serif;
    background:
      radial-gradient(circle at top left, rgba(226, 200, 168, 0.35), transparent 30%),
      linear-gradient(180deg, #f5efe6 0%, #ece2d2 100%);
    color: #221a14;
  }

  :global(button),
  :global(input),
  :global(textarea) {
    font: inherit;
  }

  .shell {
    display: grid;
    grid-template-columns: 360px 1fr;
    min-height: 100vh;
  }

  .sidebar {
    padding: 2rem;
    background: rgba(41, 29, 21, 0.92);
    color: #f8f1e5;
    display: flex;
    flex-direction: column;
    gap: 1.2rem;
    box-shadow: inset -1px 0 0 rgba(255, 255, 255, 0.08);
  }

  .editor {
    padding: 2rem;
    display: grid;
    gap: 1.25rem;
  }

  .brand h1,
  .section-head h2,
  .build-panel h2,
  .compact-head h3 {
    margin: 0;
    font-family: 'Avenir Next', 'Helvetica Neue', sans-serif;
  }

  .eyebrow {
    margin: 0 0 0.35rem;
    font-family: 'Avenir Next', 'Helvetica Neue', sans-serif;
    text-transform: uppercase;
    letter-spacing: 0.16em;
    font-size: 0.72rem;
    color: #b9824b;
  }

  .lede {
    margin: 0.6rem 0 0;
    line-height: 1.45;
    color: rgba(248, 241, 229, 0.78);
  }

  .stack {
    display: grid;
    gap: 0.35rem;
  }

  .stack span,
  .hint,
  .experience-pill small,
  .profile-pill small,
  .toggle span {
    font-family: 'Avenir Next', 'Helvetica Neue', sans-serif;
  }

  .stack span {
    font-size: 0.82rem;
    color: #6b5643;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  input,
  textarea,
  pre {
    width: 100%;
    border: 1px solid rgba(90, 62, 35, 0.16);
    border-radius: 14px;
    background: rgba(255, 251, 245, 0.92);
    padding: 0.8rem 0.9rem;
    box-sizing: border-box;
    color: inherit;
  }

  textarea {
    resize: vertical;
    min-height: 5rem;
  }

  .actions,
  .toolbar,
  .section-head.split,
  .inline-row {
    display: flex;
    gap: 0.65rem;
    align-items: center;
  }

  .inline-row {
    width: 100%;
  }

  .section-head.split {
    justify-content: space-between;
  }

  .compact {
    gap: 0.5rem;
  }

  button {
    border: 0;
    border-radius: 999px;
    padding: 0.75rem 1rem;
    cursor: pointer;
    transition:
      transform 140ms ease,
      opacity 140ms ease,
      background 140ms ease;
    font-family: 'Avenir Next', 'Helvetica Neue', sans-serif;
    font-weight: 600;
  }

  button:hover:enabled {
    transform: translateY(-1px);
  }

  button:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }

  .primary {
    background: linear-gradient(135deg, #bb6a2f 0%, #8f4120 100%);
    color: white;
  }

  .secondary,
  .ghost,
  .danger {
    background: rgba(255, 251, 245, 0.92);
    color: #291d15;
  }

  .danger {
    color: #8b2b1d;
  }

  .status {
    min-height: 1.5rem;
  }

  .notice,
  .error {
    margin: 0;
    padding: 0.7rem 0.85rem;
    border-radius: 12px;
    font-family: 'Avenir Next', 'Helvetica Neue', sans-serif;
  }

  .notice {
    background: rgba(90, 137, 94, 0.2);
    color: #dbeed2;
  }

  .error {
    background: rgba(150, 49, 35, 0.22);
    color: #ffd8cf;
  }

  .build-panel {
    display: grid;
    gap: 0.6rem;
    min-height: 0;
  }

  .build-panel:last-child {
    flex: 1;
  }

  pre {
    margin: 0;
    background: rgba(11, 8, 6, 0.72);
    color: #f2e7d8;
    overflow: auto;
    min-height: 260px;
    white-space: pre-wrap;
    word-break: break-word;
    font-family: 'SF Mono', Menlo, Consolas, monospace;
    font-size: 0.84rem;
  }

  .generated-files,
  .profile-pills {
    display: grid;
    gap: 0.6rem;
  }

  .profile-pills {
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    margin-bottom: 1rem;
  }

  .file-link,
  .profile-pill {
    display: grid;
    gap: 0.18rem;
    text-align: left;
    border-radius: 16px;
    background: rgba(255, 251, 245, 0.92);
    color: #291d15;
    padding: 0.8rem 0.95rem;
  }

  .profile-pill.active {
    box-shadow: inset 0 0 0 2px rgba(187, 106, 47, 0.42);
  }

  .profile-pill.selected {
    background: linear-gradient(135deg, #3e5e63 0%, #1b2f37 100%);
    color: white;
  }

  .file-link strong,
  .file-link small,
  .empty-state,
  .profile-pill span {
    font-family: 'Avenir Next', 'Helvetica Neue', sans-serif;
  }

  .file-link small,
  .profile-pill small {
    color: #6b5643;
    overflow-wrap: anywhere;
  }

  .profile-pill.selected small {
    color: rgba(255, 255, 255, 0.76);
  }

  .empty-state {
    margin: 0;
    color: rgba(248, 241, 229, 0.78);
  }

  .card {
    background: rgba(255, 252, 247, 0.75);
    border: 1px solid rgba(111, 84, 53, 0.12);
    border-radius: 28px;
    padding: 1.35rem;
    box-shadow:
      0 16px 50px rgba(90, 62, 35, 0.08),
      inset 0 1px 0 rgba(255, 255, 255, 0.48);
    backdrop-filter: blur(8px);
  }

  .grid {
    display: grid;
    gap: 0.95rem;
  }

  .grid.two {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .full {
    grid-column: 1 / -1;
  }

  .skill-list {
    display: grid;
    gap: 0.85rem;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  }

  .skill-card,
  .profile-override {
    padding: 1rem;
    border-radius: 22px;
    background: rgba(248, 240, 229, 0.88);
    border: 1px solid rgba(90, 62, 35, 0.12);
  }

  .skill-card-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 0.9rem;
  }

  .experience-layout {
    display: grid;
    grid-template-columns: 260px 1fr;
    gap: 1rem;
  }

  .experience-list {
    display: grid;
    gap: 0.75rem;
    align-content: start;
  }

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

  .experience-pill.included {
    box-shadow: inset 0 0 0 2px rgba(187, 106, 47, 0.24);
  }

  .experience-pill.selected {
    background: linear-gradient(135deg, #3e5e63 0%, #1b2f37 100%);
    color: white;
    border-color: rgba(255, 255, 255, 0.28);
  }

  .experience-pill.selected small {
    color: rgba(255, 255, 255, 0.76);
  }

  .experience-pill span {
    font-family: 'Avenir Next', 'Helvetica Neue', sans-serif;
    font-weight: 700;
  }

  .experience-form {
    display: grid;
    gap: 1rem;
  }

  .profile-override {
    display: grid;
    gap: 1rem;
  }

  .compact-head {
    align-items: center;
  }

  .toggle {
    display: inline-flex;
    align-items: center;
    gap: 0.55rem;
  }

  .toggle input {
    width: auto;
    margin: 0;
  }

  .hint {
    margin: 0;
    color: #7a695a;
    font-size: 0.82rem;
  }

  @media (max-width: 1100px) {
    .shell {
      grid-template-columns: 1fr;
    }

    .experience-layout,
    .grid.two {
      grid-template-columns: 1fr;
    }

    .sidebar {
      order: 2;
    }
  }
</style>
