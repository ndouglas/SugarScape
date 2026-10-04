import { describe, expect, it } from 'vitest';
import type { DemocraticPeaceConfig, Param } from './types';
const presentation = async () => await import('./democratic-peace').catch(() => ({})) as unknown as { democraticPeaceParams?: (params: Param[]) => Param[]; democraticPeaceLegend?: (config: DemocraticPeaceConfig, mode: string, periods: number) => string; DEMOCRATIC_PEACE_RULES?: string };
describe('Democratic peace source explanations', () => {
  it('explains the printed probability conflict without changing field bounds or choices', async () => {
    const helpers = await presentation();
    expect(helpers.democraticPeaceParams).toBeTypeOf('function');
    const field: Param = { path: 'probability_direction', kind: 'choice', label: 'Probability direction', apply: 'reset', group: 'Readings', choices: [{ value: 'printed_decreasing', label: 'Printed decreasing' }, { value: 'prose_increasing', label: 'Prose increasing' }] };
    const shown = helpers.democraticPeaceParams!([field])[0];
    expect(shown).toMatchObject(field);
    expect(shown.help).toContain('496');
    expect(shown.help).toContain('decreasing');
    expect(shown.help).toContain('increasing');
  });
  it('provides explicit alliance canvas meaning and source/display clocks', async () => {
    const helpers = await presentation();
    expect(helpers.democraticPeaceLegend).toBeTypeOf('function');
    const config = { horizon_periods: 14, periods_per_tick: 3, mechanism: 'alliances', probability_direction: 'printed_decreasing' } as DemocraticPeaceConfig;
    expect(helpers.democraticPeaceLegend!(config, 'alliances', 12)).toContain('12 of 14 source periods');
    expect(helpers.democraticPeaceLegend!(config, 'alliances', 12)).toContain('threat');
    expect(helpers.DEMOCRATIC_PEACE_RULES).toContain('2001');
    expect(helpers.DEMOCRATIC_PEACE_RULES).toContain('stipulated');
  });
});
