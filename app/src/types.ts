export type DayValue = { date: string; value: number };

export type Snapshot = {
  days: DayValue[];
  total: number;
  best_day: DayValue | null;
  avg_per_active_day: number;
  active_days: number;
  current_streak: number;
  longest_streak: number;
  projects: string[];
  models: string[];
  generated_at: number;
  unreadable_lines: number;
};

export type Metric = "cost" | "billable" | "output" | "raw";
