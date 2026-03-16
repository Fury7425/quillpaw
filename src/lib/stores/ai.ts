import { writable } from "svelte/store";
import type { AiModelStatus, AiProposal } from "$lib/types";

export const aiEnabled = writable<boolean>(false);
export const aiProposals = writable<AiProposal[]>([]);
export const aiStatus = writable<string>("AI is off by default.");
export const aiModelPath = writable<string>("");
export const aiDeviceMode = writable<"auto" | "cpu" | "npu">("auto");
export const aiModelInfo = writable<AiModelStatus | null>(null);

export function addProposal(proposal: AiProposal): void {
  aiProposals.update((list) => [proposal, ...list]);
}

export function removeProposal(id: string): void {
  aiProposals.update((list) => list.filter((item) => item.id !== id));
}

export function updateProposal(id: string, content: string): void {
  aiProposals.update((list) =>
    list.map((item) => (item.id === id ? { ...item, content } : item)),
  );
}

export function setModelStatus(status: AiModelStatus | null): void {
  aiModelInfo.set(status);
  if (status?.loaded) {
    aiEnabled.set(true);
    aiStatus.set(status.detail || "AI model loaded.");
    aiModelPath.set(status.model_path ?? "");
  } else {
    aiEnabled.set(false);
    aiStatus.set(status?.detail || "AI is off by default.");
  }
}
