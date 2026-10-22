import test from "node:test";
import assert from "node:assert/strict";
import { fitCanvasToViewport } from "./viewport.mjs";

function hostWithViewport(viewport) {
  return Object.assign(new EventTarget(), {
    innerWidth: 390, innerHeight: 844, visualViewport: viewport,
  });
}

test("keyboard resize, browser pan, rotation and keyboard dismissal", () => {
  const viewport = Object.assign(new EventTarget(), {
    width: 390, height: 844, offsetLeft: 0, offsetTop: 0, scale: 1,
  });
  const canvas = { style: {} };
  const dispose = fitCanvasToViewport(canvas, hostWithViewport(viewport));
  viewport.height = 400;
  viewport.offsetTop = 120;
  viewport.dispatchEvent(new Event("resize"));
  assert.equal(canvas.style.height, "400px");
  assert.equal(canvas.style.top, "120px");
  viewport.offsetTop = 80;
  viewport.dispatchEvent(new Event("scroll"));
  assert.equal(canvas.style.top, "80px");
  Object.assign(viewport, { width: 844, height: 390, offsetTop: 0 });
  viewport.dispatchEvent(new Event("resize"));
  assert.equal(canvas.style.width, "844px");
  assert.equal(canvas.style.height, "390px");
  Object.assign(viewport, { width: 390, height: 844 });
  viewport.dispatchEvent(new Event("resize"));
  assert.equal(canvas.style.height, "844px");
  assert.equal(canvas.style.top, "0px");
  dispose();
  viewport.height = 200;
  viewport.dispatchEvent(new Event("resize"));
  assert.equal(canvas.style.height, "844px");
});

test("pinch zoom and transient invalid sizes preserve canvas geometry", () => {
  const viewport = Object.assign(new EventTarget(), {
    width: 390, height: 844, offsetLeft: 0, offsetTop: 0, scale: 1,
  });
  const canvas = { style: {} };
  fitCanvasToViewport(canvas, hostWithViewport(viewport));
  Object.assign(viewport, { scale: 2, width: 195, height: 422, offsetLeft: 30 });
  viewport.dispatchEvent(new Event("resize"));
  assert.equal(canvas.style.width, "390px");
  assert.equal(canvas.style.left, "0px");
  Object.assign(viewport, { scale: 1, height: 0 });
  viewport.dispatchEvent(new Event("resize"));
  assert.equal(canvas.style.height, "844px");
  viewport.height = NaN;
  viewport.dispatchEvent(new Event("resize"));
  assert.equal(canvas.style.height, "844px");
});

test("browsers without VisualViewport use window dimensions", () => {
  const host = hostWithViewport(undefined);
  const canvas = { style: {} };
  const dispose = fitCanvasToViewport(canvas, host);
  assert.equal(canvas.style.height, "844px");
  host.innerHeight = 400;
  host.dispatchEvent(new Event("resize"));
  assert.equal(canvas.style.height, "400px");
  assert.equal(canvas.style.top, "0px");
  dispose();
  host.innerHeight = 200;
  host.dispatchEvent(new Event("resize"));
  assert.equal(canvas.style.height, "400px");
});
