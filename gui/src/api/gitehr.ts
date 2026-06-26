import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export interface JournalEntryInfo {
  filename: string;
  timestamp: string;
  author: string | null;
  content_preview: string;
}

export interface StateFileInfo {
  name: string;
  content: string;
  last_modified: string | null;
}

export interface RepoStatusInfo {
  is_gitehr_repo: boolean;
  gitehr_version: string | null;
  journal_entry_count: number;
  state_files: string[];
  is_encrypted: boolean;
}

export interface ContributorInfo {
  id: string;
  name: string;
  role: string | null;
  email: string | null;
  enabled: boolean;
  active: boolean;
}

export interface MpiIdentifier {
  type: string;
  value: string;
}

export interface MpiPatientInfo {
  patient_id: string;
  repo_path: string;
  status: string;
  merged_into: string | null;
  updated_at: string;
  identifiers: MpiIdentifier[];
}

export interface MpiInfo {
  version: number;
  updated_at: string;
  patients: MpiPatientInfo[];
  store_root: string;
}

export async function getCurrentDir(): Promise<string> {
  return invoke<string>("get_current_dir");
}

export async function isGitehrRepo(path: string): Promise<boolean> {
  return invoke<boolean>("is_gitehr_repo", { path });
}

export async function hasMpi(path: string): Promise<boolean> {
  return invoke<boolean>("has_mpi", { path });
}

export async function getMpi(path: string): Promise<MpiInfo> {
  return invoke<MpiInfo>("get_mpi", { path });
}

export async function pickFolder(): Promise<string | null> {
  const result = await open({
    directory: true,
    multiple: false,
    title: "Select GitEHR Repository or Store Root",
  });
  return result as string | null;
}

export async function getStatus(repoPath: string): Promise<RepoStatusInfo> {
  return invoke<RepoStatusInfo>("get_status", { repoPath });
}

export async function getJournalEntries(
  repoPath: string,
  options?: {
    limit?: number;
    offset?: number;
    reverse?: boolean;
  }
): Promise<JournalEntryInfo[]> {
  return invoke<JournalEntryInfo[]>("get_journal_entries", {
    repoPath,
    limit: options?.limit,
    offset: options?.offset,
    reverse: options?.reverse,
  });
}

export async function getStateFiles(
  repoPath: string
): Promise<StateFileInfo[]> {
  return invoke<StateFileInfo[]>("get_state_files", { repoPath });
}

export async function getStateFile(
  repoPath: string,
  filename: string
): Promise<StateFileInfo> {
  return invoke<StateFileInfo>("get_state_file", { repoPath, filename });
}

export async function updateStateFile(
  repoPath: string,
  filename: string,
  content: string
): Promise<void> {
  return invoke<void>("update_state_file", { repoPath, filename, content });
}

export async function addJournalEntry(
  repoPath: string,
  content: string
): Promise<string> {
  return invoke<string>("add_journal_entry", { repoPath, content });
}

export async function getContributors(
  repoPath: string
): Promise<ContributorInfo[]> {
  return invoke<ContributorInfo[]>("get_contributors", { repoPath });
}

export async function getCurrentContributor(
  repoPath: string
): Promise<string | null> {
  return invoke<string | null>("get_current_contributor", { repoPath });
}

export async function activateContributor(
  repoPath: string,
  contributorId: string
): Promise<void> {
  return invoke<void>("activate_contributor", { repoPath, contributorId });
}

export async function initStoreRoot(
  path: string,
  name?: string
): Promise<string> {
  return invoke<string>("init_store_root", { path, name });
}

export async function addStoreSubject(
  path: string,
  name?: string
): Promise<string> {
  return invoke<string>("add_store_subject", { path, name });
}
