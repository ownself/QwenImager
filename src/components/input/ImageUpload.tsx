import { useCallback, useState } from "react";
import {
  DndContext,
  closestCenter,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from "@dnd-kit/core";
import {
  SortableContext,
  useSortable,
  rectSortingStrategy,
  arrayMove,
} from "@dnd-kit/sortable";
import { ImagePlus, X, AlertCircle } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { stat } from "@tauri-apps/plugin-fs";
import { useClipboard } from "@/hooks/useClipboard";
import { convertFileSrc } from "@tauri-apps/api/core";

export interface UploadedImage {
  id: string;
  filePath: string;
  displayOrder: number;
  fileSize: number;
  source: "upload" | "clipboard";
}

interface ImageUploadProps {
  images: UploadedImage[];
  disabled?: boolean;
  maxCount?: number;
  onAddImages: (images: UploadedImage[]) => void;
  onRemoveImage: (imageId: string) => void;
  onReorder: (images: UploadedImage[]) => void;
}

function SortableImageItem({
  image,
  index,
  onRemove,
  disabled,
}: {
  image: UploadedImage;
  index: number;
  onRemove: (id: string) => void;
  disabled?: boolean;
}) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: image.id });

  const style: React.CSSProperties = {
    transform: transform
      ? `translate3d(${transform.x}px, ${transform.y}px, 0)`
      : undefined,
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className="group relative h-20 w-20 shrink-0 overflow-hidden rounded-lg border border-border"
    >
      <img
        src={convertFileSrc(image.filePath)}
        alt={`Image ${index + 1}`}
        className="h-full w-full cursor-grab object-cover active:cursor-grabbing"
        draggable={false}
        {...attributes}
        {...listeners}
      />
      {/* Order badge */}
      <span className="absolute bottom-1 left-1 flex h-5 w-5 items-center justify-center rounded-full bg-black/60 text-[10px] font-bold text-white">
        {index + 1}
      </span>
      {/* Remove button */}
      {!disabled && (
        <button
          onClick={(e) => {
            e.stopPropagation();
            onRemove(image.id);
          }}
          className="absolute right-1 top-1 flex h-5 w-5 items-center justify-center rounded-full bg-black/60 text-white opacity-0 transition-opacity group-hover:opacity-100"
          title="Remove"
        >
          <X className="h-3 w-3" />
        </button>
      )}
    </div>
  );
}

const MAX_FILE_SIZE = 10 * 1024 * 1024; // 10MB

export function ImageUpload({
  images,
  disabled = false,
  maxCount = 10,
  onAddImages,
  onRemoveImage,
  onReorder,
}: ImageUploadProps) {
  const [sizeError, setSizeError] = useState<string | null>(null);

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: { distance: 5 },
    })
  );

  // Clipboard paste support
  useClipboard({
    enabled: !disabled && images.length < maxCount,
    onPasteImage: (filePath) => {
      const newImage: UploadedImage = {
        id: crypto.randomUUID(),
        filePath,
        displayOrder: images.length,
        fileSize: 0,
        source: "clipboard",
      };
      onAddImages([newImage]);
    },
  });

  const handleAddClick = useCallback(async () => {
    if (disabled || images.length >= maxCount) return;
    setSizeError(null);

    const selected = await open({
      multiple: true,
      filters: [
        {
          name: "Images",
          extensions: ["png", "jpg", "jpeg", "webp", "gif"],
        },
      ],
    });

    if (!selected) return;

    const paths = Array.isArray(selected) ? selected : [selected];
    const remaining = maxCount - images.length;
    const toAdd = paths.slice(0, remaining);

    // Check file sizes
    const validImages: UploadedImage[] = [];
    const rejected: string[] = [];

    for (let i = 0; i < toAdd.length; i++) {
      const path = toAdd[i];
      try {
        const fileStat = await stat(path);
        if (fileStat.size > MAX_FILE_SIZE) {
          rejected.push(path.split(/[\\/]/).pop() ?? path);
          continue;
        }
        validImages.push({
          id: crypto.randomUUID(),
          filePath: path,
          displayOrder: images.length + validImages.length,
          fileSize: fileStat.size,
          source: "upload" as const,
        });
      } catch {
        // If stat fails, add anyway with size 0
        validImages.push({
          id: crypto.randomUUID(),
          filePath: path,
          displayOrder: images.length + validImages.length,
          fileSize: 0,
          source: "upload" as const,
        });
      }
    }

    if (rejected.length > 0) {
      setSizeError(
        `Skipped ${rejected.length} file(s) exceeding 10MB: ${rejected.join(", ")}`
      );
      setTimeout(() => setSizeError(null), 5000);
    }

    if (validImages.length > 0) {
      onAddImages(validImages);
    }
  }, [disabled, images.length, maxCount, onAddImages]);

  const handleDragEnd = useCallback(
    (event: DragEndEvent) => {
      const { active, over } = event;
      if (!over || active.id === over.id) return;

      const oldIndex = images.findIndex((img) => img.id === active.id);
      const newIndex = images.findIndex((img) => img.id === over.id);

      if (oldIndex === -1 || newIndex === -1) return;

      const reordered = arrayMove(images, oldIndex, newIndex).map(
        (img, i) => ({
          ...img,
          displayOrder: i,
        })
      );
      onReorder(reordered);
    },
    [images, onReorder]
  );

  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2">
        <DndContext
          sensors={sensors}
          collisionDetection={closestCenter}
          onDragEnd={handleDragEnd}
        >
          <SortableContext
            items={images.map((img) => img.id)}
            strategy={rectSortingStrategy}
          >
            <div className="flex flex-wrap gap-2">
              {images.map((image, index) => (
                <SortableImageItem
                  key={image.id}
                  image={image}
                  index={index}
                  onRemove={onRemoveImage}
                  disabled={disabled}
                />
              ))}
            </div>
          </SortableContext>
        </DndContext>

        {/* Add button */}
        {images.length < maxCount && !disabled && (
          <button
            onClick={handleAddClick}
            className="flex h-20 w-20 shrink-0 items-center justify-center rounded-lg border-2 border-dashed border-border text-muted-foreground transition-colors hover:border-primary hover:text-primary"
            title="Add images (or Ctrl+V to paste)"
          >
            <ImagePlus className="h-6 w-6" />
          </button>
        )}
      </div>

      {images.length > 0 && (
        <p className="text-xs text-muted-foreground">
          {images.length}/{maxCount} images. Drag to reorder, Ctrl+V to paste.
        </p>
      )}

      {sizeError && (
        <div className="flex items-center gap-2 rounded-md border border-destructive/50 bg-destructive/10 px-3 py-2 text-xs text-destructive">
          <AlertCircle className="h-3.5 w-3.5 shrink-0" />
          <span>{sizeError}</span>
        </div>
      )}
    </div>
  );
}
