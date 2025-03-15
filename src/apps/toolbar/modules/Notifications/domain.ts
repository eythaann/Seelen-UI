import { Toast } from '@seelen-ui/lib/types';

export interface AppNotification {
  id: number;
  appName: string;
  appDescription: string;
  appUmid: string;
  date: number;
  content: Toast;
}
