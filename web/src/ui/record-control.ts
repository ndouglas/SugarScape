import { errorMessage } from '../errors';
import { formatClock } from '../recording/frames';
import { GIF_LIMIT_NOTICE, startRecording, webmSupport, type RecordFormat, type Recording, type RecordSource } from '../recording/recorder';
import { h } from './dom';
import { showNotice } from './notice';

export interface RecordControl {
  readonly el: HTMLElement;
  /** A displayed snapshot has been drawn: record it (while recording and running). */
  capture(): void;
  /** The run state changed: pause or resume the recording. */
  sync(): void;
}

/** "● Record" with a menu (WebM | GIF, "Stamp the tick"); while recording "■ m:ss" stops and downloads. */
export function buildRecordControl(source: RecordSource): RecordControl {
  let recording: Recording | null = null;
  let stopping = false;
  let timer: ReturnType<typeof setInterval> | undefined;
  const video = webmSupport();
  const stamp = h('input', { type: 'checkbox', checked: true });
  const webm = h(
    'button',
    {
      disabled: !video,
      title: video ? `Record the grid as video (${video.mime})` : 'This browser cannot record video',
      onclick: () => start('webm'),
    },
    'WebM',
  );
  const gif = h(
    'button',
    { title: 'Record the grid as an animated GIF (about 15 frames a second, at most 900 frames)', onclick: () => start('gif') },
    'GIF',
  );
  const menu = h(
    'details',
    { class: 'menu' },
    h('summary', { title: 'Record the grid as it runs' }, '● Record'),
    h('div', { class: 'menu-items' }, webm, gif, h('label', {}, stamp, ' Stamp the tick')),
  );
  const stopButton = h('button', { class: 'record-stop', hidden: true, title: 'Stop recording and download the file', onclick: () => void stop() });

  const show = (): void => {
    menu.hidden = recording !== null;
    stopButton.hidden = recording === null;
    if (recording && !stopping) stopButton.textContent = `■ ${formatClock(recording.elapsed())}`;
  };

  function start(format: RecordFormat): void {
    menu.open = false;
    try {
      recording = startRecording(
        format,
        source,
        stamp.checked,
        () => {
          showNotice(GIF_LIMIT_NOTICE, 10_000);
          void stop();
        },
        // Stopping rejects with the encoder's failure, which the notice below reports.
        () => void stop(),
      );
    } catch (e) {
      showNotice(`Recording could not start (${errorMessage(e)})`, 10_000);
      return;
    }
    timer = setInterval(() => {
      recording?.sync();
      show();
    }, 500);
    show();
  }

  async function stop(): Promise<void> {
    const r = recording;
    if (!r || stopping) return;
    stopping = true;
    clearInterval(timer);
    stopButton.disabled = true;
    stopButton.textContent = 'Saving…';
    try {
      await r.stop((done, total) => (stopButton.textContent = `Finishing GIF… ${done} / ${total}`));
    } catch (e) {
      showNotice(`The recording could not be saved (${errorMessage(e)})`, 10_000);
    } finally {
      recording = null;
      stopping = false;
      stopButton.disabled = false;
      show();
    }
  }

  return {
    el: h('span', { class: 'record' }, menu, stopButton),
    capture: () => {
      if (!stopping) recording?.capture();
    },
    sync: () => {
      if (!stopping) recording?.sync();
    },
  };
}
