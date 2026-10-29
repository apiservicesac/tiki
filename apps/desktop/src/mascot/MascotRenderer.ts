import type { MoodName } from "../backend/contracts";
import { MarkupEscaper } from "../dom/MarkupEscaper";
import { ACTOR_GEOMETRY, EYE_SHAPES, FACE_APPEARANCES, FACE_GEOMETRY, type EyeShape } from "./mascotAppearance";

/** The outer `<svg>` of a drawing: its mood, accessible name, size and coordinates. */
interface SvgFrame {
  mood: MoodName;
  label: string;
  width: number;
  height: number;
  viewBox: string;
  className: string;
}

/**
 * Draws Tiki as an SVG string: a visor with two eyes whose shape tells the mood. The eyes
 * use `currentColor`, so the stylesheet gives them the mood color (`--mood-<name>`) and
 * every mood color lives in one place.
 *
 * `render` draws the face alone, as it rests in the notch. `renderActor` draws Tiki with
 * arms and legs, as it springs out to announce an alert; the stylesheet grows the limbs
 * and moves them.
 */
export class MascotRenderer {
  /** `width` is the face's width in pixels; its height is half of it. */
  render(mood: MoodName, label: string, width: number): string {
    const height = Math.round(width / FACE_GEOMETRY.aspectRatio);
    return this.svg({ mood, label, width, height, viewBox: FACE_GEOMETRY.viewBox, className: "" }, this.face(mood));
  }

  /** `width` is the actor's width in pixels, arms included. */
  renderActor(mood: MoodName, width: number): string {
    const height = Math.round(width / ACTOR_GEOMETRY.aspectRatio);
    const body = this.limbs() + `<g transform="translate(${ACTOR_GEOMETRY.faceShift} 0)">${this.face(mood)}</g>`;
    return this.svg({ mood, label: "", width, height, viewBox: ACTOR_GEOMETRY.viewBox, className: "mascot-actor" }, body);
  }

  private svg(frame: SvgFrame, body: string): string {
    const { mood, label, width, height, viewBox, className: extraClass } = frame;
    const title = label ? `<title>${MarkupEscaper.escape(label)}</title>` : "";
    const ariaAttributes = label ? `role="img" aria-label="${MarkupEscaper.escape(label)}"` : `aria-hidden="true"`;
    const className = ["mascot", extraClass, `mascot-${mood}`].filter(Boolean).join(" ");
    return `<svg class="${className}" data-motion="${FACE_APPEARANCES[mood].motion}" width="${width}" height="${height}" `
      + `viewBox="${viewBox}" ${ariaAttributes}>${title}${body}</svg>`;
  }

  private face(mood: MoodName): string {
    const appearance = FACE_APPEARANCES[mood];
    const [leftX, rightX] = FACE_GEOMETRY.eyeCenters as readonly [number, number];
    const { inset, width, height, radius } = FACE_GEOMETRY.visor;
    return `<rect class="mascot-visor" x="${inset}" y="${inset}" width="${width}" height="${height}" rx="${radius}"/>`
      + `<g class="mascot-eyes">`
      + `<g class="mascot-eye">${this.eye(leftX, appearance.leftEye, appearance.eyeTilt)}</g>`
      + `<g class="mascot-eye">${this.eye(rightX, appearance.rightEye, -appearance.eyeTilt)}</g>`
      + `</g>`;
  }

  /** Two arms from the sides of the visor and two legs from below it, each in its own group. */
  private limbs(): string {
    const { shoulders, hips, armReach, legLength, footLength } = ACTOR_GEOMETRY;
    const [startShoulder, endShoulder] = shoulders as readonly [readonly [number, number], readonly [number, number]];
    const [startHip, endHip] = hips as readonly [readonly [number, number], readonly [number, number]];
    const arm = (side: "start" | "end", [x, y]: readonly [number, number], outwards: number) =>
      this.limb(`actor-arm actor-arm-${side}`, `M${x} ${y}l${outwards * armReach} ${armReach}`);
    const leg = (side: "start" | "end", [x, y]: readonly [number, number], outwards: number) =>
      this.limb(`actor-leg actor-leg-${side}`, `M${x} ${y}v${legLength}h${outwards * footLength}`);
    return arm("start", startShoulder, -1) + arm("end", endShoulder, 1)
      + leg("start", startHip, -1) + leg("end", endHip, 1);
  }

  private eye(centerX: number, shape: EyeShape, tilt: number): string {
    const y = FACE_GEOMETRY.eyeLine;
    const measures = EYE_SHAPES[shape];
    if ("halfWidth" in measures) {
      const { halfWidth, ends, bend } = measures;
      return this.stroke(`M${centerX - halfWidth} ${y + ends}Q${centerX} ${y + bend} ${centerX + halfWidth} ${y + ends}`);
    }
    const { width, height, drop } = measures;
    const rotation = tilt === 0 ? "" : ` transform="rotate(${tilt} ${centerX} ${y})"`;
    return `<rect x="${centerX - width / 2}" y="${y + drop - height / 2}" width="${width}" height="${height}" `
      + `rx="${Math.min(width, height) / 2}" fill="currentColor"${rotation}/>`;
  }

  private limb(className: string, path: string): string {
    return `<g class="actor-limb ${className}"><path d="${path}" stroke="currentColor" stroke-width="${FACE_GEOMETRY.limbStroke}" `
      + `fill="none" stroke-linecap="round" stroke-linejoin="round"/></g>`;
  }

  private stroke(path: string): string {
    return `<path d="${path}" stroke="currentColor" stroke-width="${FACE_GEOMETRY.eyeStroke}" fill="none" stroke-linecap="round"/>`;
  }
}
