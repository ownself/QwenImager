import { useState, useCallback, useRef, useEffect } from "react";
import { Plus, Trash2, PanelLeftClose, PanelLeft } from "lucide-react";
import type { ConversationSummary } from "@/lib/tauri";
import { cn } from "@/lib/utils";
import {
  Tooltip,
  TooltipTrigger,
  TooltipContent,
} from "@/components/ui/tooltip";
import { useUIStore } from "@/stores/uiStore";

interface SidebarProps {
  conversations: ConversationSummary[];
  currentConversationId: string | null;
  collapsed: boolean;
  onToggle: () => void;
  onNewConversation: () => void;
  onSelectConversation: (id: string) => void;
  onDeleteConversation: (id: string) => void;
}

function formatTime(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  const now = new Date();
  const diffDays = Math.floor(
    (now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24)
  );

  if (diffDays === 0) {
    return date.toLocaleTimeString(undefined, {
      hour: "2-digit",
      minute: "2-digit",
    });
  }
  if (diffDays === 1) return "Yesterday";
  if (diffDays < 7) return `${diffDays}d ago`;
  return date.toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
  });
}

/** Individual conversation list item with hover-to-show delete button. */
function ConversationItem({
  conv,
  isActive,
  onSelect,
  onDelete,
}: {
  conv: ConversationSummary;
  isActive: boolean;
  onSelect: (id: string) => void;
  onDelete: (id: string) => void;
}) {
  const [hovered, setHovered] = useState(false);

  return (
    <div
      onClick={() => onSelect(conv.id)}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      className={cn(
        "mx-2 flex cursor-pointer items-center rounded-lg px-2 py-2.5 text-sm transition-colors duration-150",
        isActive
          ? "bg-primary/10 text-foreground shadow-sm border-l-2 border-l-primary"
          : "text-muted-foreground hover:bg-accent hover:text-foreground"
      )}
    >
      <div className="min-w-0 flex-1">
        <div className="truncate font-medium">{conv.title}</div>
        <div className="flex items-center gap-1 text-xs text-muted-foreground">
          <span>{formatTime(conv.updatedAt)}</span>
          <span>&middot;</span>
          <span>{conv.messageCount} msgs</span>
        </div>
      </div>
      {hovered && (
        <Tooltip>
          <TooltipTrigger asChild>
            <button
              onClick={(e) => {
                e.stopPropagation();
                onDelete(conv.id);
              }}
              className="ml-1 flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground transition-colors hover:bg-destructive/10 hover:text-destructive"
            >
              <Trash2 className="h-3 w-3" />
            </button>
          </TooltipTrigger>
          <TooltipContent side="right">Delete conversation</TooltipContent>
        </Tooltip>
      )}
    </div>
  );
}

export function Sidebar({
  conversations,
  currentConversationId,
  collapsed,
  onToggle,
  onNewConversation,
  onSelectConversation,
  onDeleteConversation,
}: SidebarProps) {
  const sidebarWidth = useUIStore((s) => s.sidebarWidth);
  const setSidebarWidth = useUIStore((s) => s.setSidebarWidth);
  const [isResizing, setIsResizing] = useState(false);
  const startXRef = useRef<number>(0);
  const startWidthRef = useRef<number>(0);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
    startXRef.current = e.clientX;
    startWidthRef.current = sidebarWidth;
  }, [sidebarWidth]);

  const handleMouseMove = useCallback((e: MouseEvent) => {
    if (!isResizing) return;
    const diff = e.clientX - startXRef.current;
    setSidebarWidth(startWidthRef.current + diff);
  }, [isResizing, setSidebarWidth]);

  const handleMouseUp = useCallback(() => {
    setIsResizing(false);
  }, []);

  useEffect(() => {
    if (isResizing) {
      document.addEventListener("mousemove", handleMouseMove);
      document.addEventListener("mouseup", handleMouseUp);
      document.body.style.cursor = "col-resize";
      document.body.style.userSelect = "none";
    }
    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };
  }, [isResizing, handleMouseMove, handleMouseUp]);

  return (
    <div
      className={cn(
        "relative flex h-full flex-col border-r border-border bg-secondary shadow-sm transition-[width] duration-150 ease-out overflow-hidden",
        collapsed ? "w-12" : "w-12"
      )}
      style={!collapsed ? { width: sidebarWidth } : undefined}
    >
      {/* Resize handle */}
      {!collapsed && (
        <div
          className={cn(
            "absolute right-0 top-0 bottom-0 w-1 cursor-col-resize bg-transparent hover:bg-primary/30 transition-colors z-10",
            isResizing && "bg-primary/50"
          )}
          onMouseDown={handleMouseDown}
        />
      )}
      {/* Header */}
      <div
        className={cn(
          "flex shrink-0 items-center border-b border-border px-4 py-3",
          collapsed ? "justify-center" : "justify-between"
        )}
      >
        {!collapsed && (
          <Tooltip>
            <TooltipTrigger asChild>
              <button
                onClick={onNewConversation}
                className={cn(
                  "flex items-center gap-1.5 rounded-lg px-2 py-1.5 text-sm font-medium text-foreground transition-colors hover:bg-accent",
                  "animate-in fade-in duration-200"
                )}
              >
                <Plus className="h-4 w-4" />
                <span>New Chat</span>
              </button>
            </TooltipTrigger>
            <TooltipContent side="bottom">New conversation</TooltipContent>
          </Tooltip>
        )}

        {collapsed && (
          <Tooltip>
            <TooltipTrigger asChild>
              <button
                onClick={onNewConversation}
                className="flex h-8 w-8 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
              >
                <Plus className="h-4 w-4" />
              </button>
            </TooltipTrigger>
            <TooltipContent side="right">New conversation</TooltipContent>
          </Tooltip>
        )}

        <Tooltip>
          <TooltipTrigger asChild>
            <button
              onClick={onToggle}
              className={cn(
                "flex h-8 w-8 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:bg-accent hover:text-foreground",
                collapsed && "mt-2"
              )}
            >
              {collapsed ? (
                <PanelLeft className="h-4 w-4" />
              ) : (
                <PanelLeftClose className="h-4 w-4" />
              )}
            </button>
          </TooltipTrigger>
          <TooltipContent side={collapsed ? "right" : "bottom"}>
            {collapsed ? "Expand sidebar" : "Collapse sidebar"}
          </TooltipContent>
        </Tooltip>
      </div>

      {/* Conversation list */}
      {!collapsed && (
        <div className="flex-1 overflow-y-auto overflow-x-hidden">
          {conversations.length === 0 ? (
            <div className="px-3 py-6 text-center text-xs text-muted-foreground animate-in fade-in duration-200">
              No conversations yet
            </div>
          ) : (
            <div className="py-1">
              {conversations.map((conv) => (
                <ConversationItem
                  key={conv.id}
                  conv={conv}
                  isActive={currentConversationId === conv.id}
                  onSelect={onSelectConversation}
                  onDelete={onDeleteConversation}
                />
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
