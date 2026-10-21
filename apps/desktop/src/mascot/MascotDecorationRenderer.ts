import { MarkupEscaper } from "../dom/MarkupEscaper";
import { MASCOT_PALETTE, MASCOT_SHAPES, type MascotAppearance } from "./mascotAppearance";

/** Draws what hangs around the body: the ticket tag, the badge and the mood decorations. */
export class MascotDecorationRenderer {
  /** The ticket tag hanging from a string, with the mood word on it. */
  ticketTag(label: string): string {
    return `<path d="M70 46C64 30 70 20 81 15" stroke="${MASCOT_PALETTE.string}" stroke-width="3" fill="none" stroke-linecap="round"/>`
      + `<g class="mascot-tag" transform="translate(86 3) rotate(10)">`
      + `<path d="${MASCOT_SHAPES.tag}" fill="${MASCOT_PALETTE.tag}" stroke="${MASCOT_PALETTE.tagOutline}" stroke-width="2" stroke-linejoin="round"/>`
      + `<circle cx="-3" cy="12" r="2.8" fill="${MASCOT_PALETTE.tagHole}" stroke="${MASCOT_PALETTE.tagOutline}" stroke-width="1.5"/>`
      + `<text x="23" y="16" text-anchor="middle" font-family="JetBrains Mono, monospace" font-size="9" font-weight="700" fill="${MASCOT_PALETTE.tagText}">${MarkupEscaper.escape(label)}</text>`
      + `</g>`;
  }

  badge(appearance: MascotAppearance): string {
    if (!appearance.badge) {
      return "";
    }
    const circle = `<circle cx="128" cy="54" r="12" fill="${appearance.color}" stroke="${MASCOT_PALETTE.ink}" stroke-width="2.5"/>`;
    if (appearance.badge === "dots") {
      return circle + [122, 128, 134]
        .map((dotX) => `<circle cx="${dotX}" cy="54" r="2.2" fill="${MASCOT_PALETTE.white}"/>`)
        .join("");
    }
    return circle
      + `<text x="128" y="59" text-anchor="middle" font-size="13" font-weight="800" fill="${MASCOT_PALETTE.white}">${appearance.badge}</text>`;
  }

  decoration(appearance: MascotAppearance): string {
    switch (appearance.decoration) {
      case "sweat":
        return `<path d="M30 70q-5 8 0 11q5-3 0-11z" fill="${MASCOT_PALETTE.sweat}"/>`
          + `<path d="M140 82q-4 7 0 10q4-3 0-10z" fill="${MASCOT_PALETTE.sweat}"/>`;
      case "sparkle":
        return `<path d="M16 50l2 6 6 2-6 2-2 6-2-6-6-2 6-2z" fill="${MASCOT_PALETTE.sparkle}"/>`
          + `<path d="M146 84l2 5 5 2-5 2-2 5-2-5-5-2 5-2z" fill="${MASCOT_PALETTE.sparkle}"/>`;
      case "sleep":
        return `<g class="mascot-sleep"><text x="22" y="46" font-size="16" font-weight="700" fill="${MASCOT_PALETTE.sleep}">z</text>`
          + `<text x="10" y="34" font-size="12" font-weight="700" fill="${MASCOT_PALETTE.sleep}">z</text></g>`;
      default:
        return "";
    }
  }
}
