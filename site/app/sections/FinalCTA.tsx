"use client";

import dynamic from "next/dynamic";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import { CheckCircle2 } from "lucide-react";
import { FadeIn } from "@/app/components/animations/FadeIn";
import { Button } from "@/app/components/ui/Button";
import { Input } from "@/app/components/ui/Input";

const ParticleField = dynamic(
  () =>
    import("@/app/components/three/ParticleField").then((m) => m.ParticleField),
  { ssr: false }
);

const schema = z.object({
  email: z.string().email("Enter a valid work email"),
});

type FormData = z.infer<typeof schema>;

export function FinalCTA() {
  const [done, setDone] = useState(false);
  const {
    register,
    handleSubmit,
    formState: { errors, isSubmitting },
  } = useForm<FormData>({
    resolver: zodResolver(schema),
  });

  const onSubmit = async (_data: FormData) => {
    await new Promise((r) => setTimeout(r, 600));
    setDone(true);
  };

  return (
    <section
      id="cta"
      className="relative overflow-hidden border-t border-white/[0.06] py-24 md:py-32"
    >
      <ParticleField />
      <div className="container-x relative z-10">
        <FadeIn className="mx-auto max-w-2xl text-center">
          <h2 className="font-display text-[clamp(2rem,4.5vw,3.25rem)] font-semibold leading-[1.1] tracking-[-0.02em] text-white">
            Stop shipping bugs.{" "}
            <span className="text-gradient-amber">
              Start shipping confidence.
            </span>
          </h2>
          <p className="mt-5 text-base text-muted sm:text-lg">
            Join 12,000+ developers who catch vulnerabilities before production.
          </p>

          {done ? (
            <div className="mt-10 inline-flex items-center gap-2 rounded-full border border-success/30 bg-success/10 px-6 py-4 text-success">
              <CheckCircle2 className="h-5 w-5" />
              You&apos;re on the list. We&apos;ll be in touch.
            </div>
          ) : (
            <form
              onSubmit={handleSubmit(onSubmit)}
              className="mx-auto mt-10 flex w-full max-w-md flex-col gap-3 sm:flex-row"
            >
              <div className="flex-1">
                <Input
                  type="email"
                  placeholder="you@company.com"
                  aria-label="Email"
                  {...register("email")}
                />
                {errors.email && (
                  <p className="mt-1.5 text-left text-xs text-error">
                    {errors.email.message}
                  </p>
                )}
              </div>
              <Button type="submit" size="lg" disabled={isSubmitting} magnetic>
                {isSubmitting ? "Joining…" : "Get Started Free"}
              </Button>
            </form>
          )}

          <p className="mt-5 text-sm text-muted">
            No credit card required. Free forever plan available.
          </p>
        </FadeIn>
      </div>
    </section>
  );
}
