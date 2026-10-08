import { it, expect } from 'vitest';
import { ReplayController } from './replay';
import { record } from './record.fixture';
it('seek and reset use retained record',()=>{const r=new ReplayController();r.load(record);r.step();expect(r.index).toBe(1);r.seek(2);r.reset();expect(r.index).toBe(0);expect(r.record).toBe(record)});
it('reduced motion disables autoplay',()=>{const r=new ReplayController(()=>{},()=>true);r.load(record);r.play();expect(r.playing).toBe(false)});
it('bounds seeks and stops at end',()=>{const r=new ReplayController();r.load(record);r.seek(900);expect(r.index).toBe(2);r.step();expect(r.playing).toBe(false);r.seek(-9);expect(r.index).toBe(0)});
