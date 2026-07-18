import type { Metadata } from "next";
import { Space_Grotesk, JetBrains_Mono } from "next/font/google";
import localFont from "next/font/local";
import { Analytics } from "@vercel/analytics/react";
import { Providers } from "@/app/components/Providers";
import "./globals.css";

const space = Space_Grotesk({
  subsets: ["latin"],
  variable: "--font-space",
  display: "swap",
});

const mono = JetBrains_Mono({
  subsets: ["latin"],
  variable: "--font-mono",
  display: "swap",
});

const geist = localFont({
  src: "./fonts/Geist-Variable.woff2",
  variable: "--font-geist",
  display: "swap",
  weight: "100 900",
});

export const metadata: Metadata = {
  title: "BugBee — Catch bugs before they bite",
  description:
    "NeuralBroker's AI-powered bug detection platform. Scan every commit, find vulnerabilities, and ship with confidence.",
  metadataBase: new URL("https://getbugbee.vercel.app"),
  openGraph: {
    title: "BugBee — Catch bugs before they bite",
    description:
      "AI engine that scans commits, finds vulnerabilities, and suggests fixes — before your users do.",
    type: "website",
    images: [
      {
        url: "/bugbee-og-v2.png",
        width: 1731,
        height: 909,
        alt: "BugBee — AI-powered bug detection",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "BugBee — Catch bugs before they bite",
    description:
      "AI-powered bug detection from NeuralBroker. Local-first. Defensive-only.",
    images: ["/bugbee-og-v2.png"],
  },
  robots: { index: true, follow: true },
};

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  const jsonLd = {
    "@context": "https://schema.org",
    "@type": "SoftwareApplication",
    name: "BugBee",
    applicationCategory: "DeveloperApplication",
    operatingSystem: "macOS, Linux, Windows",
    description:
      "AI-powered bug detection and code quality platform by NeuralBroker. Local-first, defensive-only, BYOK models.",
    offers: {
      "@type": "Offer",
      price: "0",
      priceCurrency: "USD",
    },
  };

  return (
    <html
      lang="en"
      className={`${space.variable} ${mono.variable} ${geist.variable}`}
    >
      <body>
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }}
        />
        <Providers>{children}</Providers>
        <Analytics />
      </body>
    </html>
  );
}
