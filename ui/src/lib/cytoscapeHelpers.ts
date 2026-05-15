import type cytoscape from "cytoscape";
import { NODE_COLORS, FRIENDLY_TYPES } from "./constants";
import type { NodeData } from "./types";

export const NODE_CARD_W = 160;
// Header height is fixed; body grows with content
const CARD_HEADER_H = 24;
const BODY_TEXT_W = NODE_CARD_W - 16; // 8px padding each side

function escapeHtml(str: string): string {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

export function truncateLabel(str: string, maxChars = 80): string {
  if (!str) return "";
  const normalized = str
    .replace(/<[^>]*>/g, " ")
    .replace(/[\r\n\t]+/g, " ")
    .replace(/\s+/g, " ")
    .trim();
  const trimmed =
    normalized.length > maxChars ? normalized.slice(0, maxChars - 1) + "…" : normalized;
  return escapeHtml(trimmed);
}

// Hidden off-screen div used to measure how tall the body text renders at NODE_CARD_W.
// Created once and reused — avoids DOM measurement loops.
let _ruler: HTMLDivElement | null = null;
function getBodyRuler(): HTMLDivElement {
  if (!_ruler) {
    _ruler = document.createElement("div");
    _ruler.style.cssText =
      `position:fixed;top:-9999px;left:-9999px;width:${BODY_TEXT_W}px;` +
      `font-size:10px;font-family:ui-sans-serif,system-ui,sans-serif;` +
      `line-height:1.45;word-break:break-word;padding:5px 8px;` +
      `visibility:hidden;pointer-events:none;`;
    document.body.appendChild(_ruler);
  }
  return _ruler;
}

/**
 * Measures the full card height (header + body) for a given content string.
 * Uses a hidden ruler div so the measurement is synchronous and loop-free.
 */
export function measureCardHeight(content: string): number {
  const ruler = getBodyRuler();
  ruler.innerHTML = content; // innerHTML so escaped entities render correctly
  return CARD_HEADER_H + ruler.offsetHeight;
}

export function buildNodeCardHtml(data: NodeData, selected: boolean, cardHeight: number): string {
  const color = NODE_COLORS[data.node_type] ?? { bg: "#64748b", fg: "#fff" };
  const type = escapeHtml(FRIENDLY_TYPES[data.node_type] ?? data.node_type);
  const content = truncateLabel(data.label ?? "", 80);
  const isCompleted = data.status === "completed";
  const outline = selected ? "outline:2px solid #a78bfa;outline-offset:2px;" : "";
  const bodyH = cardHeight - CARD_HEADER_H;

  const badge = isCompleted
    ? `<span style="flex-shrink:0;width:14px;height:14px;border-radius:50%;background:#22c55e;display:flex;align-items:center;justify-content:center;font-size:8px;color:#fff;line-height:1;">✓</span>`
    : "";

  return (
    `<div style="width:${NODE_CARD_W}px;height:${cardHeight}px;border-radius:6px;overflow:hidden;box-shadow:0 2px 10px rgba(0,0,0,.55);font-family:ui-sans-serif,system-ui,sans-serif;${outline}">` +
    `<div style="background:${color.bg};color:${color.fg};height:${CARD_HEADER_H}px;padding:0 8px;display:flex;align-items:center;justify-content:space-between;gap:4px;">` +
    `<span style="font-size:10px;font-weight:700;letter-spacing:0.5px;text-transform:uppercase;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">${type}</span>` +
    badge +
    `</div>` +
    `<div style="background:#1a1f2e;height:${bodyH}px;padding:5px 8px;font-size:10px;color:#94a3b8;line-height:1.45;overflow:hidden;word-break:break-word;">${content}</div>` +
    `</div>`
  );
}

export function buildCyStyles(): cytoscape.Stylesheet[] {
  return [
    {
      selector: "node",
      style: {
        label: "",
        "background-opacity": 0,
        "border-width": 0,
        width: NODE_CARD_W,
        height: 60, // initial default; updated dynamically per-node after measurement
        padding: 0,
        shape: "roundrectangle",
      } as cytoscape.Css.Node,
    },
    {
      selector: "edge",
      style: {
        label: "data(edge_type)",
        "font-size": "9px",
        color: "#64748b",
        "curve-style": "bezier",
        "target-arrow-shape": "triangle",
        "line-color": "#3d4165",
        "target-arrow-color": "#3d4165",
        width: 1,
        "text-rotation": "autorotate",
      } as cytoscape.Css.Edge,
    },
  ];
}

export function applyFilters(
  cy: cytoscape.Core,
  hiddenNodeTypes: Set<string>,
  hiddenEdgeTypes: Set<string>,
): void {
  cy.elements().show();
  hiddenNodeTypes.forEach((t) => cy.nodes(`[node_type = "${t}"]`).hide());
  hiddenEdgeTypes.forEach((t) => cy.edges(`[edge_type = "${t}"]`).hide());
  cy.nodes(":hidden").connectedEdges().hide();
}
