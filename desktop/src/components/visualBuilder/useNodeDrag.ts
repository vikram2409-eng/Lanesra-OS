import { useEffect, useRef } from "react";
import type { PointerEvent as ReactPointerEvent } from "react";

// UX/UI Modernization, Phase A (issue #192): Shared Visual Builder
// Framework - pointer-drag node repositioning, extracted from
// AgentTeamsAdmin.tsx's GraphEditor. Generic over any node shape carrying
// x/y - the caller supplies `getKey` rather than this hook assuming a
// field name, since different builders key their nodes differently
// (Execution Graph nodes use `node_key`, a future Business Rule canvas
// might not).
export function useNodeDrag<TNode extends { x: number; y: number }>(
  setNodes: (updater: (prev: TNode[]) => TNode[]) => void,
  zoom: number,
  readOnly: boolean,
  getKey: (node: TNode) => string,
) {
  const dragRef = useRef<{ key: string; startX: number; startY: number; nodeX: number; nodeY: number } | null>(null);

  useEffect(() => {
    function onMove(e: PointerEvent) {
      if (!dragRef.current) return;
      const d = dragRef.current;
      const dx = (e.clientX - d.startX) / zoom;
      const dy = (e.clientY - d.startY) / zoom;
      setNodes((prev) => prev.map((n) => (getKey(n) === d.key ? { ...n, x: Math.max(0, d.nodeX + dx), y: Math.max(0, d.nodeY + dy) } : n)));
    }
    function onUp() {
      dragRef.current = null;
    }
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    return () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [zoom]);

  function startDrag(e: ReactPointerEvent, node: TNode) {
    if (readOnly) return;
    dragRef.current = { key: getKey(node), startX: e.clientX, startY: e.clientY, nodeX: node.x, nodeY: node.y };
  }

  return { startDrag };
}
