import { en } from "./en";
import { es } from "./es";

const catalogs = { en, es } as const;
export type Locale = keyof typeof catalogs;

let locale: Locale = "en";

export function setLocale(next: Locale) {
  locale = next in catalogs ? next : "en";
}

export function getLocale() {
  return locale;
}

function resolve(obj: Record<string, unknown>, path: string): string | undefined {
  const parts = path.split(".");
  let cur: unknown = obj;
  for (const part of parts) {
    if (cur && typeof cur === "object" && part in (cur as object)) {
      cur = (cur as Record<string, unknown>)[part];
    } else {
      return undefined;
    }
  }
  return typeof cur === "string" ? cur : undefined;
}

export function t(key: string): string {
  return resolve(catalogs[locale] as unknown as Record<string, unknown>, key)
    ?? resolve(en as unknown as Record<string, unknown>, key)
    ?? key;
}

export function navLabel(key: keyof typeof en.nav): string {
  return t(`nav.${key}`);
}
