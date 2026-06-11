import { api } from "$lib/api";
import { setLocale, type Locale } from "$lib/i18n";
import type { UiPreferences } from "$lib/types";

const defaults: UiPreferences = {
  school_name: "ClassMate",
  theme: "default",
  font_scale: "100",
  accent_color: "#2563eb",
  locale: "en",
};

let prefs: UiPreferences = { ...defaults };
const listeners = new Set<(p: UiPreferences) => void>();

function apply(p: UiPreferences) {
  const root = document.documentElement;
  root.style.setProperty("--primary", p.accent_color);
  root.style.setProperty("--primary-hover", p.accent_color);
  root.style.setProperty("font-size", `${Number(p.font_scale) / 100 * 15}px`);
  root.dataset.theme = p.theme;
  setLocale((p.locale as Locale) || "en");
}

export const preferences = {
  get current() {
    return prefs;
  },

  subscribe(fn: (p: UiPreferences) => void) {
    listeners.add(fn);
    fn(prefs);
    return () => listeners.delete(fn);
  },

  async init() {
    try {
      prefs = await api.getUiPreferences();
    } catch {
      prefs = { ...defaults };
    }
    apply(prefs);
    listeners.forEach((fn) => fn(prefs));
  },

  async save(next: UiPreferences) {
    await api.setUiPreferences(next);
    prefs = next;
    apply(prefs);
    listeners.forEach((fn) => fn(prefs));
  },
};
