import type { MessageDetail } from "@/lib/tauri";
import { ImageResult } from "./ImageResult";
import { User, Bot } from "lucide-react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { cn } from "@/lib/utils";

interface MessageBubbleProps {
  message: MessageDetail;
}

export function MessageBubble({ message }: MessageBubbleProps) {
  const isUser = message.role === "user";

  return (
    <div
      className={cn(
        "flex gap-3 animate-in fade-in slide-in-from-bottom-2 duration-300",
        isUser ? "flex-row-reverse" : "flex-row"
      )}
    >
      {/* Avatar */}
      <div
        className={cn(
          "flex h-8 w-8 shrink-0 items-center justify-center rounded-full ring-2 ring-background shadow-sm",
          isUser
            ? "bg-primary text-primary-foreground"
            : "bg-secondary text-muted-foreground"
        )}
      >
        {isUser ? <User className="h-4 w-4" /> : <Bot className="h-4 w-4" />}
      </div>

      {/* Content */}
      <div
        className={cn(
          "flex max-w-[80%] flex-col gap-2",
          isUser ? "items-end" : "items-start"
        )}
      >
        {/* Text content */}
        {message.textContent && (
          <div
            className={cn(
              "rounded-2xl px-4 py-3 text-sm",
              isUser
                ? "bg-primary text-primary-foreground shadow-md shadow-primary/20"
                : "bg-secondary text-foreground shadow-sm"
            )}
          >
            <p className="whitespace-pre-wrap break-words">
              {message.textContent}
            </p>
          </div>
        )}

        {/* Uploaded attachments (reference images for img2img) */}
        {message.attachments.length > 0 && (
          <div className="flex flex-wrap gap-2">
            {message.attachments.map((att) => (
              <div
                key={att.id}
                className="relative h-20 w-20 overflow-hidden rounded-lg border border-border"
              >
                <img
                  src={convertFileSrc(att.filePath)}
                  alt={`Reference ${att.displayOrder + 1}`}
                  className="h-full w-full object-cover"
                  loading="lazy"
                />
                <span className="absolute bottom-0.5 left-0.5 flex h-5 w-5 items-center justify-center rounded-full bg-black/60 text-xs font-bold text-white">
                  {att.displayOrder + 1}
                </span>
              </div>
            ))}
          </div>
        )}

        {/* Generation results (images) */}
        {message.results.length > 0 && (
          <div className="flex flex-wrap gap-3">
            {message.results.map((result) => (
              <ImageResult key={result.id} result={result} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
