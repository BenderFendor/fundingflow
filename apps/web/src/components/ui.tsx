import React from "react";

export function CropMarks({ className = "" }: { className?: string }) {
  return (
    <>
      <span className={`absolute top-1.5 left-2.5 pointer-events-none font-mono text-[9px] select-none text-current/30 ${className}`}>+</span>
      <span className={`absolute top-1.5 right-2.5 pointer-events-none font-mono text-[9px] select-none text-current/30 ${className}`}>+</span>
      <span className={`absolute bottom-1.5 left-2.5 pointer-events-none font-mono text-[9px] select-none text-current/30 ${className}`}>+</span>
      <span className={`absolute bottom-1.5 right-2.5 pointer-events-none font-mono text-[9px] select-none text-current/30 ${className}`}>+</span>
    </>
  );
}

export function Decimals({ children, className = "" }: { children: React.ReactNode; className?: string }) {
  if (children == null) return null;
  const text = React.Children.toArray(children).join("");
  // Matches prefix/number, optional decimal part, and optional suffix
  const match = text.match(/^([^\d]*\d+)(?:\.(\d+))?([^\d]*)$/);
  if (!match) {
    return <span className={className}>{text}</span>;
  }
  const [, before, decimal, after] = match;
  return (
    <span className={className}>
      {before}
      {decimal ? <span className="text-[0.65em] opacity-80 leading-none">.{decimal}</span> : null}
      {after}
    </span>
  );
}
