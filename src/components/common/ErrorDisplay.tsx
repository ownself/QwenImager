import type { AppError } from "@/lib/tauri";
import { AlertCircle, KeyRound, Wifi, Clock, Settings, FileWarning } from "lucide-react";

interface ErrorDisplayProps {
  error: AppError | string;
  className?: string;
}

const iconMap: Record<string, React.ElementType> = {
  unauthorized: KeyRound,
  network: Wifi,
  timeout: Clock,
  config: Settings,
  validation: FileWarning,
};

const hintMap: Record<string, string> = {
  unauthorized: "Please check your API key in ~/.qwenimage/setting.json",
  network: "Please check your network connection",
  timeout: "The request timed out. Please try again.",
  config: "Please check your configuration file",
  rateLimited: "API rate limit reached. Please wait and try again.",
  validation: "Please check the input parameters",
  io: "A file system error occurred",
  notFound: "The requested resource was not found",
  api: "An API error occurred",
};

export function ErrorDisplay({ error, className = "" }: ErrorDisplayProps) {
  const isString = typeof error === "string";
  const kind = isString ? "api" : error.kind;
  const message = isString ? error : error.message;
  const hint = hintMap[kind] ?? "An unexpected error occurred";
  const Icon = iconMap[kind] ?? AlertCircle;

  return (
    <div
      className={`flex items-start gap-3 rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-800 dark:border-red-800 dark:bg-red-950 dark:text-red-200 ${className}`}
    >
      <Icon className="mt-0.5 h-5 w-5 shrink-0" />
      <div className="min-w-0">
        <p className="font-medium">{hint}</p>
        <p className="mt-1 text-red-600 dark:text-red-400 break-words">
          {message}
        </p>
      </div>
    </div>
  );
}
