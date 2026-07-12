export function ProgressBar({
  value,
  max = 1,
}: {
  value: number;
  max?: number;
}) {
  return (
    <div className="rounded-full bg-blue-900">
      <div
        className="h-1 rounded-full bg-blue-500 transition-all duration-400"
        style={{
          width: `${Math.min(Math.max(value / max, 0), 1) * 100}%`,
        }}
      />
    </div>
  );
}
