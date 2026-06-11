import { api } from "$lib/api";
import type { User } from "$lib/types";

let currentUser: User | null = null;
const listeners = new Set<(user: User | null) => void>();

export const auth = {
  get user() {
    return currentUser;
  },

  subscribe(fn: (user: User | null) => void) {
    listeners.add(fn);
    fn(currentUser);
    return () => listeners.delete(fn);
  },

  async init() {
    currentUser = await api.getSession();
    listeners.forEach((fn) => fn(currentUser));
    if (currentUser) {
      const { tenancy } = await import("$lib/stores/tenancy");
      await tenancy.init();
    }
    return currentUser;
  },

  async login(email: string, password: string) {
    currentUser = await api.login({ email, password });
    listeners.forEach((fn) => fn(currentUser));
    const { tenancy } = await import("$lib/stores/tenancy");
    await tenancy.init();
    return currentUser;
  },

  async logout() {
    await api.logout();
    currentUser = null;
    listeners.forEach((fn) => fn(currentUser));
    const { tenancy } = await import("$lib/stores/tenancy");
    tenancy.clear();
  },
};
