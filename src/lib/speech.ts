type SpeechRecognitionCtor = new () => SpeechRecognition;

function getSpeechRecognition(): SpeechRecognitionCtor | null {
  const w = window as Window & {
    SpeechRecognition?: SpeechRecognitionCtor;
    webkitSpeechRecognition?: SpeechRecognitionCtor;
  };
  return w.SpeechRecognition ?? w.webkitSpeechRecognition ?? null;
}
export function sttSupported(): boolean {
  return typeof window !== "undefined" && getSpeechRecognition() != null;
}

export function ttsSupported(): boolean {
  return typeof window !== "undefined" && "speechSynthesis" in window;
}

export interface SpeechListener {
  stop: () => void;
}

export function startSpeechToText(
  onFinal: (text: string) => void,
  onInterim?: (text: string) => void,
  lang = "en-GB",
): SpeechListener {
  const Ctor = getSpeechRecognition();
  if (!Ctor) {
    throw new Error("Speech recognition is not supported in this browser. Try Chrome or Edge.");
  }
  const recognition = new Ctor();
  recognition.lang = lang;
  recognition.continuous = true;
  recognition.interimResults = true;
  recognition.onresult = (event: SpeechRecognitionEvent) => {
    let interim = "";
    let finalText = "";
    for (let i = event.resultIndex; i < event.results.length; i++) {
      const chunk = event.results[i][0]?.transcript ?? "";
      if (event.results[i].isFinal) {
        finalText += chunk;
      } else {
        interim += chunk;
      }
    }
    if (finalText) onFinal(finalText.trim());
    if (interim && onInterim) onInterim(interim.trim());
  };
  recognition.start();
  return {
    stop: () => recognition.stop(),
  };
}

export function speakText(text: string, lang = "en-GB"): void {
  if (!ttsSupported() || !text.trim()) return;
  window.speechSynthesis.cancel();
  const utterance = new SpeechSynthesisUtterance(text.trim());
  utterance.lang = lang;
  utterance.rate = 1;
  window.speechSynthesis.speak(utterance);
}

export function stopSpeaking(): void {
  if (ttsSupported()) {
    window.speechSynthesis.cancel();
  }
}
