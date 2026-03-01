import { create } from "zustand";
import {
  invokeCommand,
  createChannel,
  type MessageDetail,
  type GenerationEvent,
  type GenerationParams,
  type EditResult,
  type ConversationSummary,
} from "@/lib/tauri";
import { parsePrompt } from "@/lib/promptParser";
import { useConfigStore } from "@/stores/configStore";
import type { UploadedImage } from "@/components/input/ImageUpload";

export type GeneratingStatus =
  | "submitted"
  | "polling"
  | "succeeded"
  | "failed"
  | null;

export type Mode = "text2img" | "img2img" | "translate";

interface ConversationState {
  // Conversation list
  conversations: ConversationSummary[];

  // Current conversation
  currentConversationId: string | null;
  messages: MessageDetail[];

  // Generation status
  generatingStatus: GeneratingStatus;
  generatingTaskId: string | null;
  generatingError: string | null;

  // Mode
  mode: Mode;

  // Selected model (null = use default)
  selectedModel: string | null;

  // Uploaded images (img2img / translate)
  uploadedImages: UploadedImage[];

  // Translation language settings
  sourceLang: string;
  targetLang: string;

  // Actions
  setMode: (mode: Mode) => void;
  setSelectedModel: (name: string | null) => void;
  createConversation: () => Promise<string>;
  sendPrompt: (prompt: string, params?: GenerationParams) => Promise<void>;
  addMessage: (message: MessageDetail) => void;
  setCurrentConversation: (id: string | null) => void;
  loadMessages: (conversationId: string) => Promise<void>;
  resetGenerating: () => void;

  // Image upload actions
  addImages: (images: UploadedImage[]) => void;
  removeImage: (imageId: string) => void;
  reorderImages: (images: UploadedImage[]) => void;
  clearImages: () => void;

  // Conversation list actions
  loadConversations: () => Promise<void>;
  switchConversation: (id: string) => Promise<void>;
  deleteConversation: (id: string) => Promise<void>;
  newConversation: () => void;

  // Translation actions
  setSourceLang: (lang: string) => void;
  setTargetLang: (lang: string) => void;
}

