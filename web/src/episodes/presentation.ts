import { h } from '../ui/dom';
import type { Json } from './types';
export const obj = (value: Json | undefined): Record<string, Json> => value !== null && typeof value === 'object' && !Array.isArray(value) ? value : {};
export const list = (value: Json | undefined): Json[] => Array.isArray(value) ? value : [];
export const label = (value: string): string => value.replace(/_/g, ' ');
/** Exact integer and fraction atoms are rendered as text, without Number conversion. */
export function valueText(value: Json | undefined): string {
  if (value === null || value === undefined) return 'Unavailable';
  if (typeof value === 'boolean') return String(value);
  if (typeof value !== 'object') return String(value);
  if (Array.isArray(value)) return value.map(valueText).join(', ');
  if ('numerator' in value && 'denominator' in value) return `${value.numerator} / ${value.denominator}`;
  return Object.entries(value).map(([key, val]) => `${label(key)}: ${valueText(val)}`).join('; ');
}
export function details(title: string, value: Json | undefined, open = false): HTMLElement {
  const body = h('dl', { class: 'episode-facts' });
  for (const [key, val] of Object.entries(obj(value))) body.append(h('dt', {}, label(key)), h('dd', {}, valueText(val)));
  if (!body.childNodes.length) body.append(h('dd', {}, valueText(value)));
  return h('details', { open }, h('summary', {}, title), body);
}
export function fact(title: string, value: Json | undefined): HTMLElement { return h('p', {}, h('strong', {}, `${title}: `), valueText(value)); }
export function probability(title: string, value: Json | undefined): HTMLElement {
  return h('div', { class: 'episode-probability' }, h('span', {}, title), typeof value === 'number' ? h('meter', { min: 0, max: 1, value, 'aria-label': title }) : null, h('strong', {}, valueText(value)));
}
export function svgElement<K extends keyof SVGElementTagNameMap>(tag: K, attrs: Record<string, string | number>, text?: string): SVGElementTagNameMap[K] {
  const node = document.createElementNS('http://www.w3.org/2000/svg', tag);
  for (const [key, value] of Object.entries(attrs)) node.setAttribute(key, String(value));
  if (text !== undefined) node.textContent = text;
  return node;
}
