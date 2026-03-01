import type { GenerationParams } from "./tauri";

/**
 * Result of parsing a prompt for embedded parameters.
 */
export interface ParsedPrompt {
  /** Prompt with extracted parameter tokens removed */
  cleanPrompt: string;
  /** Parameters extracted from the prompt text */
  extractedParams: GenerationParams;
}

// ── Size aliases ──
// Users can type these keywords instead of numeric dimensions.
// Sorted longest-first at definition time so matching is greedy.

const SIZE_ALIASES: [string, string][] = [
  // Chinese aliases (longer first to avoid partial matches, e.g. "手机壁纸" before "壁纸")
  ["手机壁纸", "1080*1920"],
  ["横屏", "1280*720"],
  ["竖屏", "720*1280"],
  ["方图", "1024*1024"],
  ["方形", "1024*1024"],
  ["壁纸", "1920*1080"],
  ["高清", "1280*1280"],
  ["超宽", "2560*1080"],
  // English aliases
  ["widescreen", "1280*720"],
  ["landscape", "1280*720"],
  ["wallpaper", "1920*1080"],
  ["portrait", "720*1280"],
  ["square", "1024*1024"],
  ["hd", "1280*1280"],
];

/**
 * Parse the prompt string to extract embedded size/resolution parameters.
 *
 * Supports:
 * - Numeric dimensions: "1280*720", "1280x720", "1280×720"
 *   Can appear anywhere in the prompt (no whitespace boundary required).
 * - Size aliases (Chinese/English): "横屏", "竖屏", "landscape", etc.
 *   Chinese aliases can be adjacent to other Chinese characters.
 *   English aliases require word boundaries.
 *
 * The matched token is removed from the returned `cleanPrompt`.
 * The user-specified size is passed through as-is — no validation or snapping.
 * If the API rejects the size, the error propagates naturally.
 *
 * @example
 * parsePrompt("一只猫 1280*720")
 * // => { cleanPrompt: "一只猫", extractedParams: { size: "1280*720" } }
 *
 * parsePrompt("生成横屏图片")
 * // => { cleanPrompt: "生成图片", extractedParams: { size: "1280*720" } }
 *
 * parsePrompt("一只猫1280x720在草地上")
 * // => { cleanPrompt: "一只猫在草地上", extractedParams: { size: "1280*720" } }
 */
export function parsePrompt(prompt: string): ParsedPrompt {
  const extractedParams: GenerationParams = {};
  let cleanPrompt = prompt;

  // 1. Try numeric dimensions first (most explicit, highest priority).
  //    Matches "1280*720", "1280x720", "1280×720" anywhere in the string.
  //    No whitespace boundary required — works with "猫1280*720草地".
  const sizeRegex = /(\d{3,4})\s*[*x×]\s*(\d{3,4})/i;
  const sizeMatch = cleanPrompt.match(sizeRegex);
  if (sizeMatch) {
    extractedParams.size = `${sizeMatch[1]}*${sizeMatch[2]}`;
    cleanPrompt = cleanPrompt.replace(sizeRegex, "").trim();
    cleanPrompt = cleanPrompt.replace(/\s{2,}/g, " ");
    return { cleanPrompt, extractedParams };
  }

  // 2. Try size aliases (longer aliases checked first).
  for (const [alias, size] of SIZE_ALIASES) {
    const isAscii = /^[a-z]+$/i.test(alias);

    let aliasRegex: RegExp;
    if (isAscii) {
      // English aliases need word boundaries to avoid matching substrings.
      // e.g. "hd" should not match inside "thread".
      aliasRegex = new RegExp(`\\b${alias}\\b`, "i");
    } else {
      // Chinese aliases: match the exact characters anywhere in the string.
      // Chinese text typically has no spaces between words, so we cannot
      // rely on word boundaries. Direct substring match is correct here.
      aliasRegex = new RegExp(alias);
    }

    const match = cleanPrompt.match(aliasRegex);
    if (match) {
      extractedParams.size = size;
      cleanPrompt = cleanPrompt.replace(aliasRegex, "").trim();
      cleanPrompt = cleanPrompt.replace(/\s{2,}/g, " ");
      return { cleanPrompt, extractedParams };
    }
  }

  return { cleanPrompt, extractedParams };
}

/**
 * Get all available size aliases for UI display or autocomplete.
 */
export function getSizeAliases(): [string, string][] {
  return [...SIZE_ALIASES];
}
