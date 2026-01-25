

export default function Home() {
  return (
    <main className="min-h-screen bg-background text-foreground">
      <header className="p-4 border-b">
        <h1 className="text-2xl font-bold">NovaFund</h1>
      </header>
      <section className="p-8">
        <h2 className="text-xl">Welcome to NovaFund Collective</h2>
        <p className="mt-2 text-muted-foreground">
          The decentralized micro-investment platform on Stellar.
        </p>
      </section>
    </main>
  );
}
