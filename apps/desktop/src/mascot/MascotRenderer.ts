import type { MoodName } from "../backend/contracts";
import { MascotDecorationRenderer } from "./MascotDecorationRenderer";
import { MascotFaceRenderer } from "./MascotFaceRenderer";
import { MASCOT_APPEARANCES, MASCOT_PALETTE, MASCOT_SHAPES, type MascotAppearance } from "./mascotAppearance";

/** Draws Tiki as an SVG string for a mood, with the translated word on its ticket tag. */
export class MascotRenderer {
  private readonly face = new MascotFaceRenderer();
  private readonly decorations = new MascotDecorationRenderer();

  render(mood: MoodName, tagLabel: string, size: number): string {
    const appearance = MASCOT_APPEARANCES[mood];
    const tintId = `tiki-tint-${mood}`;
    return `<svg class="mascot mascot-${mood}" width="${size}" height="${size}" viewBox="${MASCOT_SHAPES.viewBox}" aria-hidden="true">`
      + this.gradients(appearance, tintId)
      + `<ellipse cx="80" cy="152" rx="42" ry="5" fill="${MASCOT_PALETTE.shadow}" opacity=".18"/>`
      + `<g class="${appearance.animation ? `mascot-${appearance.animation}` : ""}">`
      + this.decorations.ticketTag(tagLabel)
      + this.arms(mood)
      + this.body(tintId)
      + this.face.render(appearance)
      + this.decorations.badge(appearance)
      + this.decorations.decoration(appearance)
      + `</g></svg>`;
  }

  private gradients(appearance: MascotAppearance, tintId: string): string {
    const tintOpacity = appearance.tint * 0.92;
    return `<defs>`
      + `<linearGradient id="tiki-body" x1="0" y1="0" x2="0" y2="1">`
      + `<stop offset="0" stop-color="${MASCOT_PALETTE.bodyTop}"/><stop offset="1" stop-color="${MASCOT_PALETTE.bodyBottom}"/>`
      + `</linearGradient>`
      + `<linearGradient id="${tintId}" x1="0" y1="1" x2="0" y2="0">`
      + `<stop offset="0" stop-color="${appearance.color}" stop-opacity="${tintOpacity}"/>`
      + `<stop offset="1" stop-color="${appearance.color}" stop-opacity="0"/>`
      + `</linearGradient></defs>`;
  }

  private body(tintId: string): string {
    const shape = MASCOT_SHAPES.body;
    return `<ellipse cx="60" cy="142" rx="13" ry="6" fill="${MASCOT_PALETTE.outline}"/>`
      + `<ellipse cx="100" cy="142" rx="13" ry="6" fill="${MASCOT_PALETTE.outline}"/>`
      + `<path d="${shape}" fill="url(#tiki-body)"/>`
      + `<path d="${shape}" fill="url(#${tintId})"/>`
      + `<path d="${shape}" fill="none" stroke="${MASCOT_PALETTE.outline}" stroke-width="2.2"/>`
      + `<path d="M128 74C138 92 138 116 122 132C130 114 130 94 120 78Z" fill="${MASCOT_PALETTE.shade}" opacity=".45"/>`
      + `<ellipse cx="56" cy="62" rx="20" ry="8" fill="${MASCOT_PALETTE.white}" opacity=".65" transform="rotate(-24 56 62)"/>`;
  }

  private arms(mood: MoodName): string {
    if (mood === "happy") {
      return this.arm(22, 78, -38) + this.arm(138, 78, 38);
    }
    if (mood === "calm") {
      return this.arm(24, 112, 28) + `<g class="mascot-wave">${this.arm(140, 86, 40)}</g>`;
    }
    return this.arm(24, 112, 28) + this.arm(136, 112, -28);
  }

  private arm(centerX: number, centerY: number, rotation: number): string {
    return `<ellipse cx="${centerX}" cy="${centerY}" rx="9" ry="14" fill="${MASCOT_PALETTE.arm}" `
      + `stroke="${MASCOT_PALETTE.outline}" stroke-width="2" transform="rotate(${rotation} ${centerX} ${centerY})"/>`;
  }
}
