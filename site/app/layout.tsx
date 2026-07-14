import type { Metadata } from "next";
import { Analytics } from "@vercel/analytics/react";
import "./globals.css";
import "./footer.css";

export const metadata: Metadata = {
  title: "Bugbee — Terminal-first security workbench",
  description: "A local-first security workbench for investigating application risk with evidence and human review.",
  metadataBase: new URL("https://getbugbee.vercel.app"),
  openGraph: {
    title: "Bugbee — Terminal-first security workbench",
    description: "Investigate application risk locally, preserve the evidence, and keep engineers in control.",
    type: "website",
    images: ["/bugbee-security-space.webp"],
  },
  twitter: { card: "summary_large_image", title: "Bugbee — Terminal-first security workbench", description: "A local-first security workbench for engineering teams." },
  robots: { index: false, follow: false },
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  const jsonLd = { "@context": "https://schema.org", "@type": "SoftwareApplication", name: "Bugbee", applicationCategory: "DeveloperApplication", description: "A local-first security workbench for investigating application risk with evidence and human review.", operatingSystem: "macOS, Linux, Windows" };
  return <html lang="en"><head><link rel="preload" as="image" href="/bugbee-security-space.webp" type="image/webp" /><script type="application/ld+json" dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }} /></head><body>{children}<footer className="siteFooter"><a className="brand" href="#top"><img src="/bugbee-mark-light.png" alt="" />bugbee</a><span>Private beta · Defensive security tooling</span></footer><Analytics /></body></html>;
}
