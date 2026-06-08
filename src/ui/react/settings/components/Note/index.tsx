import type { ComponentChildren } from "preact";
import cs from "./index.module.css";

interface NoteProps {
  type?: "info" | "warning" | "error";
  children: ComponentChildren;
}

export function Note({ type, children }: NoteProps) {
  return <p className={`${cs.note} ${type ? cs[type] : ""}`}>{children}</p>;
}
