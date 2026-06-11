import { api } from "$lib/api";
import type { TenancyContext } from "$lib/types";

let context: TenancyContext | null = null;
const listeners = new Set<(ctx: TenancyContext | null) => void>();

export const tenancy = {
  get context() {
    return context;
  },

  subscribe(fn: (ctx: TenancyContext | null) => void) {
    listeners.add(fn);
    fn(context);
    return () => listeners.delete(fn);
  },

  async init() {
    try {
      context = await api.getTenancyContext();
    } catch {
      context = null;
    }
    listeners.forEach((fn) => fn(context));
    return context;
  },

  async setActiveSchool(schoolId: string) {
    context = await api.setActiveSchool(schoolId);
    listeners.forEach((fn) => fn(context));
    return context;
  },

  clear() {
    context = null;
    listeners.forEach((fn) => fn(context));
  },
};
