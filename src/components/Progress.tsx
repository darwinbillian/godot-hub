export function Progress({
  max = 1,
  value = 0,
  ...props
}: React.ComponentPropsWithoutRef<"div"> & {
  max?: number;
  value?: number;
}) {
  const percentage = Math.min(Math.max(value / max, 0), 1);
  return (
    <div {...props}>
      <div style={{ width: `${percentage * 100}%` }} />
    </div>
  );
}
