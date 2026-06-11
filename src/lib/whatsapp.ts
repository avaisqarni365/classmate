export function openWhatsApp(url: string) {
  window.open(url, "_blank", "noopener,noreferrer");
}

export async function copyText(text: string) {
  await navigator.clipboard.writeText(text);
}

export function waMeUrl(phoneDigits: string, message: string): string {
  const encoded = encodeURIComponent(message);
  return `https://wa.me/${phoneDigits}?text=${encoded}`;
}
