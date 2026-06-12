import type {
  CourseMaterial,
  CreateCaptureSessionInput,
  CaptureSession,
  TextbookMaterialContent,
  SpeakNoteContent,
  HandwritingContent,
} from "$lib/types";

export function parseTextbookContent(content: string): TextbookMaterialContent | null {
  try {
    const parsed = JSON.parse(content) as TextbookMaterialContent;
    if (parsed.provider === "openstax" && parsed.read_url) {
      return parsed;
    }
  } catch {
    return null;
  }
  return null;
}

export function materialReadUrl(material: CourseMaterial): string | null {
  if (material.kind === "link") {
    return material.content;
  }
  if (material.kind === "textbook") {
    return parseTextbookContent(material.content)?.read_url ?? null;
  }
  return null;
}

export function materialPdfUrl(material: CourseMaterial): string | null {
  if (material.kind !== "textbook") {
    return null;
  }
  return parseTextbookContent(material.content)?.pdf_url ?? null;
}

export function materialNotes(material: CourseMaterial): string {
  if (material.kind === "textbook") {
    return parseTextbookContent(material.content)?.notes?.trim() ?? "";
  }
  if (material.kind === "speak_note") {
    return parseSpeakNoteContent(material.content)?.body?.trim() ?? "";
  }
  return material.content;
}

export function parseSpeakNoteContent(content: string): SpeakNoteContent | null {
  try {
    const parsed = JSON.parse(content) as SpeakNoteContent;
    if (parsed.body) return parsed;
  } catch {
    return null;
  }
  return null;
}

export function parseHandwritingContent(content: string): HandwritingContent | null {
  try {
    return JSON.parse(content) as HandwritingContent;
  } catch {
    return null;
  }
}

export function buildSpeakNoteContent(input: {
  body: string;
  outline?: string;
  transcript?: string;
}): string {
  return JSON.stringify({
    body: input.body.trim(),
    outline: input.outline?.trim() || undefined,
    transcript: input.transcript?.trim() || undefined,
  });
}

export function buildOpenStaxMaterialContent(
  book: {
    slug: string;
    title: string;
    subjects: string[];
    read_url: string;
    pdf_url?: string | null;
  },
  notes = "",
): string {
  return JSON.stringify({
    provider: "openstax",
    book_slug: book.slug,
    book_title: book.title,
    subjects: book.subjects,
    read_url: book.read_url,
    pdf_url: book.pdf_url ?? null,
    notes: notes.trim(),
  });
}
