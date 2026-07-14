import Link from "next/link";
import { SiteHeader } from "@/components/site-header";
import { CropMarks } from "@/components/ui";

export default function NotFound() {
  return (
    <div className="min-h-screen text-white">
      <SiteHeader />
      <main id="main-content" className="mx-auto grid min-h-[72vh] w-full max-w-5xl place-items-center px-4 py-12 sm:px-6">
        <section className="ff-paper relative w-full overflow-hidden p-8 text-black sm:p-12">
          <CropMarks className="text-black/35" />
          <p className="ff-kicker text-black/50">Record not found</p>
          <p className="ff-pixel mt-8 text-8xl font-black leading-none tracking-[-0.12em] text-black/10 sm:text-9xl">404</p>
          <h1 className="ff-sans-wide mt-5 max-w-2xl text-4xl font-black leading-[0.92] sm:text-6xl">
            This route is outside the current record set.
          </h1>
          <p className="mt-6 max-w-xl text-base leading-7 text-black/60">
            Return to search and look up an organization, person, geography, or public identifier.
          </p>
          <Link
            href="/"
            className="ff-focus-ring mt-9 inline-flex border-2 border-black bg-black px-5 py-3 font-mono text-[10px] font-black uppercase tracking-wider text-white hover:bg-[#dfff00] hover:text-black"
          >
            Return to search
          </Link>
        </section>
      </main>
    </div>
  );
}
