import { invoke } from "@tauri-apps/api/core";
import { Channel } from "@tauri-apps/api/core";

// ── Error types ──

export interface AppError {
  kind:
    | "network"
    | "api"
    | "timeout"
    | "config"
    | "unauthorized"
    | "rateLimited"
    | "io"
    | "notFound"
    | "validation";
  message: string;
}

/**
 * Unified Tauri command invocation wrapper with typed error handling.
 * Converts Tauri error responses into `AppError` objects.
 */
export async function invokeCommand<T>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (err: unknown) {
    // Tauri serializes AppError as { kind, message }
    if (
      typeof err === "object" &&
      err !== null &&
      "kind" in err &&
      "message" in err
    ) {
      throw err as AppError;
    }
    // Fallback for unexpected error shapes
    throw {
      kind: "api",
      message: typeof err === "string" ? err : "An unknown error occurred",
    } as AppError;
  }
}

// ── Channel helpers ──

/**
 * Create a Tauri Channel and register a callback for incoming events.
 */
export function createChannel<T>(onMessage: (event: T) => void): Channel<T> {
  const channel = new Channel<T>();
  channel.onmessage = onMessage;
  return channel;
}

// ── Configuration types ──

export interface ConfigStatus {
  loaded: boolean;
  available_models: string[];
  error_message?: string;
}

// ── Generation types ──

export interface GenerationParams {
  size?: string;
  n?: number;
  negative_prompt?: string;
  prompt_extend?: boolean;
  watermark?: boolean;
}

export type GenerationEvent =
  | { event: "submitted"; data: { taskId: string } }
  | { event: "polling"; data: { taskId: string; status: string } }
  | {
      event: "succeeded";
      data: { taskId: string; messageId: string; imageUrls: string[] };
    }
  | { event: "failed"; data: { taskId: string; error: string } };

export interface EditResult {
  messageId: string;
  imageUrls: string[];
}

// ── Conversation types ──

export interface ConversationSummary {
  id: string;
  title: string;
  createdAt: number;
  updatedAt: number;
  messageCount: number;
}

export interface MessageDetail {
  id: string;
  role: "user" | "assistant";
  textContent?: string;
  mode: "text2img" | "img2img" | "translate";
  attachments: AttachmentInfo[];
  results: GenerationResultInfo[];
  extraParams?: Record<string, unknown>;
  createdAt: number;
}

export interface AttachmentInfo {
  id: string;
  filePath: string;
  displayOrder: number;
  fileSize: number;
  mimeType: string;
  source: "upload" | "clipboard";
}

export interface GenerationResultInfo {
  id: string;
  resourceUrl?: string;
  localPath?: string;
  resourceType: "image" | "video";
  modelUsed: string;
  createdAt: number;
}
