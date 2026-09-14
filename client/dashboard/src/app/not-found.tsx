/**
 * 404 page — using the 8bitcn block exactly as advertised.
 *
 * The pixel geometry fits here because a 404 is out-of-band: it is not the
 * parking board, so the retro motif signals "you have left the console" without
 * redefining what the retro language means inside the board itself.
 */
import NotFound1 from "@/components/ui/8bit/blocks/not-found1";
import type { Metadata } from "next";

export const metadata: Metadata = { title: "Not Found" };

export default function NotFound() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-background">
      <NotFound1
        title="Bay not found"
        description="This route does not exist. Return to the parking board."
        cta="Back to dashboard"
        href="/"
        imageSrc=""
      />
    </div>
  );
}
