export default function Loading() {
  return (
    <main id="main-content" className="mx-auto min-h-screen w-full max-w-[1440px] px-4 py-5 sm:px-6 lg:px-8" aria-busy="true">
      <div className="grid animate-pulse border border-white/10 lg:grid-cols-12">
        <div className="min-h-[620px] bg-[#f2efe7]/90 lg:col-span-8" />
        <div className="min-h-[620px] border-l border-white/10 bg-[#101318] lg:col-span-4" />
      </div>
      <div className="mt-5 grid sm:grid-cols-2 lg:grid-cols-4">
        {[0, 1, 2, 3].map((item) => (
          <div key={item} className="h-48 border border-white/10 bg-white/[0.035]" />
        ))}
      </div>
    </main>
  );
}
