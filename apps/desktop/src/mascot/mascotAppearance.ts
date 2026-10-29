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
  visor: Object.freeze({ inset: 1, width: 62, height: 30, radius: 15 }),
  eyeCenters: Object.freeze([24, 40]),
  eyeLine: 16,
  /** Width of the drawn eyes (happy, closed) and of the limbs. */
  eyeStroke: 2.6,
  limbStroke: 3.4,
});

/** A rounded eye: its size, and how far below the eye line its center sits. */
interface PillEye {
  width: number;
  height: number;
  drop: number;
}

/** A drawn eye: a curve from one side to the other, its middle `bend` above or below the ends. */
interface CurveEye {
  halfWidth: number;
  ends: number;
  bend: number;
}

/** Every eye shape, measured once. */
export const EYE_SHAPES: Readonly<Record<EyeShape, PillEye | CurveEye>> = Object.freeze({
  open: { width: 6, height: 11, drop: 0 },
  wide: { width: 7, height: 11, drop: 0 },
  small: { width: 5.5, height: 9, drop: 0 },
  big: { width: 8, height: 13, drop: 0 },
  lidded: { width: 7, height: 5, drop: 2.5 },
  happy: { halfWidth: 4, ends: 2, bend: -5 },
  closed: { halfWidth: 4, ends: 1, bend: 3 },
});

/**
 * Tiki with arms and legs, as it springs out to announce an alert: the face drawn
 * `faceShift` units to the right, arms from the sides of the visor and legs from below it.
 * The stylesheet pivots each limb on its shoulder or hip (see island.css).
 */
export const ACTOR_GEOMETRY = Object.freeze({
  viewBox: "0 0 84 50",
  aspectRatio: 84 / 50,
  faceShift: 10,
  shoulders: Object.freeze([Object.freeze([12, 17]), Object.freeze([72, 17])]),
  hips: Object.freeze([Object.freeze([34, 31]), Object.freeze([50, 31])]),
  /** Arms hang at 45° from the shoulders; legs go straight down and end in a small foot. */
  armReach: 8.4,
  legLength: 13,
  footLength: 3,
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
