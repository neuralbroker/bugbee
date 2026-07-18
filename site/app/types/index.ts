export type Severity = "critical" | "warning" | "info";

export interface BugFinding {
  id: string;
  line: number;
  severity: Severity;
  title: string;
  explanation: string;
  fix: string;
  original: string;
  replacement: string;
}

export interface PricingTier {
  id: string;
  name: string;
  price: number | "custom";
  period: string;
  description: string;
  features: string[];
  cta: string;
  popular?: boolean;
}

export interface Testimonial {
  quote: string;
  name: string;
  title: string;
  company: string;
  rating: number;
  initials: string;
}

export interface FAQItem {
  question: string;
  answer: string;
}
