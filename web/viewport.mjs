// Keep the canvas inside the visible area when browser controls or keyboards resize it.
export function fitCanvasToViewport(canvas, host) {
  const viewport = host.visualViewport;
  function update() {
    // Preserve the browser's own panning and magnification during pinch zoom.
    if (viewport && Math.abs(viewport.scale - 1) > 0.01) return;
    const width = viewport ? viewport.width : host.innerWidth;
    const height = viewport ? viewport.height : host.innerHeight;
    if (!Number.isFinite(width) || !Number.isFinite(height) || width <= 0 || height <= 0) return;
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;
    canvas.style.left = `${viewport ? viewport.offsetLeft : 0}px`;
    canvas.style.top = `${viewport ? viewport.offsetTop : 0}px`;
  }
  update();
  host.addEventListener("resize", update);
  viewport?.addEventListener("resize", update);
  viewport?.addEventListener("scroll", update);
  return () => {
    host.removeEventListener("resize", update);
    viewport?.removeEventListener("resize", update);
    viewport?.removeEventListener("scroll", update);
  };
}
