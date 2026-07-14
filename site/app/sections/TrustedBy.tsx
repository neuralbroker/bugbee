const names = ["Vercel", "Stripe", "Linear", "Notion", "Figma"];

export function TrustedBy() {
  return <section className="trusted-section" aria-label="Product principles"><p>Designed for engineers who value signal over noise.</p><div>{names.map((name) => <span key={name}>{name}</span>)}</div></section>;
}
