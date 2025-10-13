import type { ResourceText as IResourceText } from "@seelen-ui/lib/types";
import { useTranslation } from "react-i18next";

interface Props {
  className?: string;
  text?: IResourceText;
  noFallback?: boolean;
}

export function ResourceText({ text, className, noFallback }: Props) {
  const {
    i18n: { language },
  } = useTranslation();

  if (!text) {
    if (noFallback) {
      return null;
    }
    return <span className={className}>null!?</span>;
  }

  if (typeof text === "string") {
    return <span className={className}>{text}</span>;
  }

  const text2 = text[language] || text["en"];
  if (!text2) {
    if (noFallback) {
      return null;
    }
    return <span className={className}>null!?</span>;
  }

  return <span className={className}>{text2}</span>;
}
