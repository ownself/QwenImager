import { useCallback } from "react";
import { useConversationStore } from "@/stores/conversationStore";
import type { GenerationParams } from "@/lib/tauri";

/**
 * Hook that wraps the text-to-image generation workflow.
 * Manages generation state and provides a simple `generate()` API.
 */
export function useGeneration() {
  const generatingStatus = useConversationStore((s) => s.generatingStatus);
  const generatingError = useConversationStore((s) => s.generatingError);
  const generatingTaskId = useConversationStore((s) => s.generatingTaskId);
  const sendPrompt = useConversationStore((s) => s.sendPrompt);
  const resetGenerating = useConversationStore((s) => s.resetGenerating);

  const isGenerating = generatingStatus !== null && generatingStatus !== "failed";

  const generate = useCallback(
    async (prompt: string, params?: GenerationParams) => {
      if (isGenerating) return;
      await sendPrompt(prompt, params);
    },
    [isGenerating, sendPrompt]
  );

  return {
    generate,
    isGenerating,
    status: generatingStatus,
    taskId: generatingTaskId,
    error: generatingError,
    resetError: resetGenerating,
  };
}
