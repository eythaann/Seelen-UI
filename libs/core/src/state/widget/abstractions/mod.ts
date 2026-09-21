/**
 * Widget capabilities are split across files (`0_core.ts`, `1_rect.ts`, ...) that
 * form a linear inheritance chain: `Widget_1 extends Widget_0`, `Widget_2 extends Widget_1`,
 * and so on, until `WidgetBasics` extends the last one in the sequence.
 *
 * This is plain single inheritance, not a mixin pattern — JS/TS classes can't extend
 * multiple bases at once, and mixins would add generic/type-composition overhead we don't
 * need here. Every widget wants every capability (core, rect, triggering, ...), there is no
 * case where a widget needs one capability without another, so a fixed linear order is enough.
 *
 * To add a new capability: create the next `N_name.ts` file, have `Widget_N extends Widget_{N-1}`
 * (importing it from the previous file), and update `WidgetBasics` here to extend the new
 * last link in the chain. The numeric prefix only reflects file/require order, not
 * importance.
 */
import { Widget_4 } from "./4_visibility.ts";

export abstract class WidgetBasics extends Widget_4 {}
