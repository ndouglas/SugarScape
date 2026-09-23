import type { Config } from './types';

/** Scheduled changes and outbreaks as display lines, in tick order (stable on ties). */
export function scheduleLines(config: Config): string[] {
  const lines: { tick: number; text: string }[] = [];
  for (const e of config.schedule) {
    for (const [path, value] of Object.entries(e.set)) {
      lines.push({ tick: e.tick, text: `t = ${e.tick} · ${path} = ${JSON.stringify(value)}` });
    }
  }
  for (const o of config.disease.outbreaks) {
    let text = `t = ${o.tick} · new disease`;
    if (o.length) {
      if (o.length.min === o.length.max) {
        text += ` (${o.length.min} bits)`;
      } else {
        text += ` (${o.length.min}–${o.length.max} bits)`;
      }
    }
    text += ` → ${o.agents} agent${o.agents === 1 ? '' : 's'}`;
    lines.push({ tick: o.tick, text });
  }
  return lines.sort((a, b) => a.tick - b.tick).map((l) => l.text);
}
