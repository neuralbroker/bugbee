import { Navigation } from "@/app/sections/Navigation";
import { Hero } from "@/app/sections/Hero";
import { Problem } from "@/app/sections/Problem";
import { Features } from "@/app/sections/Features";
import { HowItWorks } from "@/app/sections/HowItWorks";
import { LiveDemo } from "@/app/sections/LiveDemo";
import { Testimonials } from "@/app/sections/Testimonials";
import { Pricing } from "@/app/sections/Pricing";
import { Stats } from "@/app/sections/Stats";
import { FAQ } from "@/app/sections/FAQ";
import { FinalCTA } from "@/app/sections/FinalCTA";
import { Footer } from "@/app/sections/Footer";

export default function Home() {
  return (
    <>
      <Navigation />
      <main>
        <Hero />
        <Problem />
        <Features />
        <HowItWorks />
        <LiveDemo />
        <Testimonials />
        <Pricing />
        <Stats />
        <FAQ />
        <FinalCTA />
      </main>
      <Footer />
    </>
  );
}
