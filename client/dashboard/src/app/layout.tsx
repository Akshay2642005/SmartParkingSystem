import type { Metadata, Viewport } from "next";
import type React from "react";
import "./globals.css";
import { zedMono, zedSans } from "./fonts";
import { Providers } from "@/components/providers";

export const metadata: Metadata = {
  title: {
    default: "Smart Parking — Dashboard",
    template: "%s — Smart Parking",
  },
  description: "Real-time parking availability dashboard.",
};

export const viewport: Viewport = {
  themeColor: [
    { media: "(prefers-color-scheme: light)", color: "#f8f8fa" },
    { media: "(prefers-color-scheme: dark)", color: "#1e1e24" },
  ],
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html
      lang="en"
      suppressHydrationWarning
      className={`${zedSans.variable} ${zedMono.variable} h-full`}
    >
      <body className="min-h-full flex flex-col antialiased">
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
