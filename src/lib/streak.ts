// Pure helper for the Streak dot grid. Produces 28 cells (4 rows x 7 cols),
// oldest first, last cell = today. A cell is active if that calendar day has
// at least one timestamp in `timestampsMs`.
export interface StreakCell {
  active: boolean;
  today: boolean;
}

export function buildStreakGrid(timestampsMs: number[], now: number = Date.now()): StreakCell[] {
  const days = new Set<string>();
  for (const ts of timestampsMs) days.add(new Date(ts).toDateString());

  const cells: StreakCell[] = [];
  const base = new Date(now);
  for (let i = 27; i >= 0; i--) {
    const d = new Date(base);
    d.setDate(base.getDate() - i);
    cells.push({ active: days.has(d.toDateString()), today: i === 0 });
  }
  return cells;
}
