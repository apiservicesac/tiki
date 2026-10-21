import type { MoodName } from "../backend/contracts";

export type EyeStyle = "open" | "wide" | "happy" | "closed" | "tired" | "curious";
export type MouthStyle = "smile" | "o" | "flat" | "sad" | "big" | "none";
export type BrowStyle = "worried" | "raised";
export type BadgeStyle = "!" | "+1" | "dots";
export type Decoration = "sweat" | "sparkle" | "sleep";
export type BodyAnimation = "hurry" | "rock";

/** How the mascot looks in one mood. */
export interface MascotAppearance {
  eyes: EyeStyle;
  mouth: MouthStyle;
  /** How strongly the mood color tints the body, from 0 to 1. */
  tint: number;
  color: string;
  cheeks: boolean;
  brows?: BrowStyle;
  badge?: BadgeStyle;
  decoration?: Decoration;
  animation?: BodyAnimation;
}

/** Fixed colors and shapes of Tiki, shared by every renderer. */
export const MASCOT_PALETTE = Object.freeze({
  ink: "#1E1710",
  white: "#FFFFFF",
  bodyTop: "#FFE7AE",
  bodyBottom: "#F6B84A",
  outline: "#D99A2B",
  arm: "#F4BC4E",
  shade: "#E8A53A",
  cheek: "#F2716B",
  tongue: "#FF8FAE",
  tag: "#F2B33D",
  tagOutline: "#B97A0E",
  tagText: "#2B1F07",
  tagHole: "#FFF8EC",
  string: "#D9C49C",
  sweat: "#7DD3FC",
  sparkle: "#F7B32B",
  sleep: "#94A3B8",
  shadow: "#000000",
});

export const MASCOT_SHAPES = Object.freeze({
  viewBox: "0 0 160 160",
  body: "M80 40C117 40 140 64 140 100C140 129 118 144 80 144C42 144 20 129 20 100C20 64 43 40 80 40Z",
  tag: "M0 0H44A6 6 0 0 1 50 6V18A6 6 0 0 1 44 24H0L-11 12Z",
  eyeCenters: Object.freeze([62, 98]),
  eyeHeight: 96,
});

/** One entry per mood; the ticket tag text comes translated from the core. */
export const MASCOT_APPEARANCES: Readonly<Record<MoodName, MascotAppearance>> = Object.freeze({
  calm: { eyes: "open", mouth: "smile", tint: 0, color: "#F2B33D", cheeks: true },
  hurry: { eyes: "wide", mouth: "o", tint: 0.55, color: "#F59E0B", cheeks: false, decoration: "sweat", animation: "hurry" },
  worried: { eyes: "open", mouth: "sad", tint: 0.55, color: "#F4505E", cheeks: false, brows: "worried", badge: "!" },
  alert: { eyes: "wide", mouth: "o", tint: 0.6, color: "#F5A524", cheeks: false, brows: "raised", badge: "!" },
  impatient: { eyes: "tired", mouth: "flat", tint: 0.5, color: "#A855F7", cheeks: false, badge: "dots", animation: "rock" },
  curious: { eyes: "curious", mouth: "o", tint: 0.5, color: "#3B9EFF", cheeks: true, badge: "+1" },
  happy: { eyes: "happy", mouth: "big", tint: 0.35, color: "#34D399", cheeks: true, decoration: "sparkle" },
  sleep: { eyes: "closed", mouth: "none", tint: 0.3, color: "#94A3B8", cheeks: false, decoration: "sleep" },
});
