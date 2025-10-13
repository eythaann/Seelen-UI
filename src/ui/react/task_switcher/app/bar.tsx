import { Item } from "./item.tsx";
import { $windows } from "./state.ts";

export function Bar() {
  return (
    <div class="task-switcher">
      {$windows.value.map((w) => <Item key={w.hwnd} data={w} />)}
    </div>
  );
}
