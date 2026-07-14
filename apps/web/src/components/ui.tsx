import React from "react";

export function CropMarks({ className = "" }: { className?: string }) {
  const markClass = `absolute h-3 w-3 pointer-events-none select-none opacity-45 ${className}`;
  return (
    <>
      <span className={`${markClass} left-3 top-3 border-l border-t border-current`} />
      <span className={`${markClass} right-3 top-3 border-r border-t border-current`} />
      <span className={`${markClass} bottom-3 left-3 border-b border-l border-current`} />
      <span className={`${markClass} bottom-3 right-3 border-b border-r border-current`} />
    </>
  );
}

export function Decimals({
  children,
  className = "",
}: {
  children: React.ReactNode;
  className?: string;
}) {
  if (children == null) return null;
  const text = React.Children.toArray(children).join("");
  const match = text.match(/^([^\d]*\d+)(?:\.(\d+))?([^\d]*)$/);
  if (!match) return <span className={className}>{text}</span>;

  const [, before, decimal, after] = match;
  return (
    <span className={className}>
      {before}
      {decimal ? <span className="text-[0.62em] leading-none opacity-72">.{decimal}</span> : null}
      {after}
    </span>
  );
}

export function DataBadge({
  children,
  tone = "neutral",
}: {
  children: React.ReactNode;
  tone?: "neutral" | "acid" | "paper" | "blue";
}) {
  const tones = {
    neutral: "border-white/10 bg-white/[0.045] text-slate-400",
    acid: "border-[#dfff00]/25 bg-[#dfff00]/10 text-[#dfff00]",
    paper: "border-black/10 bg-black/[0.055] text-black/60",
    blue: "border-white/20 bg-white/10 text-white/80",
  };

  return (
    <span className={`ff-micro inline-flex items-center border px-2.5 py-1 text-[9px] font-black ${tones[tone]}`}>
      {children}
    </span>
  );
}

export function SectionHeading({
  eyebrow,
  title,
  description,
  meta,
  invert = false,
}: {
  eyebrow: string;
  title: string;
  description?: string;
  meta?: React.ReactNode;
  invert?: boolean;
}) {
  return (
    <div className="flex flex-col gap-5 border-b border-current/10 px-6 py-6 sm:px-8 sm:py-7 md:flex-row md:items-end md:justify-between">
      <div className="max-w-3xl">
        <p className={`ff-kicker ${invert ? "text-black/55" : "text-[#dfff00]"}`}>{eyebrow}</p>
        <h2 className={`ff-sans-wide mt-4 text-3xl font-black leading-[0.94] tracking-tight sm:text-4xl ${invert ? "text-black" : "text-white"}`}>
          {title}
        </h2>
        {description ? (
          <p className={`mt-3 max-w-2xl text-sm leading-6 ${invert ? "text-black/60" : "text-slate-400"}`}>
            {description}
          </p>
        ) : null}
      </div>
      {meta ? <div className="shrink-0">{meta}</div> : null}
    </div>
  );
}

export function MetricTile({
  label,
  value,
  meta,
  accent = "acid",
}: {
  label: string;
  value: React.ReactNode;
  meta?: React.ReactNode;
  accent?: "acid" | "yellow" | "cyan" | "orange" | "lavender" | "white";
}) {
  const accents = {
    acid: "text-[#dfff00]",
    yellow: "text-[#f8df1d]",
    cyan: "text-[#d0f1ed]",
    orange: "text-[#ee744f]",
    lavender: "text-[#aa9fda]",
    white: "text-white",
  };

  return (
    <div className="relative flex min-h-44 flex-col justify-between overflow-hidden border-r border-b border-white/10 bg-black/30 p-5 sm:p-6">
      <CropMarks className="text-white/20" />
      <p className="ff-micro relative z-10 text-[9px] font-black leading-4 text-slate-500">{label}</p>
      <div className="relative z-10 mt-7">
        <p className={`ff-pixel text-3xl font-black leading-none tracking-[-0.08em] sm:text-4xl ${accents[accent]}`}>
          {value}
        </p>
        {meta ? <div className="mt-4 font-mono text-[9px] uppercase leading-4 tracking-wider text-slate-600">{meta}</div> : null}
      </div>
    </div>
  );
}

export function EmptyState({
  title,
  description,
  action,
}: {
  title: string;
  description: string;
  action?: React.ReactNode;
}) {
  return (
    <div className="relative overflow-hidden border border-dashed border-white/20 bg-white/[0.025] p-8 sm:p-10">
      <CropMarks className="text-white/20" />
      <p className="ff-kicker text-slate-500">Missing data</p>
      <h3 className="ff-sans-wide mt-4 text-2xl font-black text-white">{title}</h3>
      <p className="mt-3 max-w-2xl text-sm leading-6 text-slate-500">{description}</p>
      {action ? <div className="mt-6">{action}</div> : null}
    </div>
  );
}

export function DocumentHero({
  eyebrow,
  title,
  description,
  index,
}: {
  eyebrow: string;
  title: string;
  description: string;
  index: string;
}) {
  return (
    <section className="ff-paper ff-enter relative overflow-hidden p-7 sm:p-10 lg:p-12">
      <CropMarks className="text-black/40" />
      <div className="absolute -bottom-10 right-0 select-none font-mono text-[9rem] font-black leading-none tracking-[-0.12em] text-black/[0.035] sm:text-[13rem]">
        {index}
      </div>
      <div className="relative z-10 max-w-3xl">
        <p className="ff-kicker text-black/55">{eyebrow}</p>
        <h1 className="ff-sans-wide mt-7 text-5xl font-black leading-[0.86] tracking-[-0.07em] text-black sm:text-7xl">
          {title}
        </h1>
        <p className="mt-7 max-w-2xl text-base font-medium leading-7 text-black/62 sm:text-lg">
          {description}
        </p>
      </div>
    </section>
  );
}

export function DocumentSection({
  index,
  title,
  children,
  tone = "terminal",
}: {
  index: string;
  title: string;
  children: React.ReactNode;
  tone?: "terminal" | "blueprint" | "paper";
}) {
  const toneClass = tone === "blueprint" ? "ff-blueprint" : tone === "paper" ? "ff-paper" : "ff-terminal";
  const invert = tone === "paper";

  return (
    <section className={`${toneClass} relative overflow-hidden`}>
      <CropMarks className={invert ? "text-black/30" : "text-white/20"} />
      <div className={`grid gap-6 border-b px-6 py-6 sm:grid-cols-[6rem_1fr] sm:px-8 ${invert ? "border-black/10" : "border-white/10"}`}>
        <span className={`ff-pixel text-3xl font-black ${invert ? "text-black/20" : "text-white/20"}`}>{index}</span>
        <h2 className={`ff-sans-wide text-2xl font-black sm:text-3xl ${invert ? "text-black" : "text-white"}`}>{title}</h2>
      </div>
      <div className={`px-6 py-7 text-sm leading-7 sm:px-8 sm:py-8 ${invert ? "text-black/65" : "text-slate-400"}`}>
        {children}
      </div>
    </section>
  );
}
