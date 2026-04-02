import type cytoscape from 'cytoscape'
import { NODE_COLORS, FRIENDLY_TYPES } from './constants'

export function darkenColor(hex: string, pct: number): string {
  const num = parseInt(hex.replace('#', ''), 16)
  const amt = Math.round(2.55 * pct)
  const r = Math.max(0, (num >> 16) - amt)
  const g = Math.max(0, ((num >> 8) & 0xff) - amt)
  const b = Math.max(0, (num & 0xff) - amt)
  return '#' + [r, g, b].map(v => v.toString(16).padStart(2, '0')).join('')
}

export function wrapText(str: string, maxLen = 80): string {
  if (!str) return ''
  const words = str.split(' ')
  const lines: string[] = []
  let currentLine = ''

  for (const word of words) {
    if (word.length > maxLen) {
      if (currentLine) lines.push(currentLine)
      lines.push(word)
      currentLine = ''
    } else if (currentLine.length + word.length + (currentLine ? 1 : 0) <= maxLen) {
      currentLine = currentLine ? currentLine + ' ' + word : word
    } else {
      lines.push(currentLine)
      currentLine = word
    }
  }
  if (currentLine) lines.push(currentLine)
  return lines.join('\n')
}

export function buildCyStyles(): cytoscape.Stylesheet[] {
  return [
    {
      selector: 'node',
      style: {
        'label': (ele: cytoscape.NodeSingular) =>
          (FRIENDLY_TYPES[ele.data('node_type')] || ele.data('node_type') || '') +
          '\n' + wrapText(ele.data('label') || ''),
        'text-wrap': 'wrap',
        'color': (ele: cytoscape.NodeSingular) =>
          (NODE_COLORS[ele.data('node_type')] || { fg: '#fff' }).fg,
        'background-fill': 'linear-gradient',
        'background-gradient-direction': 'to-bottom',
        'background-gradient-stop-colors': (ele: cytoscape.NodeSingular) => {
          const bg = (NODE_COLORS[ele.data('node_type')] || { bg: '#64748b' }).bg
          const dark = darkenColor(bg, 28)
          return `${dark} ${dark} ${bg} ${bg}`
        },
        'background-gradient-stop-positions': '0 48 48 100',
        'text-valign': 'center',
        'text-halign': 'center',
        'font-size': '11px',
        'width': 'label',
        'height': 'label',
        'padding': '8px',
        'shape': (ele: cytoscape.NodeSingular) => {
          const t = ele.data('node_type')
          if (t === 'context') return 'diamond'
          if (t === 'persona' || t === 'workflow') return 'star'
          return 'roundrectangle'
        },
      } as cytoscape.Css.Node,
    },
    {
      selector: 'edge',
      style: {
        'label': 'data(edge_type)',
        'font-size': '9px',
        'color': '#64748b',
        'curve-style': 'bezier',
        'target-arrow-shape': 'triangle',
        'line-color': '#3d4165',
        'target-arrow-color': '#3d4165',
        'width': 1,
        'text-rotation': 'autorotate',
      } as cytoscape.Css.Edge,
    },
    {
      selector: 'node[status = "completed"]',
      style: {
        'background-image': `data:image/svg+xml,${encodeURIComponent(
          '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16">' +
          '<circle cx="8" cy="8" r="8" fill="#22c55e"/>' +
          '<path d="M3.5 8l3 3 6-6" stroke="white" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round"/>' +
          '</svg>'
        )}`,
        'background-fit': 'none',
        'background-width': '16px',
        'background-height': '16px',
        'background-position-x': '100%',
        'background-position-y': '0%',
        'background-clip': 'none',
        'background-image-opacity': 1,
      } as cytoscape.Css.Node,
    },
    {
      selector: ':selected',
      style: { 'border-width': 2, 'border-color': '#a78bfa' } as cytoscape.Css.Node,
    },
  ]
}

export function applyFilters(
  cy: cytoscape.Core,
  hiddenNodeTypes: Set<string>,
  hiddenEdgeTypes: Set<string>,
): void {
  cy.elements().show()
  hiddenNodeTypes.forEach(t => cy.nodes(`[node_type = "${t}"]`).hide())
  hiddenEdgeTypes.forEach(t => cy.edges(`[edge_type = "${t}"]`).hide())
  cy.nodes(':hidden').connectedEdges().hide()
}
