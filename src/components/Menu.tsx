export type CloseEventHandler = () => void;

export function Menu({
  onClose,
  open,
  ...props
}: React.ComponentPropsWithoutRef<"div"> & {
  onClose?: CloseEventHandler;
  open: boolean;
}) {
  if (!open) {
    return null;
  }

  return (
    <div className="z-10">
      <div
        className="fixed inset-0"
        onClick={() => {
          onClose?.();
        }}
      />
      <div className="z-10">
        <div {...props} />
      </div>
    </div>
  );
}
