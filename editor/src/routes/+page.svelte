<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { BuildResult, ExperienceEntry, GeneratedFile, ResumeData } from '$lib/types';

  const blankResume = (): ResumeData => ({
    name: '',
    address: '',
    email: '',
    phone: '',
    linkedin: '',
    github: '',
    website: '',
    summary: '',
    skills: {},
    experience: [],
    education: {
      degree: '',
      field_of_study: '',
      school: '',
      graduation_date: ''
    },
    languages: []
  });

  const blankExperience = (): ExperienceEntry => ({
    employer: '',
    job_title: '',
    location: '',
    dates: '',
    additional_role: '',
    bullets: ['']
  });

  let repoRoot = '';
  let resume: ResumeData = blankResume();
  let selectedExperience = 0;
  let languagesDraft = '';
  let skillDrafts: Record<string, string> = {};
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

  function hydrateDrafts() {
    languagesDraft = resume.languages.join('\n');
    skillDrafts = Object.fromEntries(
      Object.entries(resume.skills).map(([key, values]) => [key, values.join('\n')])
    );
    if (resume.experience.length === 0) {
      resume.experience = [blankExperience()];
      selectedExperience = 0;
    } else {
      selectedExperience = Math.min(selectedExperience, resume.experience.length - 1);
    }
  }

  function selectedRole(): ExperienceEntry | null {
    return resume.experience[selectedExperience] ?? null;
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
      resume = JSON.parse(raw) as ResumeData;
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
      for (const [key, value] of Object.entries(skillDrafts)) {
        resume.skills[key] = normalizeLines(value);
      }
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

  function updateBullets(value: string) {
    const role = selectedRole();
    if (!role) return;
    role.bullets = normalizeLines(value);
  }

  function addSkillCategory() {
    const label = window.prompt('New skill category name');
    if (!label) return;
    const key = label.trim();
    if (!key || resume.skills[key]) return;
    resume.skills[key] = [];
    skillDrafts[key] = '';
  }

  function removeSkillCategory(key: string) {
    const next = { ...resume.skills };
    delete next[key];
    resume.skills = next;
    const nextDrafts = { ...skillDrafts };
    delete nextDrafts[key];
    skillDrafts = nextDrafts;
  }

  function addExperience() {
    resume.experience = [...resume.experience, blankExperience()];
    selectedExperience = resume.experience.length - 1;
  }

  function removeExperience(index: number) {
    if (resume.experience.length === 1) {
      resume.experience = [blankExperience()];
      selectedExperience = 0;
      return;
    }
    resume.experience = resume.experience.filter((_, current) => current !== index);
    selectedExperience = Math.max(0, Math.min(selectedExperience, resume.experience.length - 1));
  }

  function moveExperience(index: number, delta: number) {
    const target = index + delta;
    if (target < 0 || target >= resume.experience.length) return;
    const next = [...resume.experience];
    const [item] = next.splice(index, 1);
    next.splice(target, 0, item);
    resume.experience = next;
    selectedExperience = target;
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
    content="Edit resume.json and run the local Docker resume build from a Tauri desktop app."
  />
</svelte:head>

<div class="shell">
  <aside class="sidebar">
    <div class="brand">
      <p class="eyebrow">Local Editor</p>
      <h1>Resume Control Room</h1>
      <p class="lede">Edit the source JSON, save it, and trigger the Docker build without leaving the app.</p>
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
      <div class="section-head">
        <p class="eyebrow">Identity</p>
        <h2>Header</h2>
      </div>
      <div class="grid two">
        <label class="stack">
          <span>Name</span>
          <input bind:value={resume.name} />
        </label>
        <label class="stack">
          <span>Email</span>
          <input bind:value={resume.email} />
        </label>
        <label class="stack full">
          <span>Address</span>
          <input bind:value={resume.address} />
        </label>
        <label class="stack">
          <span>Phone</span>
          <input bind:value={resume.phone} />
        </label>
        <label class="stack">
          <span>Website</span>
          <input bind:value={resume.website} />
        </label>
        <label class="stack">
          <span>LinkedIn</span>
          <input bind:value={resume.linkedin} />
        </label>
        <label class="stack">
          <span>GitHub</span>
          <input bind:value={resume.github} />
        </label>
      </div>
    </section>

    <section class="card">
      <div class="section-head">
        <p class="eyebrow">Summary</p>
        <h2>Professional Snapshot</h2>
      </div>
      <label class="stack">
        <span>Summary</span>
        <textarea bind:value={resume.summary} rows="5"></textarea>
      </label>
    </section>

    <section class="card">
      <div class="section-head split">
        <div>
          <p class="eyebrow">Skills</p>
          <h2>Categories</h2>
        </div>
        <button class="secondary" on:click={addSkillCategory}>Add Category</button>
      </div>
      <div class="skill-list">
        {#each Object.keys(skillDrafts) as key}
          <div class="skill-card">
            <div class="skill-card-head">
              <h3>{key}</h3>
              <button class="ghost" on:click={() => removeSkillCategory(key)}>Remove</button>
            </div>
            <textarea bind:value={skillDrafts[key]} rows="4"></textarea>
            <p class="hint">One value per line.</p>
          </div>
        {/each}
      </div>
    </section>

    <section class="card experience-card">
      <div class="section-head split">
        <div>
          <p class="eyebrow">Experience</p>
          <h2>Role History</h2>
        </div>
        <button class="secondary" on:click={addExperience}>Add Role</button>
      </div>

      <div class="experience-layout">
        <div class="experience-list">
          {#each resume.experience as role, index}
            <button
              class:selected={index === selectedExperience}
              class="experience-pill"
              on:click={() => (selectedExperience = index)}
            >
              <span>{role.employer || 'New Employer'}</span>
              <small>{role.job_title || 'Untitled role'}</small>
            </button>
          {/each}
        </div>

        {#if selectedRole()}
          <div class="experience-form">
            <div class="toolbar">
              <button class="ghost" on:click={() => moveExperience(selectedExperience, -1)}>Move Up</button>
              <button class="ghost" on:click={() => moveExperience(selectedExperience, 1)}>Move Down</button>
              <button class="danger" on:click={() => removeExperience(selectedExperience)}>Delete</button>
            </div>

            <div class="grid two">
              <label class="stack">
                <span>Employer</span>
                <input bind:value={resume.experience[selectedExperience].employer} />
              </label>
              <label class="stack">
                <span>Job Title</span>
                <input bind:value={resume.experience[selectedExperience].job_title} />
              </label>
              <label class="stack">
                <span>Location</span>
                <input bind:value={resume.experience[selectedExperience].location} />
              </label>
              <label class="stack">
                <span>Dates</span>
                <input bind:value={resume.experience[selectedExperience].dates} />
              </label>
              <label class="stack full">
                <span>Additional Role</span>
                <input bind:value={resume.experience[selectedExperience].additional_role} />
              </label>
              <label class="stack full">
                <span>Bullets</span>
                <textarea
                  rows="8"
                  value={resume.experience[selectedExperience].bullets.join('\n')}
                  on:input={(event) => updateBullets((event.currentTarget as HTMLTextAreaElement).value)}
                ></textarea>
                <p class="hint">One bullet per line.</p>
              </label>
            </div>
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
      "Iowan Old Style", "Palatino Linotype", "Book Antiqua", Palatino, Georgia, serif;
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
  .build-panel h2 {
    margin: 0;
    font-family: "Avenir Next", "Helvetica Neue", sans-serif;
  }

  .eyebrow {
    margin: 0 0 0.35rem;
    font-family: "Avenir Next", "Helvetica Neue", sans-serif;
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
  .experience-pill small {
    font-family: "Avenir Next", "Helvetica Neue", sans-serif;
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
  .section-head.split {
    display: flex;
    gap: 0.65rem;
    align-items: center;
  }

  .section-head.split {
    justify-content: space-between;
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
    font-family: "Avenir Next", "Helvetica Neue", sans-serif;
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
    font-family: "Avenir Next", "Helvetica Neue", sans-serif;
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
    font-family: "SF Mono", Menlo, Consolas, monospace;
    font-size: 0.84rem;
  }

  .generated-files {
    display: grid;
    gap: 0.6rem;
  }

  .file-link {
    display: grid;
    gap: 0.18rem;
    text-align: left;
    border-radius: 16px;
    background: rgba(255, 251, 245, 0.92);
    color: #291d15;
    padding: 0.8rem 0.95rem;
  }

  .file-link strong,
  .file-link small,
  .empty-state {
    font-family: "Avenir Next", "Helvetica Neue", sans-serif;
  }

  .file-link small {
    color: #6b5643;
    overflow-wrap: anywhere;
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
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  }

  .skill-card {
    padding: 1rem;
    border-radius: 22px;
    background: rgba(248, 240, 229, 0.88);
    border: 1px solid rgba(90, 62, 35, 0.12);
  }

  .skill-card-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.6rem;
  }

  .skill-card-head h3 {
    margin: 0;
    font-family: "Avenir Next", "Helvetica Neue", sans-serif;
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

  .experience-pill.selected {
    background: linear-gradient(135deg, #3e5e63 0%, #1b2f37 100%);
    color: white;
    border-color: rgba(255, 255, 255, 0.28);
  }

  .experience-pill span {
    font-family: "Avenir Next", "Helvetica Neue", sans-serif;
    font-weight: 700;
  }

  .experience-form {
    display: grid;
    gap: 1rem;
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
