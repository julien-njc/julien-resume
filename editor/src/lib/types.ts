// Every list item in any section declares which profiles it belongs to.
// profile_ids: [] or missing means the item is excluded from all resume outputs.
export interface ProfileScoped {
  profile_ids: string[];
}

export interface ResumeData {
  basics: BasicsData;
  writing_style: WritingStyleData;
  active_profile_id: string;
  profiles: ResumeProfile[];
  summary?: string;
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

export interface BasicsData {
  name: string;
  address: string;
  email: string;
  phone: string;
  linkedin: string;
  github: string;
  website: string;
}

// User-authored prose describing their natural writing voice.
// Injected into all AI generation prompts (cover letters, translation).
export interface WritingStyleData {
  context: string;
}

// Profiles define identity only. Item membership is declared on each item via profile_ids.
export interface ResumeProfile {
  id: string;
  label: string;
  summary: string;
  chips?: string[];
}

export interface SkillEntry extends ProfileScoped {
  id: string;
  category: string;
  label: string;
  years_of_experience?: number;
  usage_examples?: string[];
}

export interface ExperienceEntry extends ProfileScoped {
  id: string;
  employer: string;
  job_title: string;
  location: string;
  dates: string;
  additional_role?: string;
  bullets: string[];
  overrides?: Record<string, ExperienceOverride>;
}

export interface ExperienceOverride {
  job_title?: string;
  additional_role?: string;
  bullets?: string[];
}

export interface ProjectEntry extends ProfileScoped {
  id: string;
  name: string;
  url?: string;
  role?: string;
  dates?: string;
  bullets: string[];
  overrides?: Record<string, { bullets?: string[] }>;
}

export interface EducationEntry extends ProfileScoped {
  id: string;
  degree: string;
  field_of_study: string;
  school: string;
  graduation_date: string;
  notes?: string;
}

export interface CertificationEntry extends ProfileScoped {
  id: string;
  name: string;
  issuer: string;
  date: string;
  expiry?: string;
  url?: string;
}

export interface AwardEntry extends ProfileScoped {
  id: string;
  title: string;
  issuer: string;
  date: string;
  description?: string;
}

export interface PublicationEntry extends ProfileScoped {
  id: string;
  title: string;
  publisher: string;
  date: string;
  url?: string;
  description?: string;
}

export interface VolunteerEntry extends ProfileScoped {
  id: string;
  organization: string;
  role: string;
  dates: string;
  bullets: string[];
  overrides?: Record<string, { bullets?: string[] }>;
}

export interface ReferenceEntry extends ProfileScoped {
  id: string;
  name: string;
  title: string;
  company: string;
  relationship?: string;
  email?: string;
  phone?: string;
  notes?: string;
}

export interface SpokenLanguageEntry extends ProfileScoped {
  id: string;
  name: string;
  proficiency: string;
}

// --- Build output types ---

export interface BuildResult {
  status: number;
  stdout: string;
  stderr: string;
  generated_files: GeneratedFile[];
}

export interface GeneratedFile {
  label: string;
  path: string;
  relative_path: string;
}

// --- Application tracker (applications.json) ---

export interface ApplicationRecord {
  id: string;
  job_url: string;
  job_title: string;
  company: string;
  date_applied: string;
  profile_id: string;
  language: string;
  match_score?: number;
  cover_letter_submitted?: string;
  notes?: string;
  resume_snapshot: ResumeData;
  translated_snapshot: ResumeData | null;
}

export interface ApplicationsData {
  applications: ApplicationRecord[];
}
