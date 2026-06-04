// node --test site/tests/*.test.mjs
import { test } from "node:test";
import assert from "node:assert/strict";
import { pickDownload } from "../assets/main.js";

test("desktop systems", () => {
  assert.equal(pickDownload("Mozilla/5.0 (Windows NT 10.0; Win64; x64)", "Win32"), "windows");
  assert.equal(pickDownload("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5)", "MacIntel", 0), "mac");
  assert.equal(pickDownload("Mozilla/5.0 (X11; Linux x86_64)", "Linux x86_64"), "linux");
});

test("phones and tablets get no package", () => {
  assert.equal(pickDownload("Mozilla/5.0 (Linux; Android 14)", "Linux armv8l"), null);
  assert.equal(pickDownload("Mozilla/5.0 (iPhone; CPU iPhone OS 18_0)", "iPhone"), null);
  // iPadOS reports a Mac.
  assert.equal(pickDownload("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)", "MacIntel", 5), null);
});

test("unknown", () => {
  assert.equal(pickDownload("", ""), null);
});
