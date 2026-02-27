import { ImagePlus, Languages, Type } from "lucide-react";
import type { Mode } from "@/stores/conversationStore";

interface ModeSelectorProps {
  currentMode: Mode;
  disabled?: boolean;
  onModeChange: (mode: Mode) => void;
}

const modes: { value: Mode; label: string; icon: React.ElementType; enabled: boolean }[] = [
  { value: "text2img", label: "Text to Image", icon: Type, enabled: true },
  { value: "img2img", label: "Image to Image", icon: ImagePlus, enabled: true },
  { value: "translate", label: "Translate", icon: Languages, enabled: true },
];

export function ModeSelector({
  currentMode,
  disabled = false,
  onModeChange,
}: ModeSelectorProps) {
  return (
    <div className="flex gap-1 rounded-lg bg-muted p-1">
      {modes.map(({ value, label, icon: Icon, enabled }) => (
        <button
          key={value}
          onClick={() => onModeChange(value)}
          disabled={disabled || !enabled}
          className={`flex items-center gap-1.5 rounded-md px-3 py-1.5 text-xs font-medium transition-colors
            ${
              currentMode === value
                ? "bg-background text-foreground shadow-sm"
                : "text-muted-foreground hover:text-foreground"
            }
            ${disabled || !enabled ? "cursor-not-allowed opacity-50" : "cursor-pointer"}
          `}
          title={!enabled ? "Coming soon" : label}
        >
          <Icon className="h-3.5 w-3.5" />
          <span>{label}</span>
        </button>
      ))}
    </div>
  );
}
