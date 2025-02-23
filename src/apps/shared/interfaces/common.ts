export interface FocusedApp {
  hwnd: number;
  name: string;
  title: string;
  exe: string | null;
  umid: string | null;
  isMaximized: boolean;
}
