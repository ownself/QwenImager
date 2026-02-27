import { useMemo } from "react";
import { useConfigStore } from "@/stores/configStore";
import { useConversationStore } from "@/stores/conversationStore";
import { cn } from "@/lib/utils";

interface ModelSelectorProps {
  serviceType: "text2img" | "img2img" | "translate";
}

export function ModelSelector({ serviceType }: ModelSelectorProps) {
  const availableModels = useConfigStore((s) => s.availableModels);
  const selectedModel = useConversationStore((s) => s.selectedModel);
  const setSelectedModel = useConversationStore((s) => s.setSelectedModel);

  const models = useMemo(
    () => availableModels.filter((m) => m.serviceType === serviceType),
    [availableModels, serviceType]
  );

  // Don't render if zero or one model — auto-select
  if (models.length <= 1) {
    return null;
  }

  return (
    <select
      value={selectedModel ?? ""}
      onChange={(e) => {
        const val = e.target.value;
        setSelectedModel(val === "" ? null : val);
      }}
      className={cn("appearance-none rounded-lg border border-border bg-background px-3 py-1.5 pr-8 text-xs text-foreground shadow-sm transition-shadow focus:outline-none focus:ring-2 focus:ring-ring focus:shadow-md focus:shadow-ring/10")}
    >
      <option value="">Default</option>
      {models.map((m) => (
        <option key={`${m.provider}/${m.name}`} value={m.name}>
          {m.provider} / {m.name}
        </option>
      ))}
    </select>
  );
}
