import type { ClassValue } from "clsx";

import { clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export const isDesktop =
  typeof window !== "undefined" &&
  (window.IS_DESKTOP ?? ((window as any).__TAURI__ !== undefined || (window as any).__TAURI_INTERNALS__ !== undefined));
