import { Item } from "./item.tsx";
import { $windows } from "./state.ts";

export function Bar() {
  return (
    <div class="task-switcher">
      {$windows.value.map((w, index) => <Item key={w.hwnd} data={w} index={index} />)}
    </div>
  );
}
