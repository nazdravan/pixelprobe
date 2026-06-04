// The download button: offer the package for the visitor's system.

/** "windows", "mac", "linux" or null (phones, tablets, unknown). */
export function pickDownload(ua, platform, maxTouchPoints = 0) {
  if (/Android|iPhone|iPad|iPod/i.test(ua)) return null;
  if (/Win/i.test(platform) || /Windows/i.test(ua)) return "windows";
  // iPadOS reports a Mac platform; a touch screen tells them apart.
  if (/Mac/i.test(platform)) return maxTouchPoints > 1 ? null : "mac";
  if (/Linux/i.test(platform) || /X11/.test(ua)) return "linux";
  return null;
}

function init() {
  const button = document.getElementById("download-main");
  if (!button) return;
  let kinds;
  try {
    kinds = JSON.parse(button.dataset.kinds || "{}");
  } catch {
    return;
  }
  const system = pickDownload(navigator.userAgent, navigator.platform || "", navigator.maxTouchPoints || 0);
  const pick = system && kinds[system];
  if (!pick) return;
  button.href = pick.url;
  button.querySelector(".label").textContent = pick.label;
  const note = document.getElementById("download-note");
  if (note) note.textContent = pick.note;
}

if (typeof document !== "undefined") init();
