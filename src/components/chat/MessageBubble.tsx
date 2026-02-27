import type { MessageDetail } from "@/lib/tauri";
import { ImageResult } from "./ImageResult";
import { User, Bot } from "lucide-react";
import { convertFileSrc } from "@tauri-apps/api/core";

interface MessageBubbleProps {
  message: MessageDetail;
}

export function MessageBubble({ message }: MessageBubbleProps) {
  const isUser = message.role === "user";

  return (
    <div
      className={`flex gap-3 ${isUser ? "flex-row-reverse" : "flex-row"}`}
    >
      {/* Avatar */}
      <div
        className={`flex h-8 w-8 shrink-0 items-center justify-center rounded-full ${
          isUser
            ? "bg-primary text-primary-foreground"
            : "bg-muted text-muted-foreground"
        }`}
      >
        {isUser ? <User className="h-4 w-4" /> : <Bot className="h-4 w-4" />}
      </div>

      {/* Content */}
      <div
        className={`flex max-w-[75%] flex-col gap-2 ${
          isUser ? "items-end" : "items-start"
        }`}
      >
        {/* Text content */}
        {message.textContent && (
          <div
            className={`rounded-lg px-3 py-2 text-sm ${
              isUser
                ? "bg-primary text-primary-foreground"
                : "bg-muted text-foreground"
            }`}
          >
            <p className="whitespace-pre-wrap break-words">
              {message.textContent}
            </p>
          </div>
        )}

        {/* Uploaded attachments (reference images for img2img) */}
        {message.attachments.length > 0 && (
          <div className="flex flex-wrap gap-1.5">
            {message.attachments.map((att) => (
              <div
                key={att.id}
                className="relative h-16 w-16 overflow-hidden rounded-md border border-border"
              >
                <img
                  src={convertFileSrc(att.filePath)}
                  alt={`Reference ${att.displayOrder + 1}`}
                  className="h-full w-full object-cover"
                  loading="lazy"
                />
                <span className="absolute bottom-0.5 left-0.5 flex h-4 w-4 items-center justify-center rounded-full bg-black/60 text-[8px] font-bold text-white">
                  {att.displayOrder + 1}
                </span>
              </div>
            ))}
          </div>
        )}

        {/* Generation results (images) */}
        {message.results.length > 0 && (
          <div className="flex flex-wrap gap-2">
            {message.results.map((result) => (
              <ImageResult key={result.id} result={result} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
