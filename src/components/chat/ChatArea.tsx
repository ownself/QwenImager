import { useEffect, useRef } from "react";
import type { MessageDetail } from "@/lib/tauri";
import type { GeneratingStatus } from "@/stores/conversationStore";
import { MessageBubble } from "./MessageBubble";
import { LoadingState } from "@/components/common/LoadingState";
import { cn } from "@/lib/utils";
import { Bot } from "lucide-react";

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
      className={cn("flex-1 overflow-y-auto px-4 py-6")}
    >
      <div className={cn("mx-auto flex max-w-3xl flex-col gap-6")}>
        {messages.map((message) => (
          <MessageBubble key={message.id} message={message} />
        ))}

        {/* Loading indicator during generation */}
        {generatingStatus &&
          generatingStatus !== "succeeded" &&
          generatingStatus !== "failed" && (
            <div className="flex gap-3 animate-in fade-in slide-in-from-bottom-2 duration-300">
              <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-secondary text-muted-foreground ring-2 ring-background shadow-sm">
                <Bot className="h-4 w-4" />
              </div>
              <div className={cn("rounded-2xl bg-secondary px-4 py-3 shadow-sm")}>
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
