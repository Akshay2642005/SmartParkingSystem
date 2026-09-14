/**
 * Typography: Zed Sans and Zed Mono, self-hosted.
 *
 * Both are the Iosevka-derived families published by zed-industries/zed-fonts
 * under the SIL Open Font License 1.1 (`src/fonts/LICENSE.md`). They are
 * subset to Latin plus the punctuation, arrows, and box-drawing ranges this UI
 * uses and converted to woff2, which puts each face around 21 kB.
 *
 * Self-hosting rather than `next/font/google` is intentional: the build must
 * not need network access to a font CDN, and the browser must not make a
 * third-party request on first paint.
 *
 * Zed Sans is quasi-proportional, so it keeps a near-monospace rhythm in UI
 * chrome; Zed Mono carries every number, identifier, and timestamp, where
 * tabular figures stop values from reflowing as they update live.
 */
import localFont from "next/font/local";

export const zedSans = localFont({
  src: [
    { path: "../fonts/ZedSans-Regular.woff2", weight: "400", style: "normal" },
    { path: "../fonts/ZedSans-Bold.woff2", weight: "700", style: "normal" },
  ],
  variable: "--font-zed-sans",
  display: "swap",
  fallback: ["ui-sans-serif", "system-ui", "sans-serif"],
});

export const zedMono = localFont({
  src: [
    { path: "../fonts/ZedMono-Regular.woff2", weight: "400", style: "normal" },
    { path: "../fonts/ZedMono-Bold.woff2", weight: "700", style: "normal" },
  ],
  variable: "--font-zed-mono",
  display: "swap",
  fallback: ["ui-monospace", "SFMono-Regular", "monospace"],
});
