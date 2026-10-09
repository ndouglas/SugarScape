import { checkedInput, object } from './share';
import type { EpisodeClient, EpisodeRecord, Json } from './types';
export const MAX_EPISODE_BYTES = 16 * 1024 * 1024;
export async function readEpisodeFile(file: File): Promise<string> {
  if (file.size > MAX_EPISODE_BYTES) throw new Error('episode exceeds 16 MiB');
  return file.text();
}
/** Failed attempts never replace a successful record; imports never trust their own claims. */
export class EpisodeSession {
  records: EpisodeRecord[] = [];
  shown: EpisodeRecord | null = null;
  editing: Json = null;
  constructor(private client: EpisodeClient) {}
  edit(input: Json): void { this.editing = structuredClone(input); }
  private accept(record: EpisodeRecord): EpisodeRecord { this.records = [...this.records, record].slice(-2); this.shown = record; return record; }
  async run(input: Json): Promise<EpisodeRecord> { return this.accept(await this.client.request('run', checkedInput(input))); }
  async import(text: string): Promise<EpisodeRecord> {
    if (new TextEncoder().encode(text).byteLength > MAX_EPISODE_BYTES) throw new Error('episode exceeds 16 MiB');
    const value: unknown = JSON.parse(text);
    if (!object(value) || value.kind !== 'experiment_episode' || value.version !== 1) throw new Error('incompatible episode file');
    return this.accept(await this.client.request('validate', text));
  }
  async export(): Promise<string> {
    if (!this.shown) throw new Error('run or open an episode first');
    const checked = await this.client.request('validate', JSON.stringify(this.shown));
    return JSON.stringify(checked);
  }
  dispose(): void { this.client.dispose(); this.records = []; this.shown = null; }
}
