import type { MoodName } from "../backend/contracts";
import { MarkupEscaper } from "../dom/MarkupEscaper";
import { FACE_APPEARANCES, FACE_GEOMETRY, type EyeShape } from "./mascotAppearance";

/**
 * Draws Tiki's face as an SVG string: a visor with two eyes whose shape tells the mood.
 * The eyes use `currentColor`, so the stylesheet gives them the mood color (`--mood-<name>`)
 * and every mood color lives in one place.
 */
export class MascotRenderer {
  /** `width` is the face's width in pixels; its height is half of it. */
  render(mood: MoodName, label: string, width: number): string {
    const appearance = FACE_APPEARANCES[mood];
    const height = Math.round(width / FACE_GEOMETRY.aspectRatio);
    const [leftX, rightX] = FACE_GEOMETRY.eyeCenters as readonly [number, number];
    const title = label ? `<title>${MarkupEscaper.escape(label)}</title>` : "";
    const ariaAttributes = label ? `role="img" aria-label="${MarkupEscaper.escape(label)}"` : `aria-hidden="true"`;
    return `<svg class="mascot mascot-${mood}" data-motion="${appearance.motion}" width="${width}" height="${height}" `
      + `viewBox="${FACE_GEOMETRY.viewBox}" ${ariaAttributes}>${title}`
      + this.visor()
      + `<g class="mascot-eyes">`
      + `<g class="mascot-eye">${this.eye(leftX, appearance.leftEye, appearance.eyeTilt)}</g>`
      + `<g class="mascot-eye">${this.eye(rightX, appearance.rightEye, -appearance.eyeTilt)}</g>`
      + `</g></svg>`;
  }

  private visor(): string {
    const radius = FACE_GEOMETRY.visorRadius;
    return `<rect class="mascot-visor" x="1" y="1" width="62" height="30" rx="${radius}"/>`;
  }

  private eye(centerX: number, shape: EyeShape, tilt: number): string {
    const y = FACE_GEOMETRY.eyeLine;
    const rotation = tilt === 0 ? "" : ` transform="rotate(${tilt} ${centerX} ${y})"`;
    switch (shape) {
      case "happy":
        return this.stroke(`M${centerX - 4} ${y + 2}Q${centerX} ${y - 5} ${centerX + 4} ${y + 2}`);
      case "closed":
        return this.stroke(`M${centerX - 4} ${y + 1}Q${centerX} ${y + 3} ${centerX + 4} ${y + 1}`);
      case "lidded":
        return this.pill(centerX, y + 2.5, 7, 5, rotation);
      case "small":
        return this.pill(centerX, y, 5.5, 9, rotation);
      case "wide":
        return this.pill(centerX, y, 7, 11, rotation);
      case "big":
        return this.pill(centerX, y, 8, 13, rotation);
      case "open":
        return this.pill(centerX, y, 6, 11, rotation);
    }
  }

  /** A rounded eye centered on (`centerX`, `centerY`). */
  private pill(centerX: number, centerY: number, width: number, height: number, rotation: string): string {
    return `<rect x="${centerX - width / 2}" y="${centerY - height / 2}" width="${width}" height="${height}" `
      + `rx="${Math.min(width, height) / 2}" fill="currentColor"${rotation}/>`;
  }

  private stroke(path: string): string {
    return `<path d="${path}" stroke="currentColor" stroke-width="2.6" fill="none" stroke-linecap="round"/>`;
  }
}
