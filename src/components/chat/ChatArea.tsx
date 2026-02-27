import { useEffect, useRef } from "react";
import type { MessageDetail } from "@/lib/tauri";
import type { GeneratingStatus } from "@/stores/conversationStore";
import { MessageBubble } from "./MessageBubble";
import { LoadingState } from "@/components/common/LoadingState";

interface ChatAreaProps {
  messages: MessageDetail[];
  loading?: boolean;
  generatingStatus: GeneratingStatus;
}

const statusMessages: Record<string, string> = {
  submitted: "Submitting request...",
  polling: "Generating image...",
};

export function ChatArea({
  messages,
  loading = false,
  generatingStatus,
}: ChatAreaProps) {
  const scrollRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when messages change or status changes
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages, generatingStatus]);

  if (loading) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <LoadingState message="Loading conversation..." />
      </div>
    );
  }

  if (messages.length === 0 && !generatingStatus) {
    return (
      <div className="flex flex-1 items-center justify-center text-muted-foreground">
        <div className="text-center">
          <p className="text-lg font-medium">Start a conversation</p>
          <p className="mt-1 text-sm">
            Type a prompt below to generate an image
          </p>
        </div>
      </div>
    );
  }

  return (
    <div
      ref={scrollRef}
      className="flex-1 overflow-y-auto px-4 py-4"
    >
      <div className="mx-auto flex max-w-3xl flex-col gap-4">
        {messages.map((message) => (
          <MessageBubble key={message.id} message={message} />
        ))}

        {/* Loading indicator during generation */}
        {generatingStatus &&
          generatingStatus !== "succeeded" &&
          generatingStatus !== "failed" && (
            <div className="flex gap-3">
              <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground">
                <span className="text-xs">AI</span>
              </div>
              <div className="rounded-lg bg-muted px-3 py-2">
                <LoadingState
                  message={
                    statusMessages[generatingStatus] ?? "Processing..."
                  }
                />
              </div>
            </div>
          )}
      </div>
    </div>
  );
}
