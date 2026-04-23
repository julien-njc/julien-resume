export interface ResumeData {
  name: string;
  address: string;
  email: string;
  phone: string;
  linkedin: string;
  github: string;
  website: string;
  summary: string;
  skills: Record<string, string[]>;
  experience: ExperienceEntry[];
  education: EducationData;
  languages: string[];
}

export interface ExperienceEntry {
  employer: string;
  job_title: string;
  location: string;
  dates: string;
  additional_role?: string;
  bullets: string[];
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
