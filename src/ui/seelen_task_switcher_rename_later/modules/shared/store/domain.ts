import { SimulationLinkDatum, SimulationNodeDatum } from "d3";
import { SoftOpaque, toPlain } from "readable-types";

export enum Placement {
  Left = "left",
  Right = "right",
  Top = "top",
  Bottom = "bottom",
  TopLeft = "top_left",
  TopRight = "top_right",
  BottomLeft = "bottom_left",
  BottomRight = "bottom_right",
}

export type DesktopId = SoftOpaque<string, "DesktopId">;

export type DesktopData = toPlain<Desktop> & SimulationNodeDatum;

export interface DesktopLink extends SimulationLinkDatum<DesktopData> {
  placement: Placement;
}

export class Desktop {
  id: DesktopId;
  name: string;
  left: DesktopId | null = null;
  right: DesktopId | null = null;
  top_left: DesktopId | null = null;
  top: DesktopId | null = null;
  top_right: DesktopId | null = null;
  bottom_left: DesktopId | null = null;
  bottom: DesktopId | null = null;
  bottom_right: DesktopId | null = null;
  preview: string | null = null;

  constructor(id: string, name: string) {
    this.id = id as DesktopId;
    this.name = name;
  }

  asData(): DesktopData {
    return {
      ...this,
    };
  }

  getLinks(): DesktopLink[] {
    const links = new Array<DesktopLink>();
    for (const link of Object.values(Placement)) {
      let target = this[link];
      if (target) {
        links.push({ source: this.id, target, placement: link });
      }
    }
    return links;
  }

  linkTo(desk: Desktop, placement: Placement) {
    this[placement] = desk.id;
  }
}

export interface RootState {
  desktops: Desktop[];
}
