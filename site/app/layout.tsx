import type { Metadata } from "next";
import "./globals.css";
import "./footer.css";

export const metadata: Metadata = {
  title: "Bugbee — Terminal-first security workbench",
  description: "A local-first security workbench for investigating application risk with evidence and human review.",
  robots: { index: false, follow: false },
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return <html lang="en"><head><link rel="preload" as="image" href="/bugbee-security-space.webp" type="image/webp" /></head><body>{children}<footer className="siteFooter"><a className="brand" href="#top"><img src="/bugbee-mark-light.png" alt="" />bugbee</a><span>Private beta · Defensive security tooling</span></footer></body></html>;
}
