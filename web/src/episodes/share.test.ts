import { it, expect } from 'vitest';
import { encodeEpisode, decodeEpisode, readEpisodeHash } from './share';
import { bytesToBase64Url } from '../share';
async function packed(x:unknown){const s=new Blob([JSON.stringify(x)]).stream().pipeThrough(new CompressionStream('deflate-raw'));return bytesToBase64Url(new Uint8Array(await new Response(s).arrayBuffer()))}
it('roundtrips decimal u64 and preserves other routes',async()=>{const input={study:'wink',seed:'18446744073709551615',policy:'evidence',mode:'ordinary'};expect(await decodeEpisode(await encodeEpisode(input))).toEqual(input);expect(readEpisodeHash('#c=abc')).toBeNull();expect(readEpisodeHash('#x=abc')).toBeNull();expect(readEpisodeHash('#e=abc')).toBe('abc')});
it('rejects incompatible links',async()=>{await expect(decodeEpisode(await packed({v:2,input:{study:'wink'}}))).rejects.toThrow();await expect(decodeEpisode('!!')).rejects.toThrow()});
it('bounds compressed bombs at 64 KiB',async()=>{await expect(decodeEpisode(await packed({v:1,input:{study:'wink',junk:'a'.repeat(65536)}}))).rejects.toThrow();await expect(encodeEpisode({study:'wink',junk:'a'.repeat(65536)})).rejects.toThrow()});
it('recognizes malformed and empty episode tokens for startup error handling',()=>{expect(readEpisodeHash('#e=!!')).toBe('!!');expect(readEpisodeHash('#e=')).toBe('');expect(readEpisodeHash('#s=abc')).toBeNull()});
