import { it, expect, vi } from 'vitest';
import { EpisodeSession, readEpisodeFile } from './file';
import { record } from './record.fixture';
import type { EpisodeClient } from './types';
const client=()=>({request:vi.fn().mockResolvedValue(record),cancel:vi.fn(),dispose:vi.fn(),catalog:vi.fn(),recorded:vi.fn()}) as EpisodeClient;
it('editing and failed replacement preserve shown record',async()=>{const c=client();const s=new EpisodeSession(c);await s.run(record.input);const edited={study:'testimony',fixture:'transfer'};s.edit(edited);expect(s.shown?.input).toEqual(record.input);vi.mocked(c.request).mockRejectedValueOnce(Error('invalid'));await expect(s.run(edited)).rejects.toThrow();expect(s.shown).toBe(record)});
it('retains at most two successes',async()=>{const s=new EpisodeSession(client());await s.run(record.input);await s.run(record.input);await s.run(record.input);expect(s.records).toHaveLength(2)});
it('imports and exports worker validated',async()=>{const c=client();const s=new EpisodeSession(c);await s.import(JSON.stringify(record));await s.export();expect(c.request).toHaveBeenCalledWith('validate',JSON.stringify(record));expect(c.request).toHaveBeenCalledTimes(2)});
it('bounds raw files and incompatible envelopes before worker',async()=>{const c=client();const s=new EpisodeSession(c);await expect(s.import(' '.repeat(16*1024*1024+1))).rejects.toThrow();await expect(s.import(JSON.stringify({...record,version:2}))).rejects.toThrow();expect(c.request).not.toHaveBeenCalled();await expect(readEpisodeFile({size:16*1024*1024+1,text:vi.fn()} as unknown as File)).rejects.toThrow()});
