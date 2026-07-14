import { AppShell } from "./components/AppShell";
import { Architecture } from "./sections/Architecture";
import { DeepEngine } from "./sections/DeepEngine";
import { Enterprise } from "./sections/Enterprise";
import { Footer } from "./sections/Footer";
import { Hero } from "./sections/Hero";
import { InteractiveTerminal } from "./sections/InteractiveTerminal";
import { OpenCore } from "./sections/OpenCore";
import { PrivateBeta } from "./sections/PrivateBeta";
import { TrustedBy } from "./sections/TrustedBy";
import { WhyBugbee } from "./sections/WhyBugbee";
import { Workflow } from "./sections/Workflow";

export default function Home() {
  return <AppShell><main><Hero /><TrustedBy /><InteractiveTerminal /><Workflow /><WhyBugbee /><DeepEngine /><Architecture /><Enterprise /><OpenCore /><PrivateBeta /></main><Footer /></AppShell>;
}
