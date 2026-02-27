import { useCallback, useEffect, useRef } from "react";
import { invokeCommand } from "@/lib/tauri";

interface UseClipboardOptions {
  /** Element to attach paste listener to. If not set, uses document. */
  enabled?: boolean;
  onPasteImage?: (filePath: string) => void;
  onError?: (error: string) => void;
}

/**
 * Hook that listens for paste events and saves clipboard images
 * to a temporary file via the backend, returning the file path.
 */
export function useClipboard({
  enabled = true,
  onPasteImage,
  onError,
}: UseClipboardOptions) {
  const onPasteImageRef = useRef(onPasteImage);
  const onErrorRef = useRef(onError);
  onPasteImageRef.current = onPasteImage;
  onErrorRef.current = onError;

  const handlePaste = useCallback(
    async (e: ClipboardEvent) => {
      if (!enabled) return;

      const items = e.clipboardData?.items;
      if (!items) return;

      for (const item of Array.from(items)) {
        if (!item.type.startsWith("image/")) continue;

        e.preventDefault();

        const blob = item.getAsFile();
        if (!blob) continue;

        try {
          // Convert blob to Uint8Array
          const arrayBuffer = await blob.arrayBuffer();
          const imageData = Array.from(new Uint8Array(arrayBuffer));

          // Save to temp file via backend
          const filePath = await invokeCommand<string>(
            "save_clipboard_image",
            {
              imageData,
              mimeType: item.type,
            }
          );

          onPasteImageRef.current?.(filePath);
        } catch (err) {
          const message =
            typeof err === "object" && err !== null && "message" in err
              ? (err as { message: string }).message
              : "Failed to save clipboard image";
          onErrorRef.current?.(message);
        }

        // Only handle the first image
        break;
      }
    },
    [enabled]
  );

  useEffect(() => {
    if (!enabled) return;

    document.addEventListener("paste", handlePaste);
    return () => {
      document.removeEventListener("paste", handlePaste);
    };
  }, [enabled, handlePaste]);
}
