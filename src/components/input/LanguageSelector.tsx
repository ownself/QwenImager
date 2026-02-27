import { ArrowRightLeft } from "lucide-react";
import { cn } from "@/lib/utils";

interface LanguageSelectorProps {
  sourceLang: string;
  targetLang: string;
  disabled?: boolean;
  onSourceLangChange: (lang: string) => void;
  onTargetLangChange: (lang: string) => void;
}

const LANGUAGES = [
  { code: "zh", label: "中文" },
  { code: "en", label: "English" },
  { code: "ja", label: "日本語" },
  { code: "ko", label: "한국어" },
  { code: "fr", label: "Français" },
  { code: "de", label: "Deutsch" },
  { code: "es", label: "Español" },
  { code: "ru", label: "Русский" },
];

export function LanguageSelector({
  sourceLang,
  targetLang,
  disabled = false,
  onSourceLangChange,
  onTargetLangChange,
}: LanguageSelectorProps) {
  const handleSwap = () => {
    if (disabled) return;
    onSourceLangChange(targetLang);
    onTargetLangChange(sourceLang);
  };

  return (
    <div className={cn("flex items-center gap-3 rounded-xl bg-secondary/50 px-3 py-2")}>
      <select
        value={sourceLang}
        onChange={(e) => {
          const val = e.target.value;
          // If same as target, swap
          if (val === targetLang) {
            onTargetLangChange(sourceLang);
          }
          onSourceLangChange(val);
        }}
        disabled={disabled}
        className={cn("appearance-none rounded-lg border border-border bg-background px-3 py-1.5 text-sm text-foreground shadow-sm transition-shadow focus:outline-none focus:ring-2 focus:ring-ring focus:shadow-md focus:shadow-ring/10 disabled:cursor-not-allowed disabled:opacity-50")}
      >
        {LANGUAGES.map((lang) => (
          <option key={lang.code} value={lang.code}>
            {lang.label}
          </option>
        ))}
      </select>

      <button
        onClick={handleSwap}
        disabled={disabled}
        className={cn("flex h-8 w-8 items-center justify-center rounded-lg border border-border bg-background text-muted-foreground shadow-sm transition-all duration-200 hover:bg-accent hover:text-foreground hover:scale-110 active:scale-95 disabled:cursor-not-allowed disabled:opacity-50")}
        title="Swap languages"
      >
        <ArrowRightLeft className="h-4 w-4" />
      </button>

      <select
        value={targetLang}
        onChange={(e) => {
          const val = e.target.value;
          // If same as source, swap
          if (val === sourceLang) {
            onSourceLangChange(targetLang);
          }
          onTargetLangChange(val);
        }}
        disabled={disabled}
        className={cn("appearance-none rounded-lg border border-border bg-background px-3 py-1.5 text-sm text-foreground shadow-sm transition-shadow focus:outline-none focus:ring-2 focus:ring-ring focus:shadow-md focus:shadow-ring/10 disabled:cursor-not-allowed disabled:opacity-50")}
      >
        {LANGUAGES.map((lang) => (
          <option key={lang.code} value={lang.code}>
            {lang.label}
          </option>
        ))}
      </select>
    </div>
  );
}
