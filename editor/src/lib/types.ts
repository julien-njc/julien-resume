export interface ResumeData {
  basics: BasicsData;
  active_profile_id: string;
  profiles: ResumeProfile[];
  skills: SkillEntry[];
  experience: ExperienceEntry[];
  education: EducationData;
  languages: string[];
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

export interface ResumeProfile {
  id: string;
  label: string;
  summary: string;
  skill_ids: string[];
  experience_ids: string[];
  experience_overrides?: Record<string, ExperienceOverride>;
}

export interface SkillEntry {
  id: string;
  category: string;
  label: string;
}

export interface ExperienceEntry {
  id: string;
  employer: string;
  job_title: string;
  location: string;
  dates: string;
  additional_role?: string;
  bullets: string[];
}

export interface ExperienceOverride {
  job_title?: string;
  location?: string;
  dates?: string;
  additional_role?: string;
  bullets?: string[];
}

export interface EducationData {
  degree: string;
  field_of_study: string;
  school: string;
  graduation_date: string;
}

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
