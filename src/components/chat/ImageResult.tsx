import { useState } from "react";
import { Copy, Download, Check, X } from "lucide-react";
import { invokeCommand, type GenerationResultInfo } from "@/lib/tauri";
import { cn } from "@/lib/utils";

interface ImageResultProps {
  result: GenerationResultInfo;
}

export function ImageResult({ result }: ImageResultProps) {
  const [copied, setCopied] = useState(false);
  const [saving, setSaving] = useState(false);
  const [previewOpen, setPreviewOpen] = useState(false);

  const imageUrl = result.resourceUrl ?? result.localPath ?? "";

  const handleCopy = async () => {
    if (!imageUrl || copied) return;
    try {
      // For web-based URLs, fetch and copy the image data
      const response = await fetch(imageUrl);
      const blob = await response.blob();
      await navigator.clipboard.write([
        new ClipboardItem({ [blob.type]: blob }),
      ]);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Fallback: copy URL as text
      try {
        await navigator.clipboard.writeText(imageUrl);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
      } catch {
        // Silently fail
      }
    }
  };

  const handleDownload = async () => {
    if (!imageUrl || saving) return;
    setSaving(true);
    try {
      await invokeCommand<string>("save_image", {
        sourceUrl: result.resourceUrl ?? null,
        sourcePath: result.localPath ?? null,
        savePath: null,
      });
    } catch {
      // Error handling is done by ErrorDisplay at a higher level
    } finally {
      setSaving(false);
    }
  };

  if (!imageUrl) return null;

  return (
    <>
      <div className={cn("group relative overflow-hidden rounded-xl border border-border shadow-sm transition-all duration-200 hover:shadow-lg hover:border-primary/20")}>
        <img
          src={imageUrl}
          alt="Generated image"
          className="max-h-96 max-w-full cursor-pointer object-contain transition-transform duration-200 group-hover:scale-[1.02]"
          loading="lazy"
          onClick={() => setPreviewOpen(true)}
        />

        {/* Hover action bar */}
        <div className="absolute right-3 top-3 flex gap-1.5 opacity-0 transition-opacity group-hover:opacity-100">
          <button
            onClick={handleCopy}
            disabled={copied}
            className={cn(
              "flex h-8 w-8 items-center justify-center rounded-lg bg-black/60 text-white backdrop-blur-sm transition-colors hover:bg-black/80"
            )}
            title={copied ? "Copied!" : "Copy image"}
          >
            {copied ? (
              <Check className="h-4 w-4" />
            ) : (
              <Copy className="h-4 w-4" />
            )}
          </button>
          <button
            onClick={handleDownload}
            disabled={saving}
            className={cn(
              "flex h-8 w-8 items-center justify-center rounded-lg bg-black/60 text-white backdrop-blur-sm transition-colors hover:bg-black/80"
            )}
            title="Download image"
          >
            <Download className="h-4 w-4" />
          </button>
        </div>
      </div>

      {/* Full-screen preview dialog */}
      {previewOpen && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-md animate-in fade-in duration-200"
          onClick={() => setPreviewOpen(false)}
        >
          <button
            onClick={() => setPreviewOpen(false)}
            className="absolute right-4 top-4 flex h-10 w-10 items-center justify-center rounded-full bg-white/10 text-white backdrop-blur-sm transition-colors hover:bg-white/20"
          >
            <X className="h-5 w-5" />
          </button>
          <img
            src={imageUrl}
            alt="Preview"
            className="max-h-[90vh] max-w-[90vw] rounded-lg object-contain shadow-2xl animate-in zoom-in-95 duration-200"
            onClick={(e) => e.stopPropagation()}
          />
        </div>
      )}
    </>
  );
}
