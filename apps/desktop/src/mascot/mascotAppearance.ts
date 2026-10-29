import type { MoodName } from "../backend/contracts";

/** How each eye is drawn. */
export type EyeShape = "open" | "wide" | "small" | "happy" | "closed" | "lidded" | "big";

/** A slow, subtle motion that keeps Tiki alive while it rests in a mood. */
export type FaceMotion = "blink" | "glance" | "scan" | "pulse" | "tap" | "tilt" | "bounce" | "breathe";

/** How Tiki's face looks in one mood. The color comes from the `--mood-<name>` token. */
export interface FaceAppearance {
  leftEye: EyeShape;
  rightEye: EyeShape;
  /** Degrees each eye leans towards the middle; a worried look leans them inwards. */
  eyeTilt: number;
  motion: FaceMotion;
}

/**
 * Tiki's face: a dark visor with two eyes, the same in the notch, the card and the settings
 * window. Measures are in the units of `viewBox`; the face is twice as wide as it is tall.
 */
export const FACE_GEOMETRY = Object.freeze({
  viewBox: "0 0 64 32",
  aspectRatio: 2,
  visorRadius: 15,
  eyeCenters: Object.freeze([24, 40]),
  eyeLine: 16,
});

/** One entry per mood. */
export const FACE_APPEARANCES: Readonly<Record<MoodName, FaceAppearance>> = Object.freeze({
  calm: { leftEye: "open", rightEye: "open", eyeTilt: 0, motion: "glance" },
  hurry: { leftEye: "wide", rightEye: "wide", eyeTilt: 0, motion: "scan" },
  worried: { leftEye: "small", rightEye: "small", eyeTilt: 14, motion: "blink" },
  alert: { leftEye: "big", rightEye: "big", eyeTilt: 0, motion: "pulse" },
  impatient: { leftEye: "lidded", rightEye: "lidded", eyeTilt: 0, motion: "tap" },
  curious: { leftEye: "small", rightEye: "big", eyeTilt: 0, motion: "tilt" },
  happy: { leftEye: "happy", rightEye: "happy", eyeTilt: 0, motion: "bounce" },
  sleep: { leftEye: "closed", rightEye: "closed", eyeTilt: 0, motion: "breathe" },
});
