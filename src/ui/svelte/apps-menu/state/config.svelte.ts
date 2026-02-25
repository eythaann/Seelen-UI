import { Settings, Widget } from "@seelen-ui/lib";
import { lazyRune } from "libs/ui/svelte/utils";
import z from "zod";

let settings = lazyRune(() => Settings.getAsync());
Settings.onChange((s) => (settings.value = s));
await settings.init();

const WidgetConfigSchema = z.object({
  acrylic: z.boolean(),
});

const widgetConfig = $derived.by(
  () =>
    WidgetConfigSchema.safeParse(settings.value.getCurrentWidgetConfig()).data ??
      (Widget.self.getDefaultConfig() as unknown as z.infer<typeof WidgetConfigSchema>),
);

export const ConfigState = {
  get config() {
    return widgetConfig;
  },
};
