import type { ResourceText as IResourceText } from "@seelen-ui/lib/types";
import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { useTranslation } from "react-i18next";
import { unified } from "unified";
import remarkParse from "remark-parse";
import remarkGfm from "remark-gfm";
import remarkRehype from "remark-rehype";
import rehypeStringify from "rehype-stringify";
import { useEffect } from "preact/hooks";
import { useSignal } from "@preact/signals";

interface Props {
  className?: string;
  text?: IResourceText;
}

export function ResourceText({ text, className }: Props) {
  const {
    i18n: { language },
  } = useTranslation();

  if (!text) {
    return null;
  }
  if (typeof text === "string") {
    return <span className={className}>{text}</span>;
  }

  const text2 = text[language] || text["en"];
  if (!text2) {
    return null;
  }
  return <span className={className}>{text2}</span>;
}

interface MarkdownViewerProps {
  text: IResourceText;
}

export function ResourceTextAsMarkdown({ text }: MarkdownViewerProps) {
  const html = useSignal("");
  const {
    i18n: { language },
  } = useTranslation();

  useEffect(() => {
    let input = typeof text === "string" ? text : text[language] || text["en"];
    if (!input) {
      html.value = "";
      return;
    }
    safeMarkdownToHtml(input).then((content) => (html.value = content));
  }, [text, language]);

  if (!html.value) {
    return null;
  }

  return (
    <div
      className="richText"
      dangerouslySetInnerHTML={{ __html: html.value }}
      onClick={(e) => {
        const target = e.target as HTMLElement;
        const anchor = target.closest("a");
        if (anchor?.href) {
          // force links on markdown being opened on browser
          e.preventDefault();
          invoke(SeelenCommand.OpenFile, { path: anchor.href });
        }
      }}
    />
  );
}

/** this can be used on untrusted markdown, ex user inputs */
export async function safeMarkdownToHtml(markdown: string): Promise<string> {
  const result = await unified()
    .use(remarkParse)
    .use(remarkGfm) // enable GitHub Flavored Markdown
    .use(remarkRehype) // allow conversion of markdown to html
    .use(rehypeStringify) // convert html to string
    .process(markdown);
  return result.toString();
}
