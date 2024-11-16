export default function ExpensesLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <section className="flex flex-col gap-4 py-4 md:py-2">{children}</section>
  );
}
