export function trackFamily(track: string): string {
  if (["HH", "RC", "RC2", "C", "C2", "SPL", "CHN"].includes(track)) return "cymbal";
  if (["SD", "BD", "BD2", "T1", "T2", "T3", "T4", "ST"].includes(track)) return "drum";
  if (track === "HF") return "pedal";
  if (["CB", "WB", "CL"].includes(track)) return "percussion";
  return "auxiliary";
}
