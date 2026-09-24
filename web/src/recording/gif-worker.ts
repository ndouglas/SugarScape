// Encodes GIF frames off the main thread (Decision 14): each frame is acknowledged, so the page can
// hold back while this worker is behind; `finish` answers with the file; a failure answers `error`.
import { errorMessage } from '../errors';
import { GifBuilder, type GifReply, type GifRequest } from './gif-encode';

let builder = new GifBuilder();
const reply = (message: GifReply, transfer: Transferable[] = []) => postMessage(message, { transfer });

addEventListener('message', (event: MessageEvent<GifRequest>) => {
  const m = event.data;
  try {
    if (m.type === 'frame') {
      builder.add(new Uint8ClampedArray(m.rgba), m.width, m.height, m.delay);
      reply({ type: 'ack', frames: builder.frames });
      return;
    }
    const bytes = builder.finish().buffer as ArrayBuffer;
    reply({ type: 'done', bytes }, [bytes]);
    builder = new GifBuilder();
  } catch (e) {
    builder = new GifBuilder();
    reply({ type: 'error', message: errorMessage(e) });
  }
});