export const useConversationStore = create<ConversationState>((set, get) => ({
  conversations: [],
  currentConversationId: null,
  messages: [],
  generatingStatus: null,
  generatingTaskId: null,
  generatingError: null,
  mode: "text2img",
  selectedModel: null,
  uploadedImages: [],
  sourceLang: "zh",
  targetLang: "en",

  setMode: (mode) => set({ mode, selectedModel: null }),
  setSelectedModel: (name) => set({ selectedModel: name }),

  createConversation: async () => {
    const id = await invokeCommand<string>("create_conversation", {
      title: "New Conversation",
    });
    set({
      currentConversationId: id,
      messages: [],
      generatingStatus: null,
      generatingTaskId: null,
      generatingError: null,
    });
    return id;
  },

  sendPrompt: async (prompt, params) => {
    const state = get();
    let conversationId = state.currentConversationId;

    // Auto-create conversation if none exists
    if (!conversationId) {
      conversationId = await get().createConversation();
    }

    set({
      generatingStatus: "submitted",
      generatingError: null,
    });

    try {
      const mode = state.mode;

      if (mode === "text2img") {
        // Check if the selected model supports the size parameter
        const selectedModelInfo = useConfigStore
          .getState()
          .availableModels.find((m) => m.name === state.selectedModel);
        const supportsSize = selectedModelInfo?.supportsSize ?? false;

        // Only extract size from prompt when the model supports it
        let cleanPrompt = prompt;
        let mergedParams: GenerationParams = { ...(params ?? {}) };
        if (supportsSize) {
          const parsed = parsePrompt(prompt);
          cleanPrompt = parsed.cleanPrompt;
          mergedParams = { ...parsed.extractedParams, ...mergedParams };
        }


        // Create a channel for generation events
        const channel = createChannel<GenerationEvent>((event) => {
          switch (event.event) {
            case "submitted":
              set({
                generatingStatus: "submitted",
                generatingTaskId: event.data.taskId,
              });
              break;
            case "polling":
              set({ generatingStatus: "polling" });
              break;
            case "succeeded":
              get().loadMessages(conversationId!);
              get().loadConversations();
              set({
                generatingStatus: "succeeded",
                generatingTaskId: null,
              });
              setTimeout(() => {
                set({ generatingStatus: null });
              }, 1500);
              break;
            case "failed":
              set({
                generatingStatus: "failed",
                generatingError: event.data.error,
                generatingTaskId: null,
              });
              break;
          }
        });

        // Add user message to UI immediately (optimistic)
        // Show original prompt so user sees what they typed
        const userMessage: MessageDetail = {
          id: crypto.randomUUID(),
          role: "user",
          textContent: prompt,
          mode: "text2img",
          attachments: [],
          results: [],
          createdAt: Math.floor(Date.now() / 1000),
        };
        set((s) => ({ messages: [...s.messages, userMessage] }));

        await invokeCommand("generate_image", {
          conversationId,
          prompt: cleanPrompt,
          modelName: state.selectedModel ?? null,
          params: mergedParams,
          onEvent: channel,
        });
      } else if (mode === "img2img") {
        const uploadedImages = state.uploadedImages;

        if (uploadedImages.length === 0) {
          set({
            generatingStatus: "failed",
            generatingError: "Please add at least one reference image",
          });
          return;
        }

        // Check if the selected model supports the size parameter
        const img2imgModelInfo = useConfigStore
          .getState()
          .availableModels.find((m) => m.name === state.selectedModel);
        const img2imgSupportsSize = img2imgModelInfo?.supportsSize ?? false;

        let img2imgCleanPrompt = prompt;
        let img2imgParams: GenerationParams = { ...(params ?? {}) };
        if (img2imgSupportsSize) {
          const parsed = parsePrompt(prompt);
          img2imgCleanPrompt = parsed.cleanPrompt;
          img2imgParams = { ...parsed.extractedParams, ...img2imgParams };
        }

        // Add optimistic user message with attachment info
        // Show original prompt so user sees what they typed
        const userMessage: MessageDetail = {
          id: crypto.randomUUID(),
          role: "user",
          textContent: prompt,
          mode: "img2img",
          attachments: uploadedImages.map((img, i) => ({
            id: img.id,
            filePath: img.filePath,
            displayOrder: i,
            fileSize: img.fileSize,
            mimeType: "image/png",
            source: img.source,
          })),
          results: [],
          createdAt: Math.floor(Date.now() / 1000),
        };
        set((s) => ({ messages: [...s.messages, userMessage] }));

        const imagePaths = uploadedImages.map((img) => img.filePath);

        await invokeCommand<EditResult>("edit_image", {
          conversationId,
          imagePaths,
          prompt: img2imgCleanPrompt,
          modelName: state.selectedModel ?? null,
          params: img2imgParams,
        });

        // Reload messages from DB to get full details
        await get().loadMessages(conversationId!);
        await get().loadConversations();

        set({
          generatingStatus: "succeeded",
          generatingTaskId: null,
          uploadedImages: [], // Clear after successful submission
        });
        setTimeout(() => {
          set({ generatingStatus: null });
        }, 1500);
      } else if (mode === "translate") {
        const uploadedImages = state.uploadedImages;
        const { sourceLang, targetLang } = state;

        if (uploadedImages.length === 0) {
          set({
            generatingStatus: "failed",
            generatingError: "Please add an image to translate",
          });
          return;
        }

        const imagePath = uploadedImages[0].filePath;

        // Add optimistic user message
        const userMessage: MessageDetail = {
          id: crypto.randomUUID(),
          role: "user",
          textContent: `${sourceLang} → ${targetLang}`,
          mode: "translate",
          attachments: [
            {
              id: uploadedImages[0].id,
              filePath: imagePath,
              displayOrder: 0,
              fileSize: uploadedImages[0].fileSize,
              mimeType: "image/png",
              source: uploadedImages[0].source,
            },
          ],
          results: [],
          createdAt: Math.floor(Date.now() / 1000),
        };
        set((s) => ({ messages: [...s.messages, userMessage] }));

        // Create a channel for translation events (async like text2img)
        const channel = createChannel<GenerationEvent>((event) => {
          switch (event.event) {
            case "submitted":
              set({
                generatingStatus: "submitted",
                generatingTaskId: event.data.taskId,
              });
              break;
            case "polling":
              set({ generatingStatus: "polling" });
              break;
            case "succeeded":
              get().loadMessages(conversationId!);
              get().loadConversations();
              set({
                generatingStatus: "succeeded",
                generatingTaskId: null,
                uploadedImages: [], // Clear after successful submission
              });
              setTimeout(() => {
                set({ generatingStatus: null });
              }, 1500);
              break;
            case "failed":
              set({
                generatingStatus: "failed",
                generatingError: event.data.error,
                generatingTaskId: null,
              });
              break;
          }
        });

        await invokeCommand("translate_image", {
          conversationId,
          imagePath,
          sourceLang,
          targetLang,
          modelName: state.selectedModel ?? null,
          onEvent: channel,
        });
      }
    } catch (err: unknown) {
      const currentStatus = get().generatingStatus;
      if (currentStatus !== "failed") {
        set({
          generatingStatus: "failed",
          generatingError:
            typeof err === "object" && err !== null && "message" in err
              ? (err as { message: string }).message
              : "An unknown error occurred",
          generatingTaskId: null,
        });
      }
    }
  },

  addMessage: (message) => {
    set((s) => ({ messages: [...s.messages, message] }));
  },

  setCurrentConversation: (id) => {
    set({
      currentConversationId: id,
      messages: [],
      generatingStatus: null,
      generatingTaskId: null,
      generatingError: null,
      uploadedImages: [],
    });
    if (id) {
      get().loadMessages(id);
    }
  },

  loadMessages: async (conversationId) => {
    try {
      const messages = await invokeCommand<MessageDetail[]>(
        "get_conversation_messages",
        { conversationId }
      );
      set({ messages });
    } catch {
      // Silently fail
    }
  },

  resetGenerating: () => {
    set({
      generatingStatus: null,
      generatingTaskId: null,
      generatingError: null,
    });
  },

  // Image upload actions
  addImages: (newImages) => {
    set((s) => ({
      uploadedImages: [...s.uploadedImages, ...newImages],
    }));
  },

  removeImage: (imageId) => {
    set((s) => ({
      uploadedImages: s.uploadedImages
        .filter((img) => img.id !== imageId)
        .map((img, i) => ({ ...img, displayOrder: i })),
    }));
  },

  reorderImages: (images) => {
    set({ uploadedImages: images });
  },

  clearImages: () => {
    set({ uploadedImages: [] });
  },

  // Conversation list actions
  loadConversations: async () => {
    try {
      const conversations = await invokeCommand<ConversationSummary[]>(
        "get_conversations",
        { limit: 50, offset: 0 }
      );
      set({ conversations });
    } catch {
      // Silently fail
    }
  },

  switchConversation: async (id) => {
    set({
      currentConversationId: id,
      messages: [],
      generatingStatus: null,
      generatingTaskId: null,
      generatingError: null,
      uploadedImages: [],
    });
    try {
      const messages = await invokeCommand<MessageDetail[]>(
        "get_conversation_messages",
        { conversationId: id }
      );
      set({ messages });
    } catch {
      // Silently fail
    }
  },

  deleteConversation: async (id) => {
    try {
      await invokeCommand("delete_conversation", { conversationId: id });
      const state = get();
      // If deleting the current conversation, clear it
      if (state.currentConversationId === id) {
        set({
          currentConversationId: null,
          messages: [],
          generatingStatus: null,
          generatingTaskId: null,
          generatingError: null,
          uploadedImages: [],
        });
      }
      // Refresh the conversation list
      await get().loadConversations();
    } catch {
      // Silently fail
    }
  },

  newConversation: () => {
    set({
      currentConversationId: null,
      messages: [],
      generatingStatus: null,
      generatingTaskId: null,
      generatingError: null,
      uploadedImages: [],
    });
  },

  // Translation actions
  setSourceLang: (lang) => set({ sourceLang: lang }),
  setTargetLang: (lang) => set({ targetLang: lang }),
}));
