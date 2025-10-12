import { Item } from "./item";
import { $windows } from "./state";

export function Bar() {
  return (
    <div class="task-switcher">
      {$windows.value.map((w) => <Item key={w.hwnd} data={w} />)}
    </div>
  );
}
