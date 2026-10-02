
export interface Project {
  id: string;
  name: string;
  description?: string;
  created_at: string;
}

export interface CreateProjectInput {
  name: string;
  description?: string;
}

export interface UpdateProjectInput {
  id: string;
  name: string;
  description?: string;
}

