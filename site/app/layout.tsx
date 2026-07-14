import type { Metadata } from "next";
import "./globals.css";
import "./footer.css";

export const metadata: Metadata = {
  title: "Bugbee — Terminal-first AI Security IDE",
  description: "Deep local analysis, AI-assisted review, and enterprise security workflows.",
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return <html lang="en"><body>{children}<footer className="siteFooter"><a className="brand" href="#top"><img src="/bugbee-mark-light.png" alt="" />bugbee</a><div><a href="https://github.com/neuralbroker/bugbee">GitHub</a><a href="#">Docs</a><a href="#">Privacy</a><a href="https://www.apache.org/licenses/LICENSE-2.0">Apache 2.0</a></div><span>Made by NeuralBroker</span></footer></body></html>;
}
