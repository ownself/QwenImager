import { useConversationStore } from "@/stores/conversationStore";
import { useGeneration } from "@/hooks/useGeneration";
import { ChatArea } from "@/components/chat/ChatArea";
import { PromptInput } from "@/components/input/PromptInput";
import { ModeSelector } from "@/components/input/ModeSelector";
import { ImageUpload } from "@/components/input/ImageUpload";
import { LanguageSelector } from "@/components/input/LanguageSelector";
import { ErrorDisplay } from "@/components/common/ErrorDisplay";

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
        <div className="flex flex-1 flex-col items-center justify-center gap-4 p-8">
          <div className="text-center">
            <h1 className="text-2xl font-semibold text-foreground">
              QwenImager
            </h1>
            <p className="mt-2 text-sm text-muted-foreground">
              AI-powered image generation, editing, and translation
            </p>
          </div>
          <div className="grid max-w-lg grid-cols-3 gap-3 text-center">
            <div className="rounded-lg border border-border p-3">
              <p className="text-xs font-medium text-foreground">Text to Image</p>
              <p className="mt-1 text-[10px] text-muted-foreground">
                Describe what you want to create
              </p>
            </div>
            <div className="rounded-lg border border-border p-3">
              <p className="text-xs font-medium text-foreground">Image to Image</p>
              <p className="mt-1 text-[10px] text-muted-foreground">
                Upload references and edit
              </p>
            </div>
            <div className="rounded-lg border border-border p-3">
              <p className="text-xs font-medium text-foreground">Translate</p>
              <p className="mt-1 text-[10px] text-muted-foreground">
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
      <div className="border-t border-border px-4 py-3">
        <div className="mx-auto max-w-3xl space-y-2">
          <div className="flex items-center justify-between">
            <ModeSelector
              currentMode={mode}
              onModeChange={setMode}
              disabled={isGenerating}
            />
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
              className="w-full rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90 disabled:cursor-not-allowed disabled:opacity-50"
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
