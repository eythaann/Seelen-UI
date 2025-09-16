import { Desktop, Placement } from "../shared/store/domain";

const desk1 = new Desktop("1", "Desktop 1");
const desk2 = new Desktop("2", "Desktop 2");
const desk3 = new Desktop("3", "Desktop 3");
const desk4 = new Desktop("4", "Desktop 4");

desk1.linkTo(desk2, Placement.Top);
desk2.linkTo(desk3, Placement.Right);
desk3.linkTo(desk4, Placement.Bottom);
desk4.linkTo(desk1, Placement.Left);

desk1.linkTo(desk3, Placement.TopRight);
desk2.linkTo(desk4, Placement.BottomRight);

export const mocked_desktops: Desktop[] = [desk1, desk2, desk3, desk4];
