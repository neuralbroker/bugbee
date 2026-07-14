import type { Metadata } from "next";
import localFont from "next/font/local";
import { Analytics } from "@vercel/analytics/react";
import "./globals.css";

const geist = localFont({ src: "./fonts/Geist-Variable.woff2", variable: "--font-geist", display: "swap", weight: "100 900" });

export const metadata: Metadata = {
  title: "Bugbee — Terminal-first AI Security IDE",
  description: "Bugbee is where elite engineers secure software. Local-first AI security analysis with BYOK models.",
  metadataBase: new URL("https://getbugbee.vercel.app"),
  openGraph: { title: "Bugbee — Terminal-first AI Security IDE", description: "Find vulnerabilities before attackers do.", type: "website", images: [{ url: "/bugbee-og-v2.png", width: 1731, height: 909, alt: "Bugbee abstract terminal atmosphere" }] },
  twitter: { card: "summary_large_image", title: "Bugbee — Terminal-first AI Security IDE", description: "Find vulnerabilities before attackers do.", images: ["/bugbee-og-v2.png"] },
  robots: { index: true, follow: true },
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  const jsonLd = { "@context": "https://schema.org", "@type": "SoftwareApplication", name: "Bugbee", applicationCategory: "DeveloperApplication", operatingSystem: "macOS, Linux, Windows", description: "Local-first AI security analysis with BYOK models, evidence scoring, human review, and SARIF export." };
  return <html lang="en" className={geist.variable}><body><script type="application/ld+json" dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }} />{children}<Analytics /></body></html>;
}
