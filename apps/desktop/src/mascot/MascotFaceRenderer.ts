import { MASCOT_PALETTE, MASCOT_SHAPES, type EyeStyle, type MascotAppearance } from "./mascotAppearance";

/** Draws the eyes, brows, mouth and cheeks. */
export class MascotFaceRenderer {
  private readonly ink = MASCOT_PALETTE.ink;
  private readonly eyeY = MASCOT_SHAPES.eyeHeight;

  render(appearance: MascotAppearance): string {
    return [
      appearance.cheeks ? this.cheeks() : "",
      this.eyes(appearance.eyes),
      this.brows(appearance),
      this.mouth(appearance),
    ].join("");
  }

  private eyes(style: EyeStyle): string {
    const [leftX, rightX] = MASCOT_SHAPES.eyeCenters as readonly [number, number];
    switch (style) {
      case "open":
        return `<g class="mascot-blink">${this.eye(leftX, "open")}${this.eye(rightX, "open")}</g>`;
      case "curious":
        return `<g class="mascot-blink">${this.eye(leftX, "open")}</g>${this.eye(rightX, "wide")}`;
      default:
        return this.eye(leftX, style) + this.eye(rightX, style);
    }
  }

  private eye(centerX: number, style: EyeStyle): string {
    const y = this.eyeY;
    switch (style) {
      case "wide":
        return `<circle cx="${centerX}" cy="${y}" r="10" fill="${MASCOT_PALETTE.white}" stroke="${this.ink}" stroke-width="2.5"/>`
          + `<circle cx="${centerX + 1}" cy="${y}" r="5.5" fill="${this.ink}"/>`
          + `<circle cx="${centerX + 3}" cy="${y - 3}" r="1.8" fill="${MASCOT_PALETTE.white}"/>`;
      case "happy":
        return this.stroke(`M${centerX - 8} ${y + 3}Q${centerX} ${y - 9} ${centerX + 8} ${y + 3}`, 4);
      case "closed":
        return this.stroke(`M${centerX - 8} ${y}Q${centerX} ${y + 6} ${centerX + 8} ${y}`, 3.5);
      case "tired":
        return `<path d="M${centerX - 8} ${y - 1}a8 8 0 0 0 16 0z" fill="${this.ink}"/>`
          + this.stroke(`M${centerX - 10} ${y - 1}H${centerX + 10}`, 3);
      default:
        return `<circle cx="${centerX}" cy="${y}" r="8" fill="${this.ink}"/>`
          + `<circle cx="${centerX + 3}" cy="${y - 3}" r="2.8" fill="${MASCOT_PALETTE.white}"/>`
          + `<circle cx="${centerX - 2.5}" cy="${y + 3}" r="1.2" fill="${MASCOT_PALETTE.white}" opacity=".7"/>`;
    }
  }

  private brows(appearance: MascotAppearance): string {
    if (appearance.brows === "worried") {
      return this.stroke("M51 82L67 77", 3.5) + this.stroke("M93 77L109 82", 3.5);
    }
    if (appearance.brows === "raised") {
      return this.stroke("M53 79L69 77", 3.5) + this.stroke("M91 77L107 79", 3.5);
    }
    return "";
  }

  private mouth(appearance: MascotAppearance): string {
    switch (appearance.mouth) {
      case "smile":
        return this.stroke("M72 117Q80 124 88 117", 3);
      case "o":
        return `<ellipse cx="80" cy="120" rx="3.5" ry="4.6" fill="${this.ink}"/>`;
      case "flat":
        return this.stroke("M74 119H86", 3);
      case "sad":
        return this.stroke("M73 123Q80 116 87 123", 3);
      case "big":
        return `<path d="M70 114Q80 132 90 114Z" fill="${this.ink}"/>`
          + `<ellipse cx="80" cy="123" rx="5" ry="3" fill="${MASCOT_PALETTE.tongue}"/>`;
      case "none":
        return "";
    }
  }

  private cheeks(): string {
    return `<ellipse cx="44" cy="113" rx="8" ry="5" fill="${MASCOT_PALETTE.cheek}" opacity=".55"/>`
      + `<ellipse cx="116" cy="113" rx="8" ry="5" fill="${MASCOT_PALETTE.cheek}" opacity=".55"/>`;
  }

  private stroke(path: string, width: number): string {
    return `<path d="${path}" stroke="${this.ink}" stroke-width="${width}" fill="none" stroke-linecap="round"/>`;
  }
}
