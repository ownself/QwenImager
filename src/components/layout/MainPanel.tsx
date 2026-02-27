import { useConversationStore } from "@/stores/conversationStore";
import { useGeneration } from "@/hooks/useGeneration";
import { ChatArea } from "@/components/chat/ChatArea";
import { PromptInput } from "@/components/input/PromptInput";
import { ModeSelector } from "@/components/input/ModeSelector";
import { ModelSelector } from "@/components/input/ModelSelector";
import { ImageUpload } from "@/components/input/ImageUpload";
import { LanguageSelector } from "@/components/input/LanguageSelector";
import { ErrorDisplay } from "@/components/common/ErrorDisplay";
import { cn } from "@/lib/utils";
import { Type, ImagePlus, Languages } from "lucide-react";

export function MainPanel() {
  const currentConversationId = useConversationStore(
    (s) => s.currentConversationId
  );
  const messages = useConversationStore((s) => s.messages);
  const mode = useConversationStore((s) => s.mode);
  const setMode = useConversationStore((s) => s.setMode);
  const generatingStatus = useConversationStore((s) => s.generatingStatus);

  // Image upload state
  const uploadedImages = useConversationStore((s) => s.uploadedImages);
  const addImages = useConversationStore((s) => s.addImages);
  const removeImage = useConversationStore((s) => s.removeImage);
  const reorderImages = useConversationStore((s) => s.reorderImages);

  // Translation language state
  const sourceLang = useConversationStore((s) => s.sourceLang);
  const targetLang = useConversationStore((s) => s.targetLang);
  const setSourceLang = useConversationStore((s) => s.setSourceLang);
  const setTargetLang = useConversationStore((s) => s.setTargetLang);

  const { generate, isGenerating, error: genError } = useGeneration();

  const showWelcome = !currentConversationId && messages.length === 0;

  return (
    <div className="flex flex-1 flex-col">
      {/* Welcome page or chat area */}
      {showWelcome ? (
        <div className="flex flex-1 flex-col items-center justify-center gap-8 p-8">
          <div className="text-center">
            <h1 className="bg-gradient-to-r from-primary via-primary/80 to-primary/60 bg-clip-text text-3xl font-bold tracking-tight text-transparent">
              QwenImager
            </h1>
            <p className="mt-3 text-sm text-muted-foreground">
              AI-powered image generation, editing, and translation
            </p>
          </div>
          <div className="grid max-w-xl grid-cols-3 gap-5 text-center">
            <div className="group rounded-xl border border-border bg-card p-5 shadow-sm transition-all duration-200 hover:-translate-y-0.5 hover:shadow-md hover:border-primary/30">
              <div className="mx-auto mb-3 flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10 text-primary transition-colors group-hover:bg-primary/15">
                <Type className="h-5 w-5" />
              </div>
              <p className="text-sm font-medium text-foreground">Text to Image</p>
              <p className="mt-1.5 text-xs text-muted-foreground">
                Describe what you want to create
              </p>
            </div>
            <div className="group rounded-xl border border-border bg-card p-5 shadow-sm transition-all duration-200 hover:-translate-y-0.5 hover:shadow-md hover:border-primary/30">
              <div className="mx-auto mb-3 flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10 text-primary transition-colors group-hover:bg-primary/15">
                <ImagePlus className="h-5 w-5" />
              </div>
              <p className="text-sm font-medium text-foreground">Image to Image</p>
              <p className="mt-1.5 text-xs text-muted-foreground">
                Upload references and edit
              </p>
            </div>
            <div className="group rounded-xl border border-border bg-card p-5 shadow-sm transition-all duration-200 hover:-translate-y-0.5 hover:shadow-md hover:border-primary/30">
              <div className="mx-auto mb-3 flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10 text-primary transition-colors group-hover:bg-primary/15">
                <Languages className="h-5 w-5" />
              </div>
              <p className="text-sm font-medium text-foreground">Translate</p>
              <p className="mt-1.5 text-xs text-muted-foreground">
                Translate text in images
              </p>
            </div>
          </div>
        </div>
      ) : (
        <ChatArea
          messages={messages}
          generatingStatus={generatingStatus}
        />
      )}

      {/* Generation error */}
      {genError && (
        <div className="px-4">
          <ErrorDisplay
            error={{ kind: "api", message: genError }}
            className="mb-2"
          />
        </div>
      )}

      {/* Input area at the bottom */}
      <div className={cn("border-t border-border/50 bg-background/80 px-4 py-4 shadow-[0_-1px_12px_rgba(0,0,0,0.06)] backdrop-blur-sm")}>
        <div className={cn("mx-auto max-w-3xl space-y-3")}>
          <div className="flex items-center justify-between">
            <ModeSelector
              currentMode={mode}
              onModeChange={setMode}
              disabled={isGenerating}
            />
            <ModelSelector serviceType={mode} />
          </div>

          {/* Image upload area for img2img mode */}
          {mode === "img2img" && (
            <ImageUpload
              images={uploadedImages}
              disabled={isGenerating}
              maxCount={10}
              onAddImages={addImages}
              onRemoveImage={removeImage}
              onReorder={reorderImages}
            />
          )}

          {/* Translate mode: single image upload + language selector */}
          {mode === "translate" && (
            <>
              <ImageUpload
                images={uploadedImages}
                disabled={isGenerating}
                maxCount={1}
                onAddImages={addImages}
                onRemoveImage={removeImage}
                onReorder={reorderImages}
              />
              <LanguageSelector
                sourceLang={sourceLang}
                targetLang={targetLang}
                disabled={isGenerating}
                onSourceLangChange={setSourceLang}
                onTargetLangChange={setTargetLang}
              />
            </>
          )}

          {/* Prompt input: shown for text2img and img2img, hidden for translate */}
          {mode === "translate" ? (
            <button
              onClick={() => generate("")}
              disabled={isGenerating || uploadedImages.length === 0}
              className={cn("w-full rounded-xl bg-primary px-4 py-2.5 text-sm font-medium text-primary-foreground shadow-md shadow-primary/25 transition-all duration-200 hover:bg-primary/90 hover:shadow-lg hover:shadow-primary/30 active:scale-[0.98] disabled:cursor-not-allowed disabled:opacity-50 disabled:shadow-none")}
            >
              {isGenerating ? "Translating..." : "Translate Image"}
            </button>
          ) : (
            <PromptInput
              disabled={false}
              submitting={isGenerating}
              onSubmit={(prompt) => generate(prompt)}
            />
          )}
        </div>
      </div>
    </div>
  );
}
