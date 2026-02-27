import { useEffect } from "react";
import { useConfigStore } from "@/stores/configStore";
import { useConversationStore } from "@/stores/conversationStore";
import { useUIStore } from "@/stores/uiStore";
import { MainPanel } from "@/components/layout/MainPanel";
import { Sidebar } from "@/components/layout/Sidebar";
import { ConfigGuide } from "@/components/common/ConfigGuide";
import { LoadingState } from "@/components/common/LoadingState";
import { TooltipProvider } from "@/components/ui/tooltip";

function App() {
  const loadConfig = useConfigStore((s) => s.loadConfig);
  const loading = useConfigStore((s) => s.loading);
  const configLoaded = useConfigStore((s) => s.loaded);
  const configError = useConfigStore((s) => s.errorMessage);

  const conversations = useConversationStore((s) => s.conversations);
  const currentConversationId = useConversationStore(
    (s) => s.currentConversationId
  );
  const loadConversations = useConversationStore((s) => s.loadConversations);
  const switchConversation = useConversationStore((s) => s.switchConversation);
  const deleteConversation = useConversationStore((s) => s.deleteConversation);
  const newConversation = useConversationStore((s) => s.newConversation);

  const sidebarCollapsed = useUIStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);

  useEffect(() => {
    loadConfig();
    loadConversations();
  }, [loadConfig, loadConversations]);

  if (loading) {
    return (
      <div className="flex h-screen w-screen items-center justify-center bg-background">
        <LoadingState message="Loading configuration..." />
      </div>
    );
  }

  // Show ConfigGuide when config is not loaded
  if (!configLoaded) {
    return (
      <div className="flex h-screen w-screen bg-background text-foreground">
        <ConfigGuide errorMessage={configError ?? undefined} />
      </div>
    );
  }

  return (
    <TooltipProvider>
      <div className="flex h-screen w-screen bg-background text-foreground">
        <Sidebar
          conversations={conversations}
          currentConversationId={currentConversationId}
          collapsed={sidebarCollapsed}
          onToggle={toggleSidebar}
          onNewConversation={newConversation}
          onSelectConversation={switchConversation}
          onDeleteConversation={deleteConversation}
        />
        <MainPanel />
      </div>
    </TooltipProvider>
  );
}

export default App;
