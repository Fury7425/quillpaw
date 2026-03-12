export type FileNode = {
  name: string;
  path: string;
  is_folder: boolean;
  children: FileNode[] | null;
};

export type NoteContent = {
  path: string;
  title: string;
  body: string;
  tags: string[];
  created: string;
  modified: string;
};

export type SearchResult = {
  path: string;
  title: string;
  snippet: string;
  score: number;
  result_type: string;
};

export type AiProposal = {
  id: string;
  proposal_type: string;
  title: string;
  content: string;
  target_path?: string | null;
  metadata?: unknown;
};

export type AiModelStatus = {
  loaded: boolean;
  model_path?: string | null;
  device_mode: string;
  npu_available: boolean;
  detail: string;
};

export type ModelDownloadProgress = {
  model_id: string;
  downloaded: number;
  total?: number | null;
  done: boolean;
  path?: string | null;
  error?: string | null;
};
